#![allow(dead_code)]

mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Map, Value, json};

const SEARCH_URL: &str = "https://www.searchapi.io/api/v1/search";
const TOOL_NAME: &str = "searchapi_google_search";

fn text_result(text: impl Into<String>, is_error: bool) -> CallToolResult {
    CallToolResult {
        is_error: Some(is_error),
        content: vec![Content {
            annotations: None,
            text: Some(text.into()),
            mime_type: None,
            r#type: ContentType::Text,
            data: None,
        }],
    }
}

fn string_arg<'a>(args: &'a Map<String, Value>, name: &str) -> Result<Option<&'a str>, String> {
    match args.get(name) {
        None => Ok(None),
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(Some(value)),
        Some(_) => Err(format!("Parameter '{name}' must be a non-empty string")),
    }
}

fn integer_arg(
    args: &Map<String, Value>,
    name: &str,
    min: i64,
    max: i64,
) -> Result<Option<i64>, String> {
    match args.get(name) {
        None => Ok(None),
        Some(Value::Number(value)) => match value.as_i64() {
            Some(value) if (min..=max).contains(&value) => Ok(Some(value)),
            _ => Err(format!(
                "Parameter '{name}' must be an integer from {min} to {max}"
            )),
        },
        Some(_) => Err(format!("Parameter '{name}' must be an integer")),
    }
}

fn enum_arg<'a>(
    args: &'a Map<String, Value>,
    name: &str,
    allowed: &[&str],
) -> Result<Option<&'a str>, String> {
    let value = string_arg(args, name)?;
    if let Some(value) = value {
        if allowed.contains(&value) {
            return Ok(Some(value));
        }
        return Err(format!(
            "Parameter '{name}' must be one of: {}",
            allowed.join(", ")
        ));
    }
    Ok(None)
}

fn google_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let query = match string_arg(&args, "q") {
        Ok(Some(value)) => value,
        Ok(None) => return Ok(text_result("Missing required parameter: q", true)),
        Err(error) => return Ok(text_result(error, true)),
    };

    let api_key = match config::get("SEARCHAPI_API_KEY")? {
        Some(value) if !value.trim().is_empty() => value,
        _ => {
            return Ok(text_result(
                "SEARCHAPI_API_KEY runtime configuration is required but not set",
                true,
            ));
        }
    };

    let mut params = vec![("engine", "google".to_string()), ("q", query.to_string())];
    let options = ["gl", "hl", "device"];
    for name in options {
        match string_arg(&args, name) {
            Ok(Some(value)) => params.push((name, value.to_string())),
            Ok(None) => {}
            Err(error) => return Ok(text_result(error, true)),
        }
    }
    match integer_arg(&args, "page", 1, 100) {
        Ok(Some(value)) => params.push(("page", value.to_string())),
        Ok(None) => {}
        Err(error) => return Ok(text_result(error, true)),
    }
    for (name, allowed) in [
        ("safe", &["active", "blur", "off"][..]),
        (
            "time_period",
            &[
                "last_1_minute",
                "last_5_minutes",
                "last_15_minutes",
                "last_30_minutes",
                "last_hour",
                "last_day",
                "last_week",
                "last_month",
                "last_year",
            ][..],
        ),
    ] {
        match enum_arg(&args, name, allowed) {
            Ok(Some(value)) => params.push((name, value.to_string())),
            Ok(None) => {}
            Err(error) => return Ok(text_result(error, true)),
        }
    }

    let query_string = params
        .iter()
        .map(|(name, value)| {
            format!(
                "{}={}",
                urlencoding::encode(name),
                urlencoding::encode(value)
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    let request = HttpRequest::new(format!("{SEARCH_URL}?{query_string}"))
        .with_method("GET")
        .with_header("Authorization", format!("Bearer {api_key}"));
    let response = match http::request::<String>(&request, None) {
        Ok(response) => response,
        Err(error) => {
            return Ok(text_result(
                format!("SearchAPI request failed: {error}"),
                true,
            ));
        }
    };
    let body = String::from_utf8_lossy(&response.body()).to_string();
    if !(200..300).contains(&response.status_code()) {
        let detail = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .or_else(|| value.get("message").and_then(Value::as_str))
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "SearchAPI returned an error".to_string());
        return Ok(text_result(
            format!(
                "SearchAPI error (HTTP {}): {detail}",
                response.status_code()
            ),
            true,
        ));
    }

    let payload: Value = match serde_json::from_str(&body) {
        Ok(value) => value,
        Err(_) => return Ok(text_result("SearchAPI returned invalid JSON", true)),
    };
    Ok(text_result(format_results(&payload, query), false))
}

fn format_results(payload: &Value, query: &str) -> String {
    let mut output = format!("Google search results for: {query}\n\n");
    if let Some(answer) = payload
        .get("answer_box")
        .and_then(|v| v.get("answer"))
        .and_then(Value::as_str)
    {
        output.push_str(&format!("Answer\n{answer}\n\n"));
    }
    let results = payload.get("organic_results").and_then(Value::as_array);
    if let Some(results) = results {
        for (index, result) in results.iter().enumerate() {
            let title = result
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("Untitled");
            let link = result.get("link").and_then(Value::as_str).unwrap_or("");
            let snippet = result
                .get("snippet")
                .and_then(Value::as_str)
                .unwrap_or("No description available.");
            output.push_str(&format!(
                "{}. {title}\n   {link}\n   {snippet}\n\n",
                index + 1
            ));
        }
    }
    if results.is_none_or(Vec::is_empty) {
        output.push_str("No organic results found.\n");
    }
    output
}

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        TOOL_NAME => google_search(input),
        name => Ok(text_result(format!("Unknown tool: {name}"), true)),
    }
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![ToolDescription {
            name: TOOL_NAME.into(),
            description: "Searches Google through SearchAPI.io and returns readable result titles, links, snippets, and answer-box text.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "q": {"type": "string", "description": "Google search query."},
                    "page": {"type": "integer", "minimum": 1, "maximum": 100, "description": "Results page number."},
                    "gl": {"type": "string", "description": "Google country code, for example us."},
                    "hl": {"type": "string", "description": "Google interface language code, for example en."},
                    "device": {"type": "string", "description": "Search device, for example desktop, mobile, or tablet."},
                    "time_period": {"type": "string", "enum": ["last_1_minute", "last_5_minutes", "last_15_minutes", "last_30_minutes", "last_hour", "last_day", "last_week", "last_month", "last_year"]},
                    "safe": {"type": "string", "enum": ["active", "blur", "off"]}
                },
                "required": ["q"]
            }).as_object().unwrap().clone(),
        }],
    })
}
