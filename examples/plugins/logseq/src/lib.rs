mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, InputSchema, ListToolsResult,
    PropertySchema, ToolDescription,
};
use serde_json::{Value as JsonValue, json};
use std::collections::{HashMap, HashSet};

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

    // Explore page tool (AI-driven knowledge graph traversal)
    let mut explore_properties = HashMap::new();
    explore_properties.insert(
        "query".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "Page name or search term. The plugin resolves it to the best matching page, then reads it and the pages it references.".to_string(),
            default: None,
            items: None,
        },
    );
    explore_properties.insert(
        "depth".to_string(),
        PropertySchema {
            property_type: "integer".to_string(),
            description: "How many reference hops to follow from the starting page (default: 1, recommended max: 2).".to_string(),
            default: Some(json!(1)),
            items: None,
        },
    );
    explore_properties.insert(
        "max_pages".to_string(),
        PropertySchema {
            property_type: "integer".to_string(),
            description: "Maximum number of pages to read in total, to keep the result bounded (default: 10).".to_string(),
            default: Some(json!(10)),
            items: None,
        },
    );
    explore_properties.insert(
        "include_backlinks".to_string(),
        PropertySchema {
            property_type: "boolean".to_string(),
            description: "Also follow pages that reference the starting page, not only outgoing references (default: false).".to_string(),
            default: Some(json!(false)),
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "explore_page".to_string(),
        description: "AI-friendly knowledge exploration: search for a page, read its full content, extract the pages it references, and recursively read those related pages up to a given depth. Returns one consolidated markdown document for navigating connected notes. Ideal for using Logseq notes as a navigable knowledge source.".to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: explore_properties,
            required: Some(vec!["query".to_string()]),
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
    let mut output = format!("# Blocks for page: {}\n\n", page_name);
    output.push_str(&render_page_blocks(page_name)?);
    Ok(output)
}

/// Render a page's blocks as an indented markdown tree using the canonical
/// `getPageBlocksTree` API, which preserves the real parent/child hierarchy
/// (the flat datascript query does not populate nested children).
fn render_page_blocks(page_name: &str) -> Result<String, Error> {
    let tree = make_logseq_query("logseq.Editor.getPageBlocksTree", vec![json!(page_name)])?;

    let mut output = String::new();
    match tree.as_array() {
        Some(blocks) if !blocks.is_empty() => {
            for block in blocks {
                output.push_str(&format_block_hierarchy(block, 0));
            }
        }
        _ => output.push_str("No blocks found in this page.\n"),
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

/// Best display name from a `getPage` result (camelCase keys like
/// `originalName`) OR a datascript pull result (kebab keys like
/// `original-name`).
fn page_display_name(value: &JsonValue) -> Option<String> {
    for key in ["originalName", "original-name", "name"] {
        if let Some(s) = value.get(key).and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }
    }
    None
}

/// Resolve a free-text query to a real page name: exact lookup first, then a
/// case-insensitive name search picking the closest match.
fn resolve_page_name(query: &str) -> Result<Option<String>, Error> {
    let direct = make_logseq_query("logseq.Editor.getPage", vec![json!(query)])?;
    if !direct.is_null() {
        if let Some(name) = page_display_name(&direct) {
            return Ok(Some(name));
        }
    }

    let datascript_query = format!(
        "[:find (pull ?p [:block/name :block/original-name]) :where [?p :block/name ?name] [(re-find #\"(?i){}\" ?name)]]",
        query.replace("\"", "\\\"")
    );
    let result = make_logseq_query("logseq.DB.datascriptQuery", vec![json!(datascript_query)])?;

    let mut best: Option<String> = None;
    if let Some(results) = result.as_array() {
        let lower_query = query.to_lowercase();
        for item in results {
            if let Some(name) = item.get(0).and_then(page_display_name) {
                if name.to_lowercase() == lower_query {
                    return Ok(Some(name));
                }
                if best.is_none() {
                    best = Some(name);
                }
            }
        }
    }

    Ok(best)
}

/// Deduplicated page names from a datascript pull result, excluding the source.
fn collect_unique_page_names(result: &JsonValue, exclude: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let exclude_lower = exclude.to_lowercase();
    if let Some(results) = result.as_array() {
        for item in results {
            if let Some(name) = item.get(0).and_then(page_display_name) {
                let lower = name.to_lowercase();
                if lower != exclude_lower && !names.iter().any(|n| n.to_lowercase() == lower) {
                    names.push(name);
                }
            }
        }
    }
    names
}

/// Pages referenced by the blocks of `page_name` (outgoing `[[links]]` / `#tags`).
fn get_outgoing_page_refs(page_name: &str) -> Result<Vec<String>, Error> {
    let datascript_query = format!(
        "[:find (pull ?ref [:block/name :block/original-name]) :where [?p :block/name \"{}\"] [?b :block/page ?p] [?b :block/refs ?ref] [?ref :block/name]]",
        page_name.to_lowercase().replace("\"", "\\\"")
    );
    let result = make_logseq_query("logseq.DB.datascriptQuery", vec![json!(datascript_query)])?;
    Ok(collect_unique_page_names(&result, page_name))
}

/// Pages that reference `page_name` (incoming backlinks).
fn get_backlink_pages(page_name: &str) -> Result<Vec<String>, Error> {
    let datascript_query = format!(
        "[:find (pull ?bp [:block/name :block/original-name]) :where [?p :block/name \"{}\"] [?b :block/refs ?p] [?b :block/page ?bp]]",
        page_name.to_lowercase().replace("\"", "\\\"")
    );
    let result = make_logseq_query("logseq.DB.datascriptQuery", vec![json!(datascript_query)])?;
    Ok(collect_unique_page_names(&result, page_name))
}

/// AI-friendly knowledge exploration: starting from a search term, resolve the
/// best matching page, read its content, extract the pages it references, and
/// recursively read those related pages up to `depth` hops, bounded by
/// `max_pages`. A `visited` set prevents cycles and re-reading pages.
fn explore_page(
    query: &str,
    depth: usize,
    max_pages: usize,
    include_backlinks: bool,
) -> Result<String, Error> {
    let root = match resolve_page_name(query)? {
        Some(name) => name,
        None => return Ok(format!("No page found matching \"{}\".\n", query)),
    };

    let mut output = format!("# Knowledge exploration: {}\n\n", root);
    output.push_str(&format!(
        "_From page **{}**, following references up to depth {} (max {} pages, backlinks: {})._\n\n",
        root, depth, max_pages, include_backlinks
    ));

    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(root.to_lowercase());
    let mut current = vec![root];
    let mut rendered = 0usize;

    for level in 0..=depth {
        if current.is_empty() || rendered >= max_pages {
            break;
        }

        let mut next = Vec::new();
        for page in &current {
            if rendered >= max_pages {
                break;
            }

            if level == 0 {
                output.push_str("## Main page\n\n");
            } else {
                output.push_str(&format!("## Related page (depth {}): {}\n\n", level, page));
            }

            output.push_str(&render_page_blocks(page)?);
            output.push('\n');
            rendered += 1;

            let outgoing = get_outgoing_page_refs(page).unwrap_or_default();
            let backlinks = if include_backlinks {
                get_backlink_pages(page).unwrap_or_default()
            } else {
                Vec::new()
            };

            if !outgoing.is_empty() {
                output.push_str(&format!(
                    "**References ({}):** {}\n\n",
                    outgoing.len(),
                    outgoing.join(", ")
                ));
            }
            if !backlinks.is_empty() {
                output.push_str(&format!(
                    "**Backlinks ({}):** {}\n\n",
                    backlinks.len(),
                    backlinks.join(", ")
                ));
            }
            output.push_str("---\n\n");

            for name in outgoing.into_iter().chain(backlinks.into_iter()) {
                if visited.insert(name.to_lowercase()) {
                    next.push(name);
                }
            }
        }
        current = next;
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
        "explore_page" => {
            let query = args
                .and_then(|a| a.get("query"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::msg("query parameter is required"))?;

            let depth = args
                .and_then(|a| a.get("depth"))
                .and_then(|v| v.as_i64())
                .unwrap_or(1)
                .max(0) as usize;

            let max_pages = args
                .and_then(|a| a.get("max_pages"))
                .and_then(|v| v.as_i64())
                .unwrap_or(10)
                .max(1) as usize;

            let include_backlinks = args
                .and_then(|a| a.get("include_backlinks"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            explore_page(query, depth, max_pages, include_backlinks)?
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
