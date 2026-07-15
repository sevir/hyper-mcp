#[allow(dead_code)]
mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Map, Value as JsonValue, json};
use urlencoding::encode;

const SERPAPI_SEARCH_URL: &str = "https://serpapi.com/search";

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "serpapi_google_search" => serpapi_google_search(input),
        _ => Ok(error_result(format!("Unknown tool: {}", input.params.name))),
    }
}

fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult {
        is_error: Some(true),
        content: vec![Content {
            annotations: None,
            text: Some(message.into()),
            mime_type: None,
            r#type: ContentType::Text,
            data: None,
        }],
    }
}

fn success_result(text: String) -> CallToolResult {
    CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: None,
            r#type: ContentType::Text,
            data: None,
        }],
    }
}

fn serpapi_google_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let query = match args.get("q") {
        Some(JsonValue::String(value)) if !value.trim().is_empty() => value.trim(),
        _ => {
            return Ok(error_result(
                "Missing or invalid required parameter: q (must be a non-empty string)",
            ));
        }
    };

    if let Err(message) = validate_optional_args(&args) {
        return Ok(error_result(message));
    }

    let api_key = config::get("SERPAPI_API_KEY")?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            Error::msg("SERPAPI_API_KEY runtime configuration is required but not set")
        })?;

    let mut query_params = vec![
        format!("engine=google"),
        format!("q={}", encode(query)),
        format!("api_key={}", encode(&api_key)),
    ];

    add_optional_string(&mut query_params, &args, "location", "location", 200);
    add_optional_string(
        &mut query_params,
        &args,
        "google_domain",
        "google_domain",
        100,
    );
    add_optional_string(&mut query_params, &args, "gl", "gl", 10);
    add_optional_string(&mut query_params, &args, "hl", "hl", 10);
    add_optional_string(&mut query_params, &args, "device", "device", 10);
    add_optional_string(&mut query_params, &args, "safe", "safe", 10);
    add_optional_string(&mut query_params, &args, "filter", "filter", 10);
    add_optional_string(&mut query_params, &args, "time_period", "tbs", 20);
    add_optional_string(&mut query_params, &args, "udm", "udm", 10);
    add_optional_string(&mut query_params, &args, "nfpr", "nfpr", 10);

    add_optional_integer(&mut query_params, &args, "start", 0, 1000);
    add_optional_integer(&mut query_params, &args, "num", 1, 100);

    let url = format!("{SERPAPI_SEARCH_URL}?{}", query_params.join("&"));
    let request = HttpRequest::new(&url).with_method("GET");
    let response = match http::request::<()>(&request, None) {
        Ok(response) => response,
        Err(error) => {
            return Ok(error_result(format!(
                "SerpApi HTTP request failed: {error}"
            )));
        }
    };

    let body = String::from_utf8_lossy(&response.body()).to_string();
    if !(200..300).contains(&response.status_code()) {
        return Ok(error_result(format!(
            "SerpApi returned HTTP {}: {}",
            response.status_code(),
            api_error_message(&body)
        )));
    }

    let response_json = match serde_json::from_str::<JsonValue>(&body) {
        Ok(value) => value,
        Err(error) => {
            return Ok(error_result(format!(
                "Could not parse SerpApi response: {error}"
            )));
        }
    };

    if response_json.get("error").is_some() {
        return Ok(error_result(format!(
            "SerpApi API error: {}",
            api_error_message(&body)
        )));
    }

    match format_results(&response_json) {
        Ok(text) => Ok(success_result(text)),
        Err(message) => Ok(error_result(message)),
    }
}

fn add_optional_string(
    query_params: &mut Vec<String>,
    args: &Map<String, JsonValue>,
    argument: &str,
    parameter: &str,
    max_length: usize,
) {
    if let Some(JsonValue::String(value)) = args.get(argument) {
        let value = value.trim();
        if !value.is_empty() && value.len() <= max_length {
            query_params.push(format!("{parameter}={}", encode(value)));
        }
    }
}

fn add_optional_integer(
    query_params: &mut Vec<String>,
    args: &Map<String, JsonValue>,
    argument: &str,
    minimum: i64,
    maximum: i64,
) {
    if let Some(JsonValue::Number(value)) = args.get(argument)
        && let Some(value) = value.as_i64()
        && (minimum..=maximum).contains(&value)
    {
        query_params.push(format!("{argument}={value}"));
    }
}

