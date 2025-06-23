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
        "mvn_fetch_deps" => mvn_fetch_deps(input),
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

fn mvn_fetch_deps(input: CallToolRequest) -> Result<CallToolResult, Error> {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    use serde_json::json;

    let args = input.params.arguments.unwrap_or_default();
    let group = match args.get("group") {
        Some(Value::String(s)) => s,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing 'group' argument".into()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };
    let artifact = match args.get("artifact") {
        Some(Value::String(s)) => s,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing 'artifact' argument".into()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };
    let version = match args.get("version") {
        Some(Value::String(s)) => s,
        _ => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some("Missing 'version' argument".into()),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };

    let group_path = group.replace('.', "/");
    let pom_url = format!(
        "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}.pom",
        group_path, artifact, version, artifact, version
    );

    let mut req = HttpRequest {
        url: pom_url.clone(),
        headers: BTreeMap::new(),
        method: Some("GET".to_string()),
    };
    req.headers
        .insert("User-Agent".to_string(), "maven-plugin/1.0".to_string());

    let res = match http::request::<()>(&req, None) {
        Ok(r) => r,
        Err(e) => {
            return Ok(CallToolResult {
                is_error: Some(true),
                content: vec![Content {
                    annotations: None,
                    text: Some(format!("Failed to fetch POM: {}", e)),
                    mime_type: None,
                    r#type: ContentType::Text,
                    data: None,
                }],
            });
        }
    };
    let body = res.body();
    let xml = String::from_utf8_lossy(body.as_slice());

    // Parse dependencies
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut dependencies = Vec::new();
    let mut in_dependencies = false;
    let mut current = serde_json::Map::new();
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "dependencies" {
                    in_dependencies = true;
                } else if in_dependencies && tag == "dependency" {
                    current = serde_json::Map::new();
                } else if in_dependencies {
                    current_tag = tag;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "dependencies" {
                    in_dependencies = false;
                } else if in_dependencies && tag == "dependency" {
                    dependencies.push(json!(current));
                } else if in_dependencies {
                    current_tag.clear();
                }
            }
            Ok(Event::Text(e)) => {
                if in_dependencies && !current_tag.is_empty() {
                    current.insert(current_tag.clone(), json!(e.unescape().unwrap_or_default()));
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(json!({"dependencies": dependencies}).to_string()),
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
                name: "mvn_fetch_deps".into(),
                description:  "Fetches the dependencies of a Maven package by group, artifact, and version from Maven Central.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "group": {
                            "type": "string",
                            "description": "The Maven groupId",
                        },
                        "artifact": {
                            "type": "string",
                            "description": "The Maven artifactId",
                        },
                        "version": {
                            "type": "string",
                            "description": "The Maven version",
                        },
                    },
                    "required": ["group", "artifact", "version"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
