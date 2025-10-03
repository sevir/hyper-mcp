pub mod types {
    use serde::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ListToolsResult {
        pub tools: Vec<ToolDescription>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ToolDescription {
        pub name: String,
        pub description: String,
        #[serde(rename = "inputSchema")]
        pub input_schema: InputSchema,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct InputSchema {
        #[serde(rename = "type")]
        pub schema_type: String,
        pub properties: HashMap<String, PropertySchema>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub required: Option<Vec<String>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PropertySchema {
        #[serde(rename = "type")]
        pub property_type: String,
        pub description: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<JsonValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub items: Option<Box<PropertySchema>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CallToolRequest {
        pub method: String,
        pub params: ToolCallParams,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ToolCallParams {
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub arguments: Option<HashMap<String, JsonValue>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CallToolResult {
        #[serde(skip_serializing_if = "Option::is_none", rename = "isError")]
        pub is_error: Option<bool>,
        pub content: Vec<Content>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Content {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub annotations: Option<JsonValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
        pub mime_type: Option<String>,
        #[serde(rename = "type")]
        pub r#type: ContentType,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    pub enum ContentType {
        Text,
        Image,
        Resource,
    }
}
