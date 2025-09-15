mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value as JsonValue, json};
use urlencoding::encode;

const BRAVE_SEARCH_API_BASE_URL: &str = "https://api.search.brave.com/res/v1/web/search";

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "brave_search" => brave_search(input),
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

fn brave_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
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

    // Build query parameters
    let mut query_params = vec![format!("q={}", encode(query))];

    // Optional parameters
    if let Some(JsonValue::Number(num)) = args.get("count") {
        if let Some(num_val) = num.as_i64() {
            if num_val >= 1 && num_val <= 20 {
                query_params.push(format!("count={}", num_val));
            }
        }
    }

    if let Some(JsonValue::Number(offset)) = args.get("offset") {
        if let Some(offset_val) = offset.as_i64() {
            if offset_val >= 0 {
                query_params.push(format!("offset={}", offset_val));
            }
        }
    }

    if let Some(JsonValue::String(country)) = args.get("country") {
        if !country.is_empty() {
            query_params.push(format!("country={}", encode(country)));
        }
    }

    if let Some(JsonValue::String(search_lang)) = args.get("search_lang") {
        if !search_lang.is_empty() {
            query_params.push(format!("search_lang={}", encode(search_lang)));
        }
    }

    if let Some(JsonValue::String(ui_lang)) = args.get("ui_lang") {
        if !ui_lang.is_empty() {
            query_params.push(format!("ui_lang={}", encode(ui_lang)));
        }
    }

    if let Some(JsonValue::String(safesearch)) = args.get("safesearch") {
        match safesearch.as_str() {
            "strict" | "moderate" | "off" => {
                query_params.push(format!("safesearch={}", safesearch));
            }
            _ => {}
        }
    }

    if let Some(JsonValue::String(freshness)) = args.get("freshness") {
        match freshness.as_str() {
            "pd" | "pw" | "pm" | "py" => {
                query_params.push(format!("freshness={}", freshness));
            }
            _ => {}
        }
    }

    if let Some(JsonValue::String(result_filter)) = args.get("result_filter") {
        if !result_filter.is_empty() {
            query_params.push(format!("result_filter={}", encode(result_filter)));
        }
    }

    // Build the final URL
    let query_string = query_params.join("&");
    let url = format!("{}?{}", BRAVE_SEARCH_API_BASE_URL, query_string);

    // Make the HTTP request
    let req = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("X-Subscription-Token", api_key);

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut results_text_parts = Vec::new();

                        // Extract query information
                        if let Some(query_info) = parsed_json.get("query") {
                            if let Some(original) = query_info.get("original") {
                                if let Some(original_str) = original.as_str() {
                                    results_text_parts.push(format!("Query: {}", original_str));
                                }
                            }
                        }

                        // Extract web search results
                        if let Some(web) = parsed_json.get("web") {
                            if let Some(results) = web.get("results") {
                                if let JsonValue::Array(results_array) = results {
                                    if results_array.is_empty() {
                                        results_text_parts
                                            .push("No search results found.".to_string());
                                    } else {
                                        results_text_parts.push("".to_string()); // Empty line
                                        results_text_parts.push("Search Results:".to_string());
                                        results_text_parts.push("".to_string()); // Empty line

                                        for (index, item) in results_array.iter().enumerate() {
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

                                            if let Some(url) = item.get("url") {
                                                if let Some(url_str) = url.as_str() {
                                                    results_text_parts
                                                        .push(format!("URL: {}", url_str));
                                                }
                                            }

                                            if let Some(description) = item.get("description") {
                                                if let Some(desc_str) = description.as_str() {
                                                    results_text_parts
                                                        .push(format!("Description: {}", desc_str));
                                                }
                                            }

                                            if let Some(page_age) = item.get("page_age") {
                                                if let Some(age_str) = page_age.as_str() {
                                                    results_text_parts
                                                        .push(format!("Page Age: {}", age_str));
                                                }
                                            }

                                            results_text_parts.push("".to_string()); // Empty line
                                        }
                                    }
                                }
                            }
                        }

                        // Handle discussions if available
                        if let Some(discussions) = parsed_json.get("discussions") {
                            if let Some(disc_results) = discussions.get("results") {
                                if let JsonValue::Array(disc_array) = disc_results {
                                    if !disc_array.is_empty() {
                                        results_text_parts.push("".to_string());
                                        results_text_parts.push("Discussion Results:".to_string());
                                        results_text_parts.push("".to_string());

                                        for (index, item) in disc_array.iter().take(3).enumerate() {
                                            let result_num = index + 1;
                                            results_text_parts.push(format!(
                                                "Discussion {}. {}",
                                                result_num,
                                                "-".repeat(30)
                                            ));

                                            if let Some(title) = item.get("title") {
                                                if let Some(title_str) = title.as_str() {
                                                    results_text_parts
                                                        .push(format!("Title: {}", title_str));
                                                }
                                            }

                                            if let Some(url) = item.get("url") {
                                                if let Some(url_str) = url.as_str() {
                                                    results_text_parts
                                                        .push(format!("URL: {}", url_str));
                                                }
                                            }

                                            results_text_parts.push("".to_string());
                                        }
                                    }
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
                name: "brave_search".into(),
                description: "Perform a web search using the Brave Search API. Returns search results with titles, URLs, descriptions, and optional discussions.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query string",
                        },
                        "api_key": {
                            "type": "string",
                            "description": "Your Brave Search API subscription token",
                        },
                        "count": {
                            "type": "integer",
                            "description": "Number of search results to return (1-20, default: 10)",
                            "minimum": 1,
                            "maximum": 20,
                        },
                        "offset": {
                            "type": "integer",
                            "description": "The offset for pagination (default: 0)",
                            "minimum": 0,
                        },
                        "country": {
                            "type": "string",
                            "description": "Country code for search results (e.g., 'US', 'GB', 'DE')",
                        },
                        "search_lang": {
                            "type": "string",
                            "description": "Language code for search query (e.g., 'en', 'es', 'fr')",
                        },
                        "ui_lang": {
                            "type": "string",
                            "description": "Language code for user interface (e.g., 'en', 'es', 'fr')",
                        },
                        "safesearch": {
                            "type": "string",
                            "description": "SafeSearch setting",
                            "enum": ["strict", "moderate", "off"],
                        },
                        "freshness": {
                            "type": "string",
                            "description": "Time-based freshness filter",
                            "enum": ["pd", "pw", "pm", "py"],
                        },
                        "result_filter": {
                            "type": "string",
                            "description": "Filter to return only specific result types (comma-separated)",
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
