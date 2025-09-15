mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value as JsonValue, json};
use urlencoding::encode;

const GOOGLE_SEARCH_API_BASE_URL: &str = "https://www.googleapis.com/customsearch/v1";

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "google_search" => google_search(input),
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

fn google_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    // Extract required parameters
    let query_val = args.get("query").unwrap_or(&JsonValue::Null);
    let api_key_val = args.get("api_key").unwrap_or(&JsonValue::Null);
    let search_engine_id_val = args.get("search_engine_id").unwrap_or(&JsonValue::Null);

    // Validate required parameters
    let query = match query_val {
        JsonValue::String(s) if !s.is_empty() => s,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(
                        "Missing or invalid required parameter: query (must be non-empty string)"
                            .to_string(),
                    ),
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
                    text: Some(
                        "Missing or invalid required parameter: api_key (must be non-empty string)"
                            .to_string(),
                    ),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let search_engine_id = match search_engine_id_val {
        JsonValue::String(s) if !s.is_empty() => s,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: search_engine_id (must be non-empty string)".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    // Build query parameters
    let mut query_params = vec![
        format!("key={}", api_key),
        format!("cx={}", search_engine_id),
        format!("q={}", encode(query)),
    ];

    // Optional parameters
    if let Some(JsonValue::Number(num)) = args.get("num") {
        if let Some(num_val) = num.as_i64() {
            if num_val >= 1 && num_val <= 10 {
                query_params.push(format!("num={}", num_val));
            }
        }
    }

    if let Some(JsonValue::Number(start)) = args.get("start") {
        if let Some(start_val) = start.as_i64() {
            if start_val >= 1 && start_val <= 91 {
                query_params.push(format!("start={}", start_val));
            }
        }
    }

    if let Some(JsonValue::String(safe)) = args.get("safe") {
        match safe.as_str() {
            "active" | "off" => {
                query_params.push(format!("safe={}", safe));
            }
            _ => {}
        }
    }

    if let Some(JsonValue::String(lr)) = args.get("lr") {
        query_params.push(format!("lr={}", encode(lr)));
    }

    if let Some(JsonValue::String(gl)) = args.get("gl") {
        query_params.push(format!("gl={}", encode(gl)));
    }

    if let Some(JsonValue::String(cr)) = args.get("cr") {
        query_params.push(format!("cr={}", encode(cr)));
    }

    if let Some(JsonValue::String(date_restrict)) = args.get("date_restrict") {
        query_params.push(format!("dateRestrict={}", encode(date_restrict)));
    }

    if let Some(JsonValue::String(site_search)) = args.get("site_search") {
        query_params.push(format!("siteSearch={}", encode(site_search)));
    }

    if let Some(JsonValue::String(search_type)) = args.get("search_type") {
        match search_type.as_str() {
            "image" => {
                query_params.push(format!("searchType={}", search_type));
            }
            _ => {}
        }
    }

    // Build the final URL
    let query_string = query_params.join("&");
    let url = format!("{}?{}", GOOGLE_SEARCH_API_BASE_URL, query_string);

    // Make the HTTP request
    let req = HttpRequest::new(&url).with_method("GET");

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut results_text_parts = Vec::new();

                        // Extract search information
                        if let Some(search_info) = parsed_json.get("searchInformation") {
                            if let Some(total_results) = search_info.get("formattedTotalResults") {
                                if let Some(total_str) = total_results.as_str() {
                                    results_text_parts
                                        .push(format!("Total Results: {}", total_str));
                                }
                            }
                            if let Some(search_time) = search_info.get("formattedSearchTime") {
                                if let Some(time_str) = search_time.as_str() {
                                    results_text_parts
                                        .push(format!("Search Time: {} seconds", time_str));
                                }
                            }
                        }

                        // Extract search results
                        if let Some(items) = parsed_json.get("items") {
                            if let JsonValue::Array(items_array) = items {
                                if items_array.is_empty() {
                                    results_text_parts.push("No search results found.".to_string());
                                } else {
                                    results_text_parts.push("".to_string()); // Empty line
                                    results_text_parts.push("Search Results:".to_string());
                                    results_text_parts.push("".to_string()); // Empty line

                                    for (index, item) in items_array.iter().enumerate() {
                                        let result_num = index + 1;
                                        results_text_parts.push(format!(
                                            "{}. {}",
                                            result_num,
                                            "=".repeat(50)
                                        ));

                                        if let Some(title) = item.get("title") {
                                            if let Some(title_str) = title.as_str() {
                                                results_text_parts
                                                    .push(format!("Title: {}", title_str));
                                            }
                                        }

                                        if let Some(link) = item.get("link") {
                                            if let Some(link_str) = link.as_str() {
                                                results_text_parts
                                                    .push(format!("URL: {}", link_str));
                                            }
                                        }

                                        if let Some(display_link) = item.get("displayLink") {
                                            if let Some(display_str) = display_link.as_str() {
                                                results_text_parts
                                                    .push(format!("Display Link: {}", display_str));
                                            }
                                        }

                                        if let Some(snippet) = item.get("snippet") {
                                            if let Some(snippet_str) = snippet.as_str() {
                                                results_text_parts
                                                    .push(format!("Snippet: {}", snippet_str));
                                            }
                                        }

                                        results_text_parts.push("".to_string()); // Empty line
                                    }
                                }
                            }
                        }

                        // Handle spelling suggestions
                        if let Some(spelling) = parsed_json.get("spelling") {
                            if let Some(corrected) = spelling.get("correctedQuery") {
                                if let Some(corrected_str) = corrected.as_str() {
                                    results_text_parts.push("".to_string());
                                    results_text_parts
                                        .push(format!("Did you mean: {}", corrected_str));
                                }
                            }
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
                    Err(e) => Ok(CallToolResult {
                        is_error: Some(true),
                        content: vec![Content {
                            annotations: None,
                            text: Some(format!(
                                "Failed to parse API response JSON: {}. Body: {}",
                                e, body_str
                            )),
                            mime_type: None,
                            r#type: ContentType::Text,
                            data: None,
                        }],
                    }),
                }
            } else {
                Ok(CallToolResult {
                    is_error: Some(true),
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!(
                            "API request failed with status {}: {}",
                            res.status_code(),
                            body_str
                        )),
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
                name: "google_search".into(),
                description: "Perform a Google Custom Search using the Google Custom Search JSON API. Returns search results with titles, URLs, and snippets.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query string",
                        },
                        "api_key": {
                            "type": "string",
                            "description": "Your Google Custom Search API key",
                        },
                        "search_engine_id": {
                            "type": "string",
                            "description": "Your Custom Search Engine ID (cx parameter)",
                        },
                        "num": {
                            "type": "integer",
                            "description": "Number of search results to return (1-10, default: 10)",
                            "minimum": 1,
                            "maximum": 10,
                        },
                        "start": {
                            "type": "integer",
                            "description": "The index of the first result to return (1-91, default: 1)",
                            "minimum": 1,
                            "maximum": 91,
                        },
                        "safe": {
                            "type": "string",
                            "description": "SafeSearch setting",
                            "enum": ["active", "off"],
                        },
                        "lr": {
                            "type": "string",
                            "description": "Restricts search to documents in a specific language (e.g., 'lang_en')",
                        },
                        "gl": {
                            "type": "string",
                            "description": "Geolocation of end user (two-letter country code)",
                        },
                        "cr": {
                            "type": "string",
                            "description": "Restricts search results to documents from a specific country",
                        },
                        "date_restrict": {
                            "type": "string",
                            "description": "Restricts results to URLs based on date (e.g., 'd7' for past 7 days)",
                        },
                        "site_search": {
                            "type": "string",
                            "description": "Limits search results to a specific site",
                        },
                        "search_type": {
                            "type": "string",
                            "description": "Search type (only 'image' supported for image search)",
                            "enum": ["image"],
                        },
                    },
                    "required": ["query", "api_key", "search_engine_id"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
