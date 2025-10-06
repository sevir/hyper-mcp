mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, InputSchema, ListToolsResult,
    PropertySchema, ToolDescription,
};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;

fn get_logseq_config() -> Result<(String, String), Error> {
    let base_url =
        config::get("LOGSEQ_BASE_URL")?.unwrap_or_else(|| "http://localhost:12315".to_string());
    let api_key = config::get("LOGSEQ_API_KEY")?
        .ok_or_else(|| Error::msg("LOGSEQ_API_KEY configuration is required but not set"))?;

    Ok((base_url, api_key))
}

fn make_logseq_query(method: &str, args: Vec<JsonValue>) -> Result<JsonValue, Error> {
    let (base_url, api_key) = get_logseq_config()?;

    let body = json!({
        "method": method,
        "args": args
    });

    let body_bytes = body.to_string().into_bytes();

    let req = HttpRequest::new(&format!("{}/api", base_url))
        .with_method("POST")
        .with_header("Content-Type", "application/json")
        .with_header("Authorization", &format!("Bearer {}", api_key));

    let response = http::request::<Vec<u8>>(&req, Some(body_bytes))?;

    if response.status_code() != 200 {
        return Err(Error::msg(format!(
            "Logseq API error: HTTP {}",
            response.status_code()
        )));
    }

    let body_str = String::from_utf8(response.body().to_vec())
        .map_err(|e| Error::msg(format!("Invalid UTF-8 response: {}", e)))?;

    serde_json::from_str(&body_str)
        .map_err(|e| Error::msg(format!("Failed to parse JSON response: {}", e)))
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    let mut tools = Vec::new();

    // Search pages tool
    let mut search_properties = HashMap::new();
    search_properties.insert(
        "query".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "Search query for finding pages by name or content".to_string(),
            default: None,
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "search_pages".to_string(),
        description: "Search for pages in Logseq by name or content. Returns a list of matching pages with their IDs and names.".to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: search_properties,
            required: Some(vec!["query".to_string()]),
        },
    });

    // Get page blocks tool
    let mut page_blocks_properties = HashMap::new();
    page_blocks_properties.insert(
        "page_name".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The name of the page to get blocks from".to_string(),
            default: None,
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "get_page_blocks".to_string(),
        description: "Get all blocks from a specific page in Logseq. Returns the hierarchical structure of blocks with their content.".to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: page_blocks_properties,
            required: Some(vec!["page_name".to_string()]),
        },
    });

    // Get block with children tool
    let mut block_properties = HashMap::new();
    block_properties.insert(
        "block_id".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The ID or UUID of the block to retrieve".to_string(),
            default: None,
            items: None,
        },
    );
    block_properties.insert(
        "include_children".to_string(),
        PropertySchema {
            property_type: "boolean".to_string(),
            description: "Whether to include child blocks (default: true)".to_string(),
            default: Some(json!(true)),
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "get_block".to_string(),
        description: "Get a specific block by ID with optional children. Returns the block content and its nested children if requested.".to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: block_properties,
            required: Some(vec!["block_id".to_string()]),
        },
    });

    // Get references tool
    let mut refs_properties = HashMap::new();
    refs_properties.insert(
        "page_name".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The name of the page or tag to find references for".to_string(),
            default: None,
            items: None,
        },
    );
    refs_properties.insert(
        "max_depth".to_string(),
        PropertySchema {
            property_type: "integer".to_string(),
            description:
                "Maximum depth for recursive reference retrieval (default: 1, 0 for unlimited)"
                    .to_string(),
            default: Some(json!(1)),
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "get_page_references".to_string(),
        description: "Get all blocks that reference a specific page or tag, with optional recursive depth to follow references and get full content tree. Useful for exploring connected notes and building knowledge graphs.".to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: refs_properties,
            required: Some(vec!["page_name".to_string()]),
        },
    });

    // Get page content tool
    let mut content_properties = HashMap::new();
    content_properties.insert(
        "page_name".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The name of the page to retrieve content from".to_string(),
            default: None,
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "get_page_content".to_string(),
        description: "Get the complete content of a page including all its blocks. Returns structured markdown content.".to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: content_properties,
            required: Some(vec!["page_name".to_string()]),
        },
    });

    Ok(ListToolsResult { tools })
}

