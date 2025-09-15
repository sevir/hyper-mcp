mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value as JsonValue, json};

const DUCKDUCKGO_API_BASE_URL: &str = "https://api.duckduckgo.com/";

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "duckduckgo_search" => duckduckgo_search(input),
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

fn duckduckgo_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    // Extract required parameters
    let query_val = args.get("query").unwrap_or(&JsonValue::Null);

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

    // Build query parameters
    let mut query_params = vec![
        format!("q={}", urlencoding::encode(query)),
        "format=json".to_string(),
    ];

    // Optional parameters
    if let Some(JsonValue::String(format_param)) = args.get("format") {
        match format_param.as_str() {
            "json" | "xml" => {
                query_params[1] = format!("format={}", format_param);
            }
            _ => {}
        }
    }

    if let Some(JsonValue::Bool(pretty)) = args.get("pretty") {
        if *pretty {
            query_params.push("pretty=1".to_string());
        }
    }

    if let Some(JsonValue::Bool(no_html)) = args.get("no_html") {
        if *no_html {
            query_params.push("no_html=1".to_string());
        }
    }

    if let Some(JsonValue::Bool(no_redirect)) = args.get("no_redirect") {
        if *no_redirect {
            query_params.push("no_redirect=1".to_string());
        }
    }

    if let Some(JsonValue::Bool(skip_disambig)) = args.get("skip_disambig") {
        if *skip_disambig {
            query_params.push("skip_disambig=1".to_string());
        }
    }

    // Build the final URL
    let query_string = query_params.join("&");
    let url = format!("{}?{}", DUCKDUCKGO_API_BASE_URL, query_string);

    // Make the HTTP request
    let req = HttpRequest::new(&url).with_method("GET");

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut results_text_parts = Vec::new();

                        // Extract heading
                        if let Some(heading) = parsed_json.get("Heading") {
                            if let Some(heading_str) = heading.as_str() {
                                results_text_parts.push(format!("Query: {}", query));
                                results_text_parts.push(format!("Heading: {}", heading_str));
                                results_text_parts.push("".to_string());
                            }
                        }

                        // Extract abstract/instant answer
                        if let Some(abstract_text) = parsed_json.get("AbstractText") {
                            if let Some(abstract_str) = abstract_text.as_str() {
                                if !abstract_str.is_empty() {
                                    results_text_parts.push("Instant Answer:".to_string());
                                    results_text_parts.push(abstract_str.to_string());
                                    results_text_parts.push("".to_string());

                                    if let Some(abstract_source) = parsed_json.get("AbstractSource") {
                                        if let Some(source_str) = abstract_source.as_str() {
                                            results_text_parts.push(format!("Source: {}", source_str));
                                        }
                                    }

                                    if let Some(abstract_url) = parsed_json.get("AbstractURL") {
                                        if let Some(url_str) = abstract_url.as_str() {
                                            results_text_parts.push(format!("URL: {}", url_str));
                                        }
                                    }
                                    results_text_parts.push("".to_string());
                                }
                            }
                        }

                        // Extract direct answer
                        if let Some(answer) = parsed_json.get("Answer") {
                            if let Some(answer_str) = answer.as_str() {
                                if !answer_str.is_empty() {
                                    results_text_parts.push("Answer:".to_string());
                                    results_text_parts.push(answer_str.to_string());
                                    results_text_parts.push("".to_string());
                                }
                            }
                        }

                        // Extract definition
                        if let Some(definition) = parsed_json.get("Definition") {
                            if let Some(def_str) = definition.as_str() {
                                if !def_str.is_empty() {
                                    results_text_parts.push("Definition:".to_string());
                                    results_text_parts.push(def_str.to_string());

                                    if let Some(def_source) = parsed_json.get("DefinitionSource") {
                                        if let Some(source_str) = def_source.as_str() {
                                            results_text_parts.push(format!("Definition Source: {}", source_str));
                                        }
                                    }

                                    if let Some(def_url) = parsed_json.get("DefinitionURL") {
                                        if let Some(url_str) = def_url.as_str() {
                                            results_text_parts.push(format!("Definition URL: {}", url_str));
                                        }
                                    }
                                    results_text_parts.push("".to_string());
                                }
                            }
                        }

                        // Extract related topics
                        if let Some(related_topics) = parsed_json.get("RelatedTopics") {
                            if let JsonValue::Array(topics_array) = related_topics {
                                if !topics_array.is_empty() {
                                    results_text_parts.push("Related Topics:".to_string());
                                    results_text_parts.push("".to_string());

                                    for (index, topic) in topics_array.iter().enumerate() {
                                        let topic_num = index + 1;

                                        // Handle direct topic objects
                                        if let Some(text) = topic.get("Text") {
                                            if let Some(text_str) = text.as_str() {
                                                results_text_parts.push(format!("{}. {}", topic_num, text_str));

                                                if let Some(first_url) = topic.get("FirstURL") {
                                                    if let Some(url_str) = first_url.as_str() {
                                                        results_text_parts.push(format!("   URL: https://duckduckgo.com{}", url_str));
                                                    }
                                                }
                                                results_text_parts.push("".to_string());
                                            }
                                        }

                                        // Handle topic groups with nested topics
                                        if let Some(name) = topic.get("Name") {
                                            if let Some(name_str) = name.as_str() {
                                                results_text_parts.push(format!("Category: {}", name_str));
                                            }
                                        }

                                        if let Some(topics) = topic.get("Topics") {
                                            if let JsonValue::Array(nested_topics) = topics {
                                                for (nested_index, nested_topic) in nested_topics.iter().enumerate() {
                                                    if let Some(text) = nested_topic.get("Text") {
                                                        if let Some(text_str) = text.as_str() {
                                                            results_text_parts.push(format!("   {}. {}", nested_index + 1, text_str));

                                                            if let Some(first_url) = nested_topic.get("FirstURL") {
                                                                if let Some(url_str) = first_url.as_str() {
                                                                    results_text_parts.push(format!("      URL: https://duckduckgo.com{}", url_str));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        results_text_parts.push("".to_string());
                                    }
                                }
                            }
                        }

                        // Extract results array (traditional search results)
                        if let Some(results) = parsed_json.get("Results") {
                            if let JsonValue::Array(results_array) = results {
                                if !results_array.is_empty() {
                                    results_text_parts.push("Search Results:".to_string());
                                    results_text_parts.push("".to_string());

                                    for (index, result) in results_array.iter().enumerate() {
                                        let result_num = index + 1;
                                        results_text_parts.push(format!("{}.", result_num));

                                        if let Some(text) = result.get("Text") {
                                            if let Some(text_str) = text.as_str() {
                                                results_text_parts.push(format!("   Text: {}", text_str));
                                            }
                                        }

                                        if let Some(first_url) = result.get("FirstURL") {
                                            if let Some(url_str) = first_url.as_str() {
                                                results_text_parts.push(format!("   URL: https://duckduckgo.com{}", url_str));
                                            }
                                        }
                                        results_text_parts.push("".to_string());
                                    }
                                }
                            }
                        }

                        // Handle case where no results found
                        if results_text_parts.is_empty() {
                            results_text_parts.push("No results found for the query.".to_string());
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
                name: "duckduckgo_search".into(),
                description: "Search using DuckDuckGo's Instant Answer API. Returns instant answers, definitions, and related topics for queries.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query string",
                        },
                        "format": {
                            "type": "string",
                            "description": "Response format",
                            "enum": ["json", "xml"],
                            "default": "json",
                        },
                        "pretty": {
                            "type": "boolean",
                            "description": "Pretty-print the JSON response",
                            "default": false,
                        },
                        "no_html": {
                            "type": "boolean",
                            "description": "Remove HTML from text",
                            "default": false,
                        },
                        "no_redirect": {
                            "type": "boolean",
                            "description": "Do not follow redirects",
                            "default": false,
                        },
                        "skip_disambig": {
                            "type": "boolean",
                            "description": "Skip disambiguation",
                            "default": false,
                        },
                    },
                    "required": ["query"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
