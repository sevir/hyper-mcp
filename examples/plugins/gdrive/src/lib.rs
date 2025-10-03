mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value as JsonValue, json};

const GOOGLE_DOCS_API_BASE: &str = "https://docs.googleapis.com/v1";
const GOOGLE_DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";

fn get_google_config() -> Result<String, Error> {
    let access_token = config::get("GOOGLE_ACCESS_TOKEN")?
        .ok_or_else(|| Error::msg("GOOGLE_ACCESS_TOKEN configuration is required but not set"))?;

    Ok(access_token)
}

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "list_files" => list_files(input),
        "get_file" => get_file(input),
        "create_document" => create_document(input),
        "read_document" => read_document(input),
        "append_to_document" => append_to_document(input),
        "insert_text" => insert_text(input),
        "search_files" => search_files(input),
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

fn list_files(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let access_token = get_google_config()?;

    // Optional parameters
    let page_size = args.get("page_size").and_then(|v| v.as_i64()).unwrap_or(10);

    let folder_id = args.get("folder_id").and_then(|v| v.as_str()).unwrap_or("");

    let mut query_parts = Vec::new();
    if !folder_id.is_empty() {
        query_parts.push(format!("'{}' in parents", folder_id));
    }
    query_parts.push("trashed=false".to_string());

    let query = query_parts.join(" and ");
    let url = format!(
        "{}/files?pageSize={}&q={}",
        GOOGLE_DRIVE_API_BASE,
        page_size,
        urlencoding::encode(&query)
    );

    let req = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", access_token));

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut result_text = String::from("Files:\n\n");

                        if let Some(files) = parsed_json.get("files") {
                            if let JsonValue::Array(files_array) = files {
                                for (idx, file) in files_array.iter().enumerate() {
                                    result_text.push_str(&format!("{}. ", idx + 1));

                                    if let Some(name) = file.get("name").and_then(|v| v.as_str()) {
                                        result_text.push_str(&format!("Name: {}\n", name));
                                    }

                                    if let Some(id) = file.get("id").and_then(|v| v.as_str()) {
                                        result_text.push_str(&format!("   ID: {}\n", id));
                                    }

                                    if let Some(mime_type) =
                                        file.get("mimeType").and_then(|v| v.as_str())
                                    {
                                        result_text.push_str(&format!("   Type: {}\n", mime_type));
                                    }

                                    result_text.push('\n');
                                }
                            }
                        }

                        Ok(CallToolResult {
                            is_error: None,
                            content: vec![Content {
                                annotations: None,
                                text: Some(result_text),
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
                                "Failed to parse API response: {}. Body: {}",
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

fn get_file(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let access_token = get_google_config()?;

    let file_id = match args.get("file_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: file_id".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let url = format!("{}/files/{}?fields=*", GOOGLE_DRIVE_API_BASE, file_id);

    let req = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", access_token));

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                Ok(CallToolResult {
                    is_error: None,
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!("File Information:\n\n{}", body_str)),
                        mime_type: Some("application/json".to_string()),
                        r#type: ContentType::Text,
                        data: None,
                    }],
                })
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

fn create_document(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let access_token = get_google_config()?;

    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Document");

    let request_body = json!({
        "title": title
    });

    let url = format!("{}/documents", GOOGLE_DOCS_API_BASE);

    let req = HttpRequest::new(&url)
        .with_method("POST")
        .with_header("Authorization", &format!("Bearer {}", access_token))
        .with_header("Content-Type", "application/json");

    match http::request::<String>(&req, Some(request_body.to_string())) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut result_text = String::from("Document Created:\n\n");

                        if let Some(doc_id) = parsed_json.get("documentId").and_then(|v| v.as_str())
                        {
                            result_text.push_str(&format!("Document ID: {}\n", doc_id));
                        }

                        if let Some(title) = parsed_json.get("title").and_then(|v| v.as_str()) {
                            result_text.push_str(&format!("Title: {}\n", title));
                        }

                        Ok(CallToolResult {
                            is_error: None,
                            content: vec![Content {
                                annotations: None,
                                text: Some(result_text),
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
                                "Failed to parse API response: {}. Body: {}",
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

fn read_document(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let access_token = get_google_config()?;

    let document_id = match args.get("document_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: document_id".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let url = format!("{}/documents/{}", GOOGLE_DOCS_API_BASE, document_id);

    let req = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", access_token));

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut result_text = String::from("Document Content:\n\n");

                        if let Some(title) = parsed_json.get("title").and_then(|v| v.as_str()) {
                            result_text.push_str(&format!("Title: {}\n\n", title));
                        }

                        // Extract text content
                        if let Some(body) = parsed_json.get("body") {
                            if let Some(content) = body.get("content") {
                                if let JsonValue::Array(content_array) = content {
                                    for element in content_array {
                                        if let Some(paragraph) = element.get("paragraph") {
                                            if let Some(elements) = paragraph.get("elements") {
                                                if let JsonValue::Array(elem_array) = elements {
                                                    for elem in elem_array {
                                                        if let Some(text_run) = elem.get("textRun")
                                                        {
                                                            if let Some(content) = text_run
                                                                .get("content")
                                                                .and_then(|v| v.as_str())
                                                            {
                                                                result_text.push_str(content);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Ok(CallToolResult {
                            is_error: None,
                            content: vec![Content {
                                annotations: None,
                                text: Some(result_text),
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
                                "Failed to parse API response: {}. Body: {}",
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

fn append_to_document(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let access_token = get_google_config()?;

    let document_id = match args.get("document_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: document_id".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: text".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let request_body = json!({
        "requests": [
            {
                "insertText": {
                    "location": {
                        "index": 1
                    },
                    "text": format!("{}\n", text)
                }
            }
        ]
    });

    let url = format!(
        "{}/documents/{}:batchUpdate",
        GOOGLE_DOCS_API_BASE, document_id
    );

    let req = HttpRequest::new(&url)
        .with_method("POST")
        .with_header("Authorization", &format!("Bearer {}", access_token))
        .with_header("Content-Type", "application/json");

    match http::request::<String>(&req, Some(request_body.to_string())) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                Ok(CallToolResult {
                    is_error: None,
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!(
                            "Text appended successfully to document {}",
                            document_id
                        )),
                        mime_type: Some("text/plain".to_string()),
                        r#type: ContentType::Text,
                        data: None,
                    }],
                })
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

fn insert_text(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let access_token = get_google_config()?;

    let document_id = match args.get("document_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: document_id".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: text".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let index = args.get("index").and_then(|v| v.as_i64()).unwrap_or(1);

    let request_body = json!({
        "requests": [
            {
                "insertText": {
                    "location": {
                        "index": index
                    },
                    "text": text
                }
            }
        ]
    });

    let url = format!(
        "{}/documents/{}:batchUpdate",
        GOOGLE_DOCS_API_BASE, document_id
    );

    let req = HttpRequest::new(&url)
        .with_method("POST")
        .with_header("Authorization", &format!("Bearer {}", access_token))
        .with_header("Content-Type", "application/json");

    match http::request::<String>(&req, Some(request_body.to_string())) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                Ok(CallToolResult {
                    is_error: None,
                    content: vec![Content {
                        annotations: None,
                        text: Some(format!(
                            "Text inserted successfully at index {} in document {}",
                            index, document_id
                        )),
                        mime_type: Some("text/plain".to_string()),
                        r#type: ContentType::Text,
                        data: None,
                    }],
                })
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

fn search_files(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let access_token = get_google_config()?;

    let query_str = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) if !q.is_empty() => q,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing or invalid required parameter: query".to_string()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let page_size = args.get("page_size").and_then(|v| v.as_i64()).unwrap_or(10);

    let search_query = format!("name contains '{}' and trashed=false", query_str);
    let url = format!(
        "{}/files?pageSize={}&q={}",
        GOOGLE_DRIVE_API_BASE,
        page_size,
        urlencoding::encode(&search_query)
    );

    let req = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("Authorization", &format!("Bearer {}", access_token));

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                match serde_json::from_str::<JsonValue>(&body_str) {
                    Ok(parsed_json) => {
                        let mut result_text = format!("Search Results for '{}':\n\n", query_str);

                        if let Some(files) = parsed_json.get("files") {
                            if let JsonValue::Array(files_array) = files {
                                if files_array.is_empty() {
                                    result_text.push_str("No files found.\n");
                                } else {
                                    for (idx, file) in files_array.iter().enumerate() {
                                        result_text.push_str(&format!("{}. ", idx + 1));

                                        if let Some(name) =
                                            file.get("name").and_then(|v| v.as_str())
                                        {
                                            result_text.push_str(&format!("Name: {}\n", name));
                                        }

                                        if let Some(id) = file.get("id").and_then(|v| v.as_str()) {
                                            result_text.push_str(&format!("   ID: {}\n", id));
                                        }

                                        if let Some(mime_type) =
                                            file.get("mimeType").and_then(|v| v.as_str())
                                        {
                                            result_text
                                                .push_str(&format!("   Type: {}\n", mime_type));
                                        }

                                        result_text.push('\n');
                                    }
                                }
                            }
                        }

                        Ok(CallToolResult {
                            is_error: None,
                            content: vec![Content {
                                annotations: None,
                                text: Some(result_text),
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
                                "Failed to parse API response: {}. Body: {}",
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
                name: "list_files".into(),
                description: "List files from Google Drive. Optionally filter by folder.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "folder_id": {
                            "type": "string",
                            "description": "Optional folder ID to list files from. If not provided, lists files from root.",
                        },
                        "page_size": {
                            "type": "integer",
                            "description": "Number of files to return (default: 10, max: 100)",
                            "default": 10,
                            "minimum": 1,
                            "maximum": 100,
                        },
                    },
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "get_file".into(),
                description: "Get detailed information about a specific file by ID.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_id": {
                            "type": "string",
                            "description": "The ID of the file to retrieve information about",
                        },
                    },
                    "required": ["file_id"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "create_document".into(),
                description: "Create a new Google Document.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "The title of the new document",
                            "default": "Untitled Document",
                        },
                    },
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "read_document".into(),
                description: "Read the content of a Google Document by ID.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "The ID of the document to read",
                        },
                    },
                    "required": ["document_id"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "append_to_document".into(),
                description: "Append text to the end of a Google Document.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "The ID of the document to append to",
                        },
                        "text": {
                            "type": "string",
                            "description": "The text to append to the document",
                        },
                    },
                    "required": ["document_id", "text"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "insert_text".into(),
                description: "Insert text at a specific position in a Google Document.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "document_id": {
                            "type": "string",
                            "description": "The ID of the document",
                        },
                        "text": {
                            "type": "string",
                            "description": "The text to insert",
                        },
                        "index": {
                            "type": "integer",
                            "description": "The position to insert the text (default: 1 for beginning)",
                            "default": 1,
                            "minimum": 1,
                        },
                    },
                    "required": ["document_id", "text"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "search_files".into(),
                description: "Search for files in Google Drive by name.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query string to find files by name",
                        },
                        "page_size": {
                            "type": "integer",
                            "description": "Number of results to return (default: 10, max: 100)",
                            "default": 10,
                            "minimum": 1,
                            "maximum": 100,
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