fn format_block_hierarchy(block: &JsonValue, level: usize) -> String {
    let indent = "  ".repeat(level);
    let content = block.get("content").and_then(|v| v.as_str()).unwrap_or("");

    let mut result = format!("{}• {}\n", indent, content);

    if let Some(children) = block.get("children").and_then(|v| v.as_array()) {
        for child in children {
            result.push_str(&format_block_hierarchy(child, level + 1));
        }
    }

    result
}

fn search_pages(query: &str) -> Result<String, Error> {
    // Use datascript query to search pages
    let datascript_query = format!(
        "[:find (pull ?p [*]) :where [?p :block/name ?name] [(re-find #\"(?i){}\" ?name)]]",
        query.replace("\"", "\\\"")
    );

    let result = make_logseq_query("logseq.DB.datascriptQuery", vec![json!(datascript_query)])?;

    let mut output = format!("# Search Results for \"{}\"\n\n", query);

    if let Some(results) = result.as_array() {
        if results.is_empty() {
            output.push_str("No pages found matching the query.\n");
        } else {
            output.push_str(&format!("Found {} page(s):\n\n", results.len()));
            for (idx, item) in results.iter().enumerate() {
                if let Some(page) = item.get(0) {
                    let name = page
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown");
                    let id = page
                        .get("id")
                        .and_then(|v| v.as_i64())
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "N/A".to_string());

                    output.push_str(&format!("{}. **{}** (ID: {})\n", idx + 1, name, id));
                }
            }
        }
    }

    Ok(output)
}

fn get_page_blocks(page_name: &str) -> Result<String, Error> {
    // First get the page
    let page_result = make_logseq_query("logseq.Editor.getPage", vec![json!(page_name)])?;

    if page_result.is_null() {
        return Ok(format!("Page \"{}\" not found.\n", page_name));
    }

    // Get all blocks for the page
    let datascript_query = format!(
        "[:find (pull ?b [*]) :where [?b :block/page ?p] [?p :block/name \"{}\"]]",
        page_name.to_lowercase().replace("\"", "\\\"")
    );

    let blocks_result =
        make_logseq_query("logseq.DB.datascriptQuery", vec![json!(datascript_query)])?;

    let mut output = format!("# Blocks for page: {}\n\n", page_name);

    if let Some(results) = blocks_result.as_array() {
        if results.is_empty() {
            output.push_str("No blocks found in this page.\n");
        } else {
            output.push_str(&format!("Total blocks: {}\n\n", results.len()));
            for item in results {
                if let Some(block) = item.get(0) {
                    output.push_str(&format_block_hierarchy(block, 0));
                }
            }
        }
    }

    Ok(output)
}

fn get_block(block_id: &str, include_children: bool) -> Result<String, Error> {
    let args = if include_children {
        vec![json!(block_id), json!({"includeChildren": true})]
    } else {
        vec![json!(block_id)]
    };

    let block_result = make_logseq_query("logseq.Editor.getBlock", args)?;

    if block_result.is_null() {
        return Ok(format!("Block \"{}\" not found.\n", block_id));
    }

    let mut output = format!("# Block: {}\n\n", block_id);
    output.push_str(&format_block_hierarchy(&block_result, 0));

    Ok(output)
}

