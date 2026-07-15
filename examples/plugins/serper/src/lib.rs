#[allow(dead_code)]
mod pdk;

use std::collections::BTreeMap;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value, json};

const SERPER_SEARCH_URL: &str = "https://google.serper.dev/search";

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

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "serper_web_search" => serper_web_search(input),
        _ => Ok(text_result(
            format!("Unknown tool: {}", input.params.name),
            true,
        )),
    }
}

fn serper_web_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let query = match args.get("q") {
        Some(Value::String(q)) if !q.trim().is_empty() => q.trim().to_owned(),
        _ => {
            return Ok(text_result(
                "Missing or invalid required parameter: q (must be a non-empty string)",
                true,
            ));
        }
    };

    let api_key = match config::get("SERPER_API_KEY") {
        Ok(Some(key)) if !key.trim().is_empty() => key,
        Ok(_) => {
            return Ok(text_result(
                "SERPER_API_KEY configuration is required but not set",
                true,
            ));
        }
        Err(error) => {
            return Ok(text_result(
                format!("Could not read Serper API configuration: {error}"),
                true,
            ));
        }
    };

    let mut headers = BTreeMap::new();
    headers.insert("X-API-KEY".to_string(), api_key);
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = HttpRequest {
        url: SERPER_SEARCH_URL.to_string(),
        headers,
        method: Some("POST".to_string()),
    };

    let body = json!({ "q": query });
    let res = match http::request(&req, Some(&body.to_string())) {
        Ok(response) => response,
        Err(error) => return Ok(text_result(format!("Serper request failed: {error}"), true)),
    };
    let response_text = String::from_utf8_lossy(&res.body()).to_string();
    let response: Value = match serde_json::from_str(&response_text) {
        Ok(value) => value,
        Err(error) => {
            return Ok(text_result(
                format!("Serper returned an invalid JSON response: {error}"),
                true,
            ));
        }
    };

    if !(200..300).contains(&res.status_code()) {
        let detail = response
            .get("message")
            .and_then(Value::as_str)
            .or_else(|| response.get("error").and_then(Value::as_str))
            .unwrap_or("The Serper API rejected the request");
        return Ok(text_result(
            format!("Serper API error (HTTP {}): {detail}", res.status_code()),
            true,
        ));
    }

    Ok(text_result(format_search_results(&response), false))
}

fn format_search_results(response: &Value) -> String {
    let mut output = Vec::new();
    if let Some(answer) = response.get("answerBox").and_then(Value::as_object)
        && let Some(answer_text) = answer
            .get("answer")
            .or_else(|| answer.get("snippet"))
            .and_then(Value::as_str)
    {
        output.push(format!("Answer: {answer_text}"));
    }
    if let Some(knowledge) = response.get("knowledgeGraph").and_then(Value::as_object) {
        if let Some(title) = knowledge.get("title").and_then(Value::as_str) {
            output.push(format!("Knowledge Graph: {title}"));
        }
        if let Some(description) = knowledge.get("description").and_then(Value::as_str) {
            output.push(description.to_owned());
        }
    }
    match response.get("organic").and_then(Value::as_array) {
        Some(results) if !results.is_empty() => {
            output.push("Organic Results:".to_owned());
            for (index, result) in results.iter().enumerate() {
                let title = result
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("Untitled result");
                let link = result
                    .get("link")
                    .and_then(Value::as_str)
                    .unwrap_or("URL unavailable");
                output.push(format!("{}. {title}\n   {link}", index + 1));
                if let Some(snippet) = result.get("snippet").and_then(Value::as_str) {
                    output.push(format!("   {snippet}"));
                }
            }
        }
        Some(_) => output.push("No organic search results found.".to_owned()),
        None => output.push("Serper returned no recognized search results.".to_owned()),
    }
    if let Some(questions) = response.get("peopleAlsoAsk").and_then(Value::as_array)
        && !questions.is_empty()
    {
        output.push("People Also Ask:".to_owned());
        for (index, question) in questions.iter().enumerate() {
            if let Some(question) = question.get("question").and_then(Value::as_str) {
                output.push(format!("{}. {question}", index + 1));
            }
        }
    }
    if let Some(searches) = response.get("relatedSearches").and_then(Value::as_array)
        && !searches.is_empty()
    {
        let queries = searches
            .iter()
            .filter_map(|search| search.get("query").and_then(Value::as_str))
            .collect::<Vec<_>>();
        if !queries.is_empty() {
            output.push(format!("Related Searches: {}", queries.join(", ")));
        }
    }
    output.join("\n\n")
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult{
        tools: vec![
            ToolDescription {
                name: "serper_web_search".into(),
                description: "Performs a Google web search using Serper and returns formatted answers and organic results.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "q": {
                            "type": "string",
                            "description": "The search query string",
                        },
                    },
                    "required": ["q"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
