mod pdk;

use std::collections::BTreeMap;

use extism_pdk::*;
use json::Value;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::json;

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "serper_web_search" => serper_web_search(input),
        _ => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Unknown tool: {}", input.params.name)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

fn serper_web_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let query = match args.get("q") {
        Some(Value::String(q)) => q.clone(),
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Please provide a 'q' argument for the search query".into()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let api_key = config::get("SERPER_API_KEY")?
        .ok_or_else(|| Error::msg("SERPER_API_KEY configuration is required but not set"))?;

    let mut headers = BTreeMap::new();
    headers.insert("X-API-KEY".to_string(), api_key);
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = HttpRequest {
        url: "https://google.serper.dev/search".to_string(),
        headers,
        method: Some("POST".to_string()),
    };

    let body = json!({ "q": query });
    let res = http::request(&req, Some(&body.to_string()))?;
    let response_body = res.body();
    let response_text = String::from_utf8_lossy(response_body.as_slice()).to_string();

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(response_text),
            mime_type: Some("application/json".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult{
        tools: vec![
            ToolDescription {
                name: "serper_web_search".into(),
                description:  "Performs a Google web search using the Serper API and returns the raw JSON response for the given query string.".into(),
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