fn get_page_references(page_name: &str, max_depth: i32) -> Result<String, Error> {
    // Query for blocks that reference the page
    let datascript_query = format!(
        "[:find (pull ?b [*]) :where [?b :block/refs ?p] [?p :block/name \"{}\"]]",
        page_name.to_lowercase().replace("\"", "\\\"")
    );

    let refs_result =
        make_logseq_query("logseq.DB.datascriptQuery", vec![json!(datascript_query)])?;

    let mut output = format!("# References to: {}\n\n", page_name);

    if let Some(results) = refs_result.as_array() {
        if results.is_empty() {
            output.push_str("No references found for this page.\n");
        } else {
            output.push_str(&format!("Found {} reference(s):\n\n", results.len()));
            for (idx, item) in results.iter().enumerate() {
                if let Some(block) = item.get(0) {
                    output.push_str(&format!("## Reference {}\n\n", idx + 1));

                    // Get page info
                    if let Some(page) = block.get("page") {
                        let page_name = page
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown");
                        output.push_str(&format!("**From page:** {}\n\n", page_name));
                    }

                    // If max_depth > 0, get full block with children
                    if max_depth > 0 {
                        if let Some(block_id) = block.get("id").and_then(|v| v.as_str()) {
                            match get_block(block_id, true) {
                                Ok(block_content) => output.push_str(&block_content),
                                Err(_) => output.push_str(&format_block_hierarchy(block, 0)),
                            }
                        } else {
                            output.push_str(&format_block_hierarchy(block, 0));
                        }
                    } else {
                        output.push_str(&format_block_hierarchy(block, 0));
                    }

                    output.push_str("\n---\n\n");
                }
            }
        }
    }

    Ok(output)
}

fn get_page_content(page_name: &str) -> Result<String, Error> {
    // Get the page
    let page_result = make_logseq_query("logseq.Editor.getPage", vec![json!(page_name)])?;

    if page_result.is_null() {
        return Ok(format!("Page \"{}\" not found.\n", page_name));
    }

    let mut output = format!("# Page: {}\n\n", page_name);

    // Add page metadata
    if let Some(uuid) = page_result.get("uuid").and_then(|v| v.as_str()) {
        output.push_str(&format!("**UUID:** {}\n", uuid));
    }
    if let Some(journal) = page_result.get("journal").and_then(|v| v.as_bool()) {
        if journal {
            output.push_str("**Type:** Journal page\n");
        }
    }
    output.push_str("\n");

    // Get all blocks
    match get_page_blocks(page_name) {
        Ok(blocks_content) => output.push_str(&blocks_content),
        Err(e) => output.push_str(&format!("Error retrieving blocks: {}\n", e)),
    }

    Ok(output)
}

pub(crate) fn call(request: CallToolRequest) -> Result<CallToolResult, Error> {
    let tool_name = &request.params.name;
    let args = request.params.arguments.as_ref();

    let result_text = match tool_name.as_str() {
        "search_pages" => {
            let query = args
                .and_then(|a| a.get("query"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::msg("query parameter is required"))?;

            search_pages(query)?
        }
        "get_page_blocks" => {
            let page_name = args
                .and_then(|a| a.get("page_name"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::msg("page_name parameter is required"))?;

            get_page_blocks(page_name)?
        }
        "get_block" => {
            let block_id = args
                .and_then(|a| a.get("block_id"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::msg("block_id parameter is required"))?;

            let include_children = args
                .and_then(|a| a.get("include_children"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            get_block(block_id, include_children)?
        }
        "get_page_references" => {
            let page_name = args
                .and_then(|a| a.get("page_name"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::msg("page_name parameter is required"))?;

            let max_depth = args
                .and_then(|a| a.get("max_depth"))
                .and_then(|v| v.as_i64())
                .unwrap_or(1) as i32;

            get_page_references(page_name, max_depth)?
        }
        "get_page_content" => {
            let page_name = args
                .and_then(|a| a.get("page_name"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::msg("page_name parameter is required"))?;

            get_page_content(page_name)?
        }
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Unknown tool: {}", tool_name)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result_text),
            mime_type: Some("text/markdown".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}
