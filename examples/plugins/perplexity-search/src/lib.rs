mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value as JsonValue, json};

const PERPLEXITY_API_BASE_URL: &str = "https://api.perplexity.ai/chat/completions";

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "perplexity_search" => perplexity_search(input),
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

fn perplexity_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    // Extract required parameters
    let query_val = args.get("query").unwrap_or(&JsonValue::Null);
    let api_key_val = args.get("api_key").unwrap_or(&JsonValue::Null);

    // Validate required parameters
    let query = match query_val {
        JsonValue::String(s) if !s.is_empty() => s,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: query (must be non-empty string)".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let api_key = match api_key_val {
        JsonValue::String(s) if !s.is_empty() => s,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: api_key (must be non-empty string)".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    // Build request payload
    let mut messages = vec![
        json!({
            "role": "user",
            "content": query
        })
    ];

    // Optional system message
    if let Some(JsonValue::String(system_msg)) = args.get("system_message") {
        if !system_msg.is_empty() {
            messages.insert(0, json!({
                "role": "system",
                "content": system_msg
            }));
        }
    }

    let mut request_body = json!({
        "model": "sonar-pro",
        "messages": messages,
        "max_tokens": 1000,
        "temperature": 0.2,
        "top_p": 0.9,
        "return_citations": true,
        "return_images": false,
        "return_related_questions": false,
        "search_recency_filter": "month"
    });

    // Optional model selection
    if let Some(JsonValue::String(model)) = args.get("model") {
        match model.as_str() {
            "sonar-pro" | "sonar-reasoning" | "sonar-deep-research" => {
                request_body["model"] = json!(model);
            }
            _ => {}
        }
    }

    // Optional max tokens
    if let Some(JsonValue::Number(max_tokens)) = args.get("max_tokens") {
        if let Some(tokens_val) = max_tokens.as_i64() {
            if tokens_val >= 1 && tokens_val <= 4096 {
                request_body["max_tokens"] = json!(tokens_val);
            }
        }
    }

    // Optional temperature
    if let Some(JsonValue::Number(temp)) = args.get("temperature") {
        if let Some(temp_val) = temp.as_f64() {
            if temp_val >= 0.0 && temp_val <= 2.0 {
                request_body["temperature"] = json!(temp_val);
            }
        }
    }

    // Optional search recency filter
    if let Some(JsonValue::String(recency)) = args.get("search_recency_filter") {
        match recency.as_str() {
            "month" | "week" | "day" | "hour" => {
                request_body["search_recency_filter"] = json!(recency);
            }
            _ => {}
        }
    }

    // Optional return citations
    if let Some(JsonValue::Bool(citations)) = args.get("return_citations") {
        request_body["return_citations"] = json!(citations);
    }

    // Optional return images
    if let Some(JsonValue::Bool(images)) = args.get("return_images") {
        request_body["return_images"] = json!(images);
    }

    // Optional return related questions
    if let Some(JsonValue::Bool(related)) = args.get("return_related_questions") {
        request_body["return_related_questions"] = json!(related);
    }

    // Make the HTTP request
    let req = HttpRequest::new(PERPLEXITY_API_BASE_URL)
        .with_method("POST")
        .with_header("Authorization", &format!("Bearer {}", api_key))
        .with_header("Content-Type", "application/json");

    match http::request::<String>(&req, Some(request_body.to_string())) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut results_text_parts = Vec::new();

                        // Extract the main response content
                        if let Some(choices) = parsed_json.get("choices") {
                            if let JsonValue::Array(choices_array) = choices {
                                if let Some(first_choice) = choices_array.first() {
                                    if let Some(message) = first_choice.get("message") {
                                        if let Some(content) = message.get("content") {
                                            if let Some(content_str) = content.as_str() {
                                                results_text_parts.push("Response:".to_string());
                                                results_text_parts.push(content_str.to_string());
                                                results_text_parts.push("".to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Extract citations
                        if let Some(citations) = parsed_json.get("citations") {
                            if let JsonValue::Array(citations_array) = citations {
                                if !citations_array.is_empty() {
                                    results_text_parts.push("Sources:".to_string());
                                    for (index, citation) in citations_array.iter().enumerate() {
                                        if let Some(citation_str) = citation.as_str() {
                                            results_text_parts.push(format!("{}. {}", index + 1, citation_str));
                                        }
                                    }
                                    results_text_parts.push("".to_string());
                                }
                            }
                        }

                        // Extract search results
                        if let Some(search_results) = parsed_json.get("search_results") {
                            if let JsonValue::Array(results_array) = search_results {
                                if !results_array.is_empty() {
                                    results_text_parts.push("Search Results:".to_string());
                                    results_text_parts.push("".to_string());

                                    for (index, result) in results_array.iter().take(5).enumerate() {
                                        let result_num = index + 1;
                                        results_text_parts.push(format!("{}.", result_num));

                                        if let Some(title) = result.get("title") {
                                            if let Some(title_str) = title.as_str() {
                                                results_text_parts.push(format!("   Title: {}", title_str));
                                            }
                                        }

                                        if let Some(url) = result.get("url") {
                                            if let Some(url_str) = url.as_str() {
                                                results_text_parts.push(format!("   URL: {}", url_str));
                                            }
                                        }

                                        if let Some(snippet) = result.get("snippet") {
                                            if let Some(snippet_str) = snippet.as_str() {
                                                results_text_parts.push(format!("   Snippet: {}", snippet_str));
                                            }
                                        }

                                        if let Some(date) = result.get("date") {
                                            if let Some(date_str) = date.as_str() {
                                                results_text_parts.push(format!("   Date: {}", date_str));
                                            }
                                        }

                                        results_text_parts.push("".to_string());
                                    }
                                }
                            }
                        }

                        // Extract usage information
                        if let Some(usage) = parsed_json.get("usage") {
                            results_text_parts.push("Usage Information:".to_string());

                            if let Some(prompt_tokens) = usage.get("prompt_tokens") {
                                if let Some(tokens) = prompt_tokens.as_i64() {
                                    results_text_parts.push(format!("   Prompt tokens: {}", tokens));
                                }
                            }

                            if let Some(completion_tokens) = usage.get("completion_tokens") {
                                if let Some(tokens) = completion_tokens.as_i64() {
                                    results_text_parts.push(format!("   Completion tokens: {}", tokens));
                                }
                            }

                            if let Some(total_tokens) = usage.get("total_tokens") {
                                if let Some(tokens) = total_tokens.as_i64() {
                                    results_text_parts.push(format!("   Total tokens: {}", tokens));
                                }
                            }

                            if let Some(cost) = usage.get("cost") {
                                if let Some(total_cost) = cost.get("total_cost") {
                                    if let Some(cost_val) = total_cost.as_f64() {
                                        results_text_parts.push(format!("   Total cost: ${:.4}", cost_val));
                                    }
                                }
                            }

                            results_text_parts.push("".to_string());
                        }

                        // Handle case where no results found
                        if results_text_parts.is_empty() {
                            results_text_parts.push("No response generated for the query.".to_string());
                        }

                        let final_text = results_text_parts.join("\n");

                        Ok(CallToolResult {
                            is_error: None,
                            content: vec![Content {
                                annotations: None,
                                text: Some(final_text),
                                mime_type: Some("text/plain".to_string()),
                                r#type: ContentType::Text,
                                data: None,
                            }],
                        })
                    }
                    Err(e) => {
                        Ok(CallToolResult {
                            is_error: Some(true),
                            content: vec![Content {
                                annotations: None,
                                text: Some(format!("Failed to parse API response JSON: {}. Body: {}", e, body_str)),
                                mime_type: None,
                                r#type: ContentType::Text,
                                data: None,
                            }],
                        })
                    }
                }
            } else {
                Ok(CallToolResult {
                    is_error: Some(true),
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!("API request failed with status {}: {}", res.status_code(), body_str)),
                        mime_type: None,
                        r#type: ContentType::Text,
                        data: None,
                    }],
                })
            }
        }
        Err(e) => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("HTTP request failed: {}", e)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "perplexity_search".into(),
                description: "Search using Perplexity AI's Sonar models with real-time web search capabilities. Returns AI-generated responses with citations and search results.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query or question",
                        },
                        "api_key": {
                            "type": "string",
                            "description": "Your Perplexity API key",
                        },
                        "model": {
                            "type": "string",
                            "description": "The model to use for search",
                            "enum": ["sonar-pro", "sonar-reasoning", "sonar-deep-research"],
                            "default": "sonar-pro",
                        },
                        "system_message": {
                            "type": "string",
                            "description": "Optional system message to set context",
                        },
                        "max_tokens": {
                            "type": "integer",
                            "description": "Maximum tokens in the response (1-4096)",
                            "minimum": 1,
                            "maximum": 4096,
                            "default": 1000,
                        },
                        "temperature": {
                            "type": "number",
                            "description": "Controls randomness in the response (0.0-2.0)",
                            "minimum": 0.0,
                            "maximum": 2.0,
                            "default": 0.2,
                        },
                        "search_recency_filter": {
                            "type": "string",
                            "description": "Filter search results by recency",
                            "enum": ["month", "week", "day", "hour"],
                            "default": "month",
                        },
                        "return_citations": {
                            "type": "boolean",
                            "description": "Include citations in the response",
                            "default": true,
                        },
                        "return_images": {
                            "type": "boolean",
                            "description": "Include images in search results",
                            "default": false,
                        },
                        "return_related_questions": {
                            "type": "boolean",
                            "description": "Include related questions",
                            "default": false,
                        },
                    },
                    "required": ["query", "api_key"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
