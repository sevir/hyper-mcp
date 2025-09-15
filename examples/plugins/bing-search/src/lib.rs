mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value as JsonValue, json};
use urlencoding::encode;

const BING_SEARCH_API_BASE_URL: &str = "https://api.bing.microsoft.com/v7.0/search";

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "bing_search" => bing_search(input),
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

fn bing_search(input: CallToolRequest) -> Result<CallToolResult, Error> {
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
    if let Some(JsonValue::Number(count)) = args.get("count") {
        if let Some(count_val) = count.as_i64() {
            if count_val >= 1 && count_val <= 50 {
                query_params.push(format!("count={}", count_val));
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

    if let Some(JsonValue::String(mkt)) = args.get("mkt") {
        query_params.push(format!("mkt={}", encode(mkt)));
    }

    if let Some(JsonValue::String(safe_search)) = args.get("safe_search") {
        match safe_search.as_str() {
            "Off" | "Moderate" | "Strict" => {
                query_params.push(format!("safeSearch={}", safe_search));
            }
            _ => {}
        }
    }

    if let Some(JsonValue::String(freshness)) = args.get("freshness") {
        match freshness.as_str() {
            "Day" | "Week" | "Month" => {
                query_params.push(format!("freshness={}", freshness));
            }
            _ => {
                // Check for date range format YYYY-MM-DD..YYYY-MM-DD
                if freshness.contains("..") {
                    query_params.push(format!("freshness={}", encode(freshness)));
                }
            }
        }
    }

    if let Some(JsonValue::String(response_filter)) = args.get("response_filter") {
        query_params.push(format!("responseFilter={}", encode(response_filter)));
    }

    if let Some(JsonValue::String(set_lang)) = args.get("set_lang") {
        query_params.push(format!("setLang={}", encode(set_lang)));
    }

    // Build the final URL
    let query_string = query_params.join("&");
    let url = format!("{}?{}", BING_SEARCH_API_BASE_URL, query_string);

    // Make the HTTP request
    let mut req = HttpRequest::new(&url).with_method("GET");
    req.headers
        .insert("Ocp-Apim-Subscription-Key".to_string(), api_key.to_string());

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut results_text_parts = Vec::new();

                        // Extract web results
                        if let Some(web_pages) = parsed_json.get("webPages") {
                            if let Some(total_matches) = web_pages.get("totalEstimatedMatches") {
                                if let Some(total_str) = total_matches.as_str() {
                                    results_text_parts
                                        .push(format!("Total Results: {}", total_str));
                                } else if let Some(total_num) = total_matches.as_i64() {
                                    results_text_parts
                                        .push(format!("Total Results: {}", total_num));
                                }
                            }

                            if let Some(web_search_url) = web_pages.get("webSearchUrl") {
                                if let Some(url_str) = web_search_url.as_str() {
                                    results_text_parts.push(format!("Search URL: {}", url_str));
                                }
                            }

                            if let Some(value) = web_pages.get("value") {
                                if let JsonValue::Array(results_array) = value {
                                    if results_array.is_empty() {
                                        results_text_parts
                                            .push("No search results found.".to_string());
                                    } else {
                                        results_text_parts.push("".to_string()); // Empty line
                                        results_text_parts.push("Web Search Results:".to_string());
                                        results_text_parts.push("".to_string()); // Empty line

                                        for (index, result) in results_array.iter().enumerate() {
                                            let result_num = index + 1;
                                            results_text_parts.push(format!(
                                                "{}. {}",
                                                result_num,
                                                "=".repeat(50)
                                            ));

                                            if let Some(name) = result.get("name") {
                                                if let Some(name_str) = name.as_str() {
                                                    results_text_parts
                                                        .push(format!("Title: {}", name_str));
                                                }
                                            }

                                            if let Some(url) = result.get("url") {
                                                if let Some(url_str) = url.as_str() {
                                                    results_text_parts
                                                        .push(format!("URL: {}", url_str));
                                                }
                                            }

                                            if let Some(display_url) = result.get("displayUrl") {
                                                if let Some(display_str) = display_url.as_str() {
                                                    results_text_parts.push(format!(
                                                        "Display URL: {}",
                                                        display_str
                                                    ));
                                                }
                                            }

                                            if let Some(snippet) = result.get("snippet") {
                                                if let Some(snippet_str) = snippet.as_str() {
                                                    results_text_parts
                                                        .push(format!("Snippet: {}", snippet_str));
                                                }
                                            }

                                            if let Some(date_last_crawled) =
                                                result.get("dateLastCrawled")
                                            {
                                                if let Some(date_str) = date_last_crawled.as_str() {
                                                    results_text_parts.push(format!(
                                                        "Last Crawled: {}",
                                                        date_str
                                                    ));
                                                }
                                            }

                                            results_text_parts.push("".to_string()); // Empty line
                                        }
                                    }
                                }
                            }
                        }

                        // Handle spelling suggestions
                        if let Some(spell_suggestions) = parsed_json.get("spellSuggestions") {
                            if let Some(value) = spell_suggestions.get("value") {
                                if let JsonValue::Array(suggestions_array) = value {
                                    if !suggestions_array.is_empty() {
                                        if let Some(first_suggestion) = suggestions_array.first() {
                                            if let Some(text) = first_suggestion.get("text") {
                                                if let Some(text_str) = text.as_str() {
                                                    results_text_parts.push("".to_string());
                                                    results_text_parts.push(format!(
                                                        "Did you mean: {}",
                                                        text_str
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Handle related searches
                        if let Some(related_searches) = parsed_json.get("relatedSearches") {
                            if let Some(value) = related_searches.get("value") {
                                if let JsonValue::Array(related_array) = value {
                                    if !related_array.is_empty() {
                                        results_text_parts.push("".to_string());
                                        results_text_parts.push("Related Searches:".to_string());
                                        for (index, related) in related_array.iter().enumerate() {
                                            if let Some(text) = related.get("text") {
                                                if let Some(text_str) = text.as_str() {
                                                    results_text_parts.push(format!(
                                                        "{}. {}",
                                                        index + 1,
                                                        text_str
                                                    ));
                                                }
                                            }
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
                name: "bing_search".into(),
                description: "Perform a Bing web search using the Bing Search API. Returns search results with titles, URLs, snippets, and metadata.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query string",
                        },
                        "api_key": {
                            "type": "string",
                            "description": "Your Bing Search API subscription key",
                        },
                        "count": {
                            "type": "integer",
                            "description": "Number of search results to return (1-50, default: 10)",
                            "minimum": 1,
                            "maximum": 50,
                        },
                        "offset": {
                            "type": "integer",
                            "description": "Number of search results to skip (default: 0)",
                            "minimum": 0,
                        },
                        "mkt": {
                            "type": "string",
                            "description": "Market code for the search (e.g., 'en-US', 'en-GB')",
                        },
                        "safe_search": {
                            "type": "string",
                            "description": "SafeSearch setting",
                            "enum": ["Off", "Moderate", "Strict"],
                        },
                        "freshness": {
                            "type": "string",
                            "description": "Filter by date (Day, Week, Month) or date range (YYYY-MM-DD..YYYY-MM-DD)",
                        },
                        "response_filter": {
                            "type": "string",
                            "description": "Comma-separated list of response types to include (Webpages, Images, News, etc.)",
                        },
                        "set_lang": {
                            "type": "string",
                            "description": "Language for user interface strings (e.g., 'en', 'es')",
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