fn validate_optional_args(args: &Map<String, JsonValue>) -> Result<(), String> {
    for (name, maximum) in [
        ("location", 200),
        ("google_domain", 100),
        ("gl", 10),
        ("hl", 10),
        ("time_period", 20),
        ("udm", 10),
    ] {
        if let Some(value) = args.get(name) {
            match value {
                JsonValue::String(value) if !value.trim().is_empty() && value.len() <= maximum => {}
                JsonValue::String(_) => {
                    return Err(format!(
                        "Invalid optional parameter: {name} must be non-empty and at most {maximum} characters"
                    ));
                }
                _ => {
                    return Err(format!(
                        "Invalid optional parameter: {name} must be a string"
                    ));
                }
            }
        }
    }

    for name in ["device", "safe", "filter", "nfpr"] {
        if let Some(value) = args.get(name) {
            let value = value
                .as_str()
                .ok_or_else(|| format!("Invalid optional parameter: {name} must be a string"))?;
            let valid = match name {
                "device" => matches!(value, "desktop" | "mobile" | "tablet"),
                "safe" => matches!(value, "active" | "off"),
                "filter" | "nfpr" => matches!(value, "0" | "1"),
                _ => false,
            };
            if !valid {
                return Err(format!("Invalid optional parameter: {name}"));
            }
        }
    }

    if let Some(JsonValue::String(value)) = args.get("time_period")
        && !value.starts_with("qdr:")
    {
        return Err(
            "Invalid optional parameter: time_period must use a SerpApi tbs value such as qdr:d"
                .into(),
        );
    }

    for (name, minimum, maximum) in [("start", 0, 1000), ("num", 1, 100)] {
        if let Some(value) = args.get(name) {
            let value = value
                .as_i64()
                .ok_or_else(|| format!("Invalid optional parameter: {name} must be an integer"))?;
            if !(minimum..=maximum).contains(&value) {
                return Err(format!(
                    "Invalid optional parameter: {name} must be between {minimum} and {maximum}"
                ));
            }
        }
    }

    Ok(())
}

fn api_error_message(body: &str) -> String {
    serde_json::from_str::<JsonValue>(body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .and_then(JsonValue::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| body.chars().take(500).collect())
}

fn format_results(response: &JsonValue) -> Result<String, String> {
    let mut lines = vec!["SerpApi Google Search Results".to_string()];

    if let Some(search_metadata) = response.get("search_metadata")
        && let Some(status) = search_metadata.get("status").and_then(JsonValue::as_str)
    {
        lines.push(format!("Status: {status}"));
    }
    if let Some(search_information) = response.get("search_information") {
        if let Some(total) = search_information
            .get("total_results")
            .and_then(JsonValue::as_u64)
        {
            lines.push(format!("Total results: {total}"));
        }
        if let Some(time) = search_information
            .get("time_taken_displayed")
            .and_then(JsonValue::as_str)
        {
            lines.push(format!("Search time: {time}"));
        }
    }

    let results = response
        .get("organic_results")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "SerpApi response did not contain organic_results".to_string())?;

    if results.is_empty() {
        lines.push("No organic search results found.".to_string());
    } else {
        lines.push(String::new());
        lines.push("Organic results:".to_string());
        for (index, result) in results.iter().enumerate() {
            let title = result
                .get("title")
                .and_then(JsonValue::as_str)
                .unwrap_or("Untitled");
            let link = result
                .get("link")
                .and_then(JsonValue::as_str)
                .unwrap_or("No URL");
            lines.push(format!("{}. {}", index + 1, title));
            lines.push(format!("   URL: {link}"));
            if let Some(snippet) = result.get("snippet").and_then(JsonValue::as_str) {
                lines.push(format!("   {snippet}"));
            }
            if let Some(date) = result.get("date").and_then(JsonValue::as_str) {
                lines.push(format!("   Date: {date}"));
            }
        }
    }

    if let Some(answer_box) = response.get("answer_box") {
        lines.push(String::new());
        lines.push("Answer box:".to_string());
        if let Some(answer) = answer_box.get("answer").and_then(JsonValue::as_str) {
            lines.push(answer.to_string());
        } else if let Some(snippet) = answer_box.get("snippet").and_then(JsonValue::as_str) {
            lines.push(snippet.to_string());
        }
    }

    Ok(lines.join("\n"))
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![ToolDescription {
            name: "serpapi_google_search".into(),
            description: "Search Google through SerpApi and return readable result text. The API key is supplied only through SERPAPI_API_KEY runtime configuration.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "q": { "type": "string", "description": "Google search query; required and non-empty." },
                    "location": { "type": "string", "description": "SerpApi location, for example \"Austin, Texas, United States\"." },
                    "google_domain": { "type": "string", "description": "Google domain, for example \"google.com\"." },
                    "gl": { "type": "string", "description": "Two-letter country code." },
                    "hl": { "type": "string", "description": "Two-letter language code." },
                    "device": { "type": "string", "enum": ["desktop", "mobile", "tablet"] },
                    "safe": { "type": "string", "enum": ["active", "off"] },
                    "filter": { "type": "string", "enum": ["0", "1"] },
                    "time_period": { "type": "string", "description": "SerpApi tbs value, such as \"qdr:d\", \"qdr:w\", or \"qdr:m\"." },
                    "udm": { "type": "string", "description": "Google vertical mode, such as \"2\" for images." },
                    "nfpr": { "type": "string", "enum": ["0", "1"] },
                    "start": { "type": "integer", "minimum": 0, "maximum": 1000 },
                    "num": { "type": "integer", "minimum": 1, "maximum": 100 }
                },
                "required": ["q"]
            }).as_object().unwrap().clone(),
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_organic_results_as_text() {
        let response = json!({
            "search_information": { "total_results": 1 },
            "organic_results": [{ "title": "Example", "link": "https://example.com", "snippet": "A result." }]
        });
        let text = format_results(&response).unwrap();
        assert!(text.contains("1. Example"));
        assert!(text.contains("URL: https://example.com"));
        assert!(!text.trim_start().starts_with('{'));
    }

    #[test]
    fn rejects_missing_results() {
        assert!(format_results(&json!({})).is_err());
    }
}
