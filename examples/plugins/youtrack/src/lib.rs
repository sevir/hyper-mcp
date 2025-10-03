mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, InputSchema, ListToolsResult,
    PropertySchema, ToolDescription,
};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;

fn get_youtrack_config() -> Result<(String, String), Error> {
    let base_url = config::get("YOUTRACK_BASE_URL")?
        .ok_or_else(|| Error::msg("YOUTRACK_BASE_URL configuration is required but not set"))?;
    let api_token = config::get("YOUTRACK_API_TOKEN")?
        .ok_or_else(|| Error::msg("YOUTRACK_API_TOKEN configuration is required but not set"))?;

    Ok((base_url, api_token))
}

#[plugin_fn]
pub fn list_tools() -> FnResult<Json<ListToolsResult>> {
    let mut tools = Vec::new();

    // getTasksInformation tool
    let mut tasks_properties = HashMap::new();
    tasks_properties.insert(
        "name".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The name of the agile board to query".to_string(),
            default: None,
            items: None,
        },
    );
    tasks_properties.insert(
        "num_comments".to_string(),
        PropertySchema {
            property_type: "integer".to_string(),
            description: "Number of latest comments to retrieve per task (default: 1, 0 for none)"
                .to_string(),
            default: Some(json!(1)),
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "getTasksInformation".to_string(),
        description: "Read the Agile Panel from YouTrack, obtaining information about all the in-progress tasks and returns a markdown report detailing them. Includes task ID, summary, assignee, state, estimated vs spent time, time since last update, and recent comments."
            .to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: tasks_properties,
            required: Some(vec!["name".to_string()]),
        },
    });

    // getIssueById tool
    let mut issue_properties = HashMap::new();
    issue_properties.insert(
        "issue_id".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The ID of the issue to retrieve (e.g., 'DEMO-123' or '3-3')".to_string(),
            default: None,
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "getIssueById".to_string(),
        description: "Get detailed information about a specific issue by its ID. Designed for deep analysis of problematic issues, providing complete context including all comments, description, and metadata. Accepts both readable IDs (e.g., 'DEMO-123') and internal IDs (e.g., '3-3')."
            .to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: issue_properties,
            required: Some(vec!["issue_id".to_string()]),
        },
    });

    // createIssue tool
    let mut create_properties = HashMap::new();
    create_properties.insert(
        "project".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The project short name or ID where the issue will be created".to_string(),
            default: None,
            items: None,
        },
    );
    create_properties.insert(
        "summary".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The summary/title of the issue".to_string(),
            default: None,
            items: None,
        },
    );
    create_properties.insert(
        "description".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The description of the issue (optional)".to_string(),
            default: None,
            items: None,
        },
    );
    create_properties.insert(
        "fields".to_string(),
        PropertySchema {
            property_type: "object".to_string(),
            description:
                "Additional custom fields as a JSON object. Use getCustomFields to get available fields for the project."
                    .to_string(),
            default: None,
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "createIssue".to_string(),
        description: "Create a new issue in a YouTrack project. Accepts project name, summary, description, and custom fields. Custom fields are dynamic and project-specific - use getCustomFields first to discover available fields."
            .to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: create_properties,
            required: Some(vec!["project".to_string(), "summary".to_string()]),
        },
    });

    // updateIssue tool
    let mut update_properties = HashMap::new();
    update_properties.insert(
        "issue_id".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The ID of the issue to update (e.g., 'DEMO-123')".to_string(),
            default: None,
            items: None,
        },
    );
    update_properties.insert(
        "summary".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "New summary/title for the issue (optional)".to_string(),
            default: None,
            items: None,
        },
    );
    update_properties.insert(
        "description".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "New description for the issue (optional)".to_string(),
            default: None,
            items: None,
        },
    );
    update_properties.insert(
        "fields".to_string(),
        PropertySchema {
            property_type: "object".to_string(),
            description: "Custom fields to update as a JSON object. Use getCustomFields to get available fields for the project."
                .to_string(),
            default: None,
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "updateIssue".to_string(),
        description: "Update an existing issue in YouTrack. Can update summary, description, and custom fields. Custom fields are dynamic and project-specific."
            .to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: update_properties,
            required: Some(vec!["issue_id".to_string()]),
        },
    });

    // getCustomFields tool
    let mut fields_properties = HashMap::new();
    fields_properties.insert(
        "project".to_string(),
        PropertySchema {
            property_type: "string".to_string(),
            description: "The project short name or ID to get custom fields for".to_string(),
            default: None,
            items: None,
        },
    );

    tools.push(ToolDescription {
        name: "getCustomFields".to_string(),
        description: "Get all custom fields available for a specific project. Returns field names, types, and possible values. Use this before creating or updating issues to know which fields are available and their valid values."
            .to_string(),
        input_schema: InputSchema {
            schema_type: "object".to_string(),
            properties: fields_properties,
            required: Some(vec!["project".to_string()]),
        },
    });

    Ok(Json(ListToolsResult { tools }))
}

#[plugin_fn]
pub fn call(input: Json<CallToolRequest>) -> FnResult<Json<CallToolResult>> {
    let input = input.0;
    match input.params.name.as_str() {
        "getTasksInformation" => get_tasks_information(input),
        "getIssueById" => get_issue_by_id(input),
        "createIssue" => create_issue(input),
        "updateIssue" => update_issue(input),
        "getCustomFields" => get_custom_fields(input),
        _ => Ok(Json(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Unknown tool: {}", input.params.name)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        })),
    }
}

fn get_tasks_information(input: CallToolRequest) -> FnResult<Json<CallToolResult>> {
    let args = input.params.arguments.unwrap_or_default();
    let (base_url, api_token) = match get_youtrack_config() {
        Ok(config) => config,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Configuration error: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    // Extract parameters
    let board_name = match args.get("name") {
        Some(JsonValue::String(s)) if !s.is_empty() => s,
        _ => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing required parameter: name".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let num_comments = args
        .get("num_comments")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);

    // Step 1: Find the board by name
    let boards_url = format!("{}/agiles?fields=id,name,currentSprint(id)", base_url);
    let req = HttpRequest::new(&boards_url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", api_token))
        .with_header("Accept", "application/json");

    let boards_response = match http::request::<()>(&req, None) {
        Ok(res) => res,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to fetch agile boards: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let boards: Vec<JsonValue> = match serde_json::from_slice(&boards_response.body()) {
        Ok(b) => b,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to parse boards response: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let board = boards.iter().find(|b| {
        b.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n == board_name)
            .unwrap_or(false)
    });

    let (board_id, sprint_id) = match board {
        Some(b) => {
            let board_id = b
                .get("id")
                .and_then(|id| id.as_str())
                .ok_or_else(|| Error::msg("Board ID not found"))?;
            let sprint_id = b
                .get("currentSprint")
                .and_then(|s| s.get("id"))
                .and_then(|id| id.as_str())
                .ok_or_else(|| Error::msg("No active sprint found"))?;
            (board_id, sprint_id)
        }
        None => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Board '{}' not found", board_name)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    // Step 2: Get issues from the sprint
    let issues_url = format!(
        "{}/agiles/{}/sprints/{}/issues?fields=id,idReadable,summary,customFields(name,value(name,minutes)),reporter(name),updated,comments(text,author(name),created){}",
        base_url,
        board_id,
        sprint_id,
        if num_comments > 0 {
            format!("&$top={}", num_comments)
        } else {
            String::new()
        }
    );

    let req = HttpRequest::new(&issues_url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", api_token))
        .with_header("Accept", "application/json");

    let issues_response = match http::request::<()>(&req, None) {
        Ok(res) => res,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to fetch sprint issues: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let issues: Vec<JsonValue> = match serde_json::from_slice(&issues_response.body()) {
        Ok(i) => i,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to parse issues response: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    // Format the response
    let mut report = Vec::new();
    report.push(format!("# Tasks Report - {}", board_name));
    report.push(String::new());
    report.push(format!("## 📊 Summary"));
    report.push(format!("- **Total tasks in progress:** {}", issues.len()));
    report.push(String::new());
    report.push("## 📋 Tasks in Progress".to_string());
    report.push(String::new());
    report.push(
        "| ID | Summary | Assignee | State | Est. | Spent | Last Updated | Comments |".to_string(),
    );
    report.push("|---|---|---|---|---|---|---|---|".to_string());

    for issue in &issues {
        let id = issue
            .get("idReadable")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let summary = issue
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");

        let assignee = issue
            .get("customFields")
            .and_then(|fields| fields.as_array())
            .and_then(|arr| {
                arr.iter().find(|f| {
                    f.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n == "Assignee")
                        .unwrap_or(false)
                })
            })
            .and_then(|f| f.get("value"))
            .and_then(|v| v.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unassigned");

        let state = issue
            .get("customFields")
            .and_then(|fields| fields.as_array())
            .and_then(|arr| {
                arr.iter().find(|f| {
                    f.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n == "State")
                        .unwrap_or(false)
                })
            })
            .and_then(|f| f.get("value"))
            .and_then(|v| v.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("N/A");

        let updated = issue
            .get("updated")
            .and_then(|v| v.as_i64())
            .map(|ts| format_timestamp(ts))
            .unwrap_or_else(|| "N/A".to_string());

        let comments_text = if num_comments > 0 {
            issue
                .get("comments")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .take(num_comments as usize)
                        .filter_map(|c| {
                            let author = c
                                .get("author")
                                .and_then(|a| a.get("name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("Unknown");
                            let text = c.get("text").and_then(|t| t.as_str()).unwrap_or("");
                            if !text.is_empty() {
                                Some(format!("{}: {}", author, text))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("<br>")
                })
                .unwrap_or_else(|| "No comments".to_string())
        } else {
            "-".to_string()
        };

        report.push(format!(
            "| {} | {} | {} | {} | - | - | {} | {} |",
            id, summary, assignee, state, updated, comments_text
        ));
    }

    Ok(Json(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(report.join("\n")),
            mime_type: Some("text/markdown".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    }))
}

fn get_issue_by_id(input: CallToolRequest) -> FnResult<Json<CallToolResult>> {
    let args = input.params.arguments.unwrap_or_default();
    let (base_url, api_token) = match get_youtrack_config() {
        Ok(config) => config,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Configuration error: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let issue_id = match args.get("issue_id") {
        Some(JsonValue::String(s)) if !s.is_empty() => s,
        _ => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing required parameter: issue_id".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let issue_url = format!(
        "{}/issues/{}?fields=id,idReadable,summary,description,created,updated,customFields(name,value(name,minutes)),reporter(name),comments(text,author(name),created)",
        base_url, issue_id
    );

    let req = HttpRequest::new(&issue_url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", api_token))
        .with_header("Accept", "application/json");

    let response = match http::request::<()>(&req, None) {
        Ok(res) => {
            if res.status_code() >= 400 {
                return Ok(Json(CallToolResult {
                    is_error: Some(true),
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!(
                            "Issue '{}' not found or access denied (status: {})",
                            issue_id,
                            res.status_code()
                        )),
                        mime_type: None,
                        r#type: ContentType::Text,
                        data: None,
                    }],
                }));
            }
            res
        }
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to fetch issue: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let issue: JsonValue = match serde_json::from_slice(&response.body()) {
        Ok(i) => i,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to parse issue response: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let mut report = Vec::new();
    let id = issue
        .get("idReadable")
        .and_then(|v| v.as_str())
        .unwrap_or("N/A");
    let summary = issue
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("N/A");

    report.push(format!("# Issue Details: {}", id));
    report.push(String::new());
    report.push("## 📋 Basic Information".to_string());
    report.push(format!("| Field | Value |"));
    report.push("|---|---|".to_string());
    report.push(format!("| **ID** | {} |", id));
    report.push(format!("| **Summary** | {} |", summary));

    if let Some(description) = issue.get("description").and_then(|v| v.as_str()) {
        report.push(String::new());
        report.push("## 📝 Description".to_string());
        report.push(description.to_string());
    }

    // Custom fields
    if let Some(fields) = issue.get("customFields").and_then(|f| f.as_array()) {
        report.push(String::new());
        report.push("## 🏷️ Custom Fields".to_string());
        for field in fields {
            if let (Some(name), Some(value)) = (
                field.get("name").and_then(|n| n.as_str()),
                field.get("value"),
            ) {
                let value_str = if let Some(name_val) = value.get("name").and_then(|n| n.as_str()) {
                    name_val.to_string()
                } else if let Some(minutes) = value.get("minutes").and_then(|m| m.as_i64()) {
                    format!("{}h {}m", minutes / 60, minutes % 60)
                } else {
                    value.to_string()
                };
                report.push(format!("- **{}**: {}", name, value_str));
            }
        }
    }

    // Comments
    if let Some(comments) = issue.get("comments").and_then(|c| c.as_array()) {
        if !comments.is_empty() {
            report.push(String::new());
            report.push("## 💬 Comments".to_string());
            report.push(String::new());
            for comment in comments {
                let author = comment
                    .get("author")
                    .and_then(|a| a.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("Unknown");
                let text = comment.get("text").and_then(|t| t.as_str()).unwrap_or("");
                let created = comment
                    .get("created")
                    .and_then(|c| c.as_i64())
                    .map(|ts| format_timestamp(ts))
                    .unwrap_or_else(|| "Unknown".to_string());

                report.push(format!("### {} - {}", author, created));
                report.push(text.to_string());
                report.push(String::new());
            }
        }
    }

    Ok(Json(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(report.join("\n")),
            mime_type: Some("text/markdown".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    }))
}

fn create_issue(input: CallToolRequest) -> FnResult<Json<CallToolResult>> {
    let args = input.params.arguments.unwrap_or_default();
    let (base_url, api_token) = match get_youtrack_config() {
        Ok(config) => config,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Configuration error: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let project = match args.get("project") {
        Some(JsonValue::String(s)) if !s.is_empty() => s,
        _ => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing required parameter: project".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let summary = match args.get("summary") {
        Some(JsonValue::String(s)) if !s.is_empty() => s,
        _ => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing required parameter: summary".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let mut body = json!({
        "project": {"id": project},
        "summary": summary
    });

    if let Some(JsonValue::String(desc)) = args.get("description") {
        body["description"] = json!(desc);
    }

    if let Some(fields) = args.get("fields") {
        body["customFields"] = fields.clone();
    }

    let issues_url = format!("{}/issues?fields=id,idReadable", base_url);
    let req = HttpRequest::new(&issues_url)
        .with_method("POST")
        .with_header("Authorization", &format!("Bearer {}", api_token))
        .with_header("Content-Type", "application/json")
        .with_header("Accept", "application/json");

    let response = match http::request::<String>(&req, Some(body.to_string())) {
        Ok(res) => {
            if res.status_code() >= 400 {
                let body_str = String::from_utf8_lossy(&res.body()).to_string();
                return Ok(Json(CallToolResult {
                    is_error: Some(true),
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!(
                            "Failed to create issue (status: {}): {}",
                            res.status_code(),
                            body_str
                        )),
                        mime_type: None,
                        r#type: ContentType::Text,
                        data: None,
                    }],
                }));
            }
            res
        }
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to create issue: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let issue: JsonValue = match serde_json::from_slice(&response.body()) {
        Ok(i) => i,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to parse created issue response: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let issue_id = issue
        .get("idReadable")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    Ok(Json(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(format!("✅ Successfully created issue: {}", issue_id)),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    }))
}

fn update_issue(input: CallToolRequest) -> FnResult<Json<CallToolResult>> {
    let args = input.params.arguments.unwrap_or_default();
    let (base_url, api_token) = match get_youtrack_config() {
        Ok(config) => config,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Configuration error: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let issue_id = match args.get("issue_id") {
        Some(JsonValue::String(s)) if !s.is_empty() => s,
        _ => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing required parameter: issue_id".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let mut body = json!({});
    let mut has_updates = false;

    if let Some(JsonValue::String(summary)) = args.get("summary") {
        body["summary"] = json!(summary);
        has_updates = true;
    }

    if let Some(JsonValue::String(desc)) = args.get("description") {
        body["description"] = json!(desc);
        has_updates = true;
    }

    if let Some(fields) = args.get("fields") {
        body["customFields"] = fields.clone();
        has_updates = true;
    }

    if !has_updates {
        return Ok(Json(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(
                    "No fields to update. Provide summary, description, or fields.".to_string(),
                ),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }));
    }

    let update_url = format!("{}/issues/{}", base_url, issue_id);
    let req = HttpRequest::new(&update_url)
        .with_method("POST")
        .with_header("Authorization", &format!("Bearer {}", api_token))
        .with_header("Content-Type", "application/json");

    match http::request::<String>(&req, Some(body.to_string())) {
        Ok(res) => {
            if res.status_code() >= 400 {
                let body_str = String::from_utf8_lossy(&res.body()).to_string();
                return Ok(Json(CallToolResult {
                    is_error: Some(true),
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!(
                            "Failed to update issue (status: {}): {}",
                            res.status_code(),
                            body_str
                        )),
                        mime_type: None,
                        r#type: ContentType::Text,
                        data: None,
                    }],
                }));
            }
        }
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to update issue: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    }

    Ok(Json(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(format!("✅ Successfully updated issue: {}", issue_id)),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    }))
}

fn get_custom_fields(input: CallToolRequest) -> FnResult<Json<CallToolResult>> {
    let args = input.params.arguments.unwrap_or_default();
    let (base_url, api_token) = match get_youtrack_config() {
        Ok(config) => config,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Configuration error: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let project = match args.get("project") {
        Some(JsonValue::String(s)) if !s.is_empty() => s,
        _ => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing required parameter: project".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let fields_url = format!(
        "{}/admin/projects/{}/customFields?fields=field(name,fieldType(id)),bundle(values(name))",
        base_url, project
    );

    let req = HttpRequest::new(&fields_url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", api_token))
        .with_header("Accept", "application/json");

    let response = match http::request::<()>(&req, None) {
        Ok(res) => {
            if res.status_code() >= 400 {
                return Ok(Json(CallToolResult {
                    is_error: Some(true),
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!(
                            "Project '{}' not found or access denied (status: {})",
                            project,
                            res.status_code()
                        )),
                        mime_type: None,
                        r#type: ContentType::Text,
                        data: None,
                    }],
                }));
            }
            res
        }
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to fetch custom fields: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let fields: Vec<JsonValue> = match serde_json::from_slice(&response.body()) {
        Ok(f) => f,
        Err(e) => {
            return Ok(Json(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to parse custom fields response: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            }));
        }
    };

    let mut report = Vec::new();
    report.push(format!("# Custom Fields for Project: {}", project));
    report.push(String::new());
    report.push("| Field Name | Type | Possible Values |".to_string());
    report.push("|---|---|---|".to_string());

    for field in &fields {
        let name = field
            .get("field")
            .and_then(|f| f.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown");

        let field_type = field
            .get("field")
            .and_then(|f| f.get("fieldType"))
            .and_then(|ft| ft.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("Unknown");

        let values = field
            .get("bundle")
            .and_then(|b| b.get("values"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "-".to_string());

        report.push(format!("| {} | {} | {} |", name, field_type, values));
    }

    Ok(Json(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(report.join("\n")),
            mime_type: Some("text/markdown".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    }))
}

fn format_timestamp(timestamp_ms: i64) -> String {
    let seconds = timestamp_ms / 1000;
    let now = chrono::Utc::now().timestamp();
    let diff = now - seconds;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}
