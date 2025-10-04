#![allow(unused_macros)]

use extism_pdk::*;

pub(crate) mod internal {
    pub(crate) fn return_error(e: extism_pdk::Error) -> i32 {
        let err = format!("{:?}", e);
        let mem = extism_pdk::Memory::from_bytes(&err).unwrap();
        unsafe {
            extism_pdk::extism::error_set(mem.offset());
        }
        -1
    }
}

#[allow(unused)]
macro_rules! try_input {
    () => {{
        let x = extism_pdk::input();
        match x {
            Ok(x) => x,
            Err(e) => return internal::return_error(e),
        }
    }};
}

#[allow(unused)]
macro_rules! try_input_json {
    () => {{
        let x = extism_pdk::input();
        match x {
            Ok(extism_pdk::Json(x)) => x,
            Err(e) => return internal::return_error(e),
        }
    }};
}

mod exports {
    use super::*;

    #[unsafe(no_mangle)]
    pub extern "C" fn call() -> i32 {
        let ret =
            crate::call(try_input_json!()).and_then(|x| extism_pdk::output(extism_pdk::Json(x)));

        match ret {
            Ok(()) => 0,
            Err(e) => internal::return_error(e),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn describe() -> i32 {
        let ret = crate::describe().and_then(|x| extism_pdk::output(extism_pdk::Json(x)));

        match ret {
            Ok(()) => 0,
            Err(e) => internal::return_error(e),
        }
    }
}

pub mod types {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;
    use std::collections::HashMap;

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct ListToolsResult {
        #[serde(rename = "tools")]
        pub tools: Vec<ToolDescription>,
    }

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct ToolDescription {
        #[serde(rename = "name")]
        pub name: String,
        #[serde(rename = "description")]
        pub description: String,
        #[serde(rename = "inputSchema")]
        pub input_schema: InputSchema,
    }

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct InputSchema {
        #[serde(rename = "type")]
        pub schema_type: String,
        #[serde(rename = "properties")]
        pub properties: HashMap<String, PropertySchema>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "required")]
        pub required: Option<Vec<String>>,
    }

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct PropertySchema {
        #[serde(rename = "type")]
        pub property_type: String,
        #[serde(rename = "description")]
        pub description: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "default")]
        pub default: Option<JsonValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "items")]
        pub items: Option<Box<PropertySchema>>,
    }

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct CallToolRequest {
        #[serde(rename = "method")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub method: Option<String>,
        #[serde(rename = "params")]
        pub params: ToolCallParams,
    }

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct ToolCallParams {
        #[serde(rename = "name")]
        pub name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "arguments")]
        pub arguments: Option<HashMap<String, JsonValue>>,
    }

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct CallToolResult {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "isError")]
        pub is_error: Option<bool>,
        #[serde(rename = "content")]
        pub content: Vec<Content>,
    }

    #[derive(
        Default, Debug, Clone, Serialize, Deserialize, extism_pdk::FromBytes, extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct Content {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "annotations")]
        pub annotations: Option<JsonValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "text")]
        pub text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "mimeType")]
        pub mime_type: Option<String>,
        #[serde(rename = "type")]
        pub r#type: ContentType,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde(rename = "data")]
        pub data: Option<String>,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        Copy,
        Serialize,
        Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    #[serde(rename_all = "lowercase")]
    pub enum ContentType {
        #[default]
        Text,
        Image,
        Resource,
    }
}

#[cfg(test)]
mod tests {
    use super::types::CallToolRequest;
    use serde_json::json;

    #[test]
    fn call_tool_request_allows_missing_method() {
        let value = json!({
            "params": {
                "name": "search_pages",
                "arguments": {
                    "query": "Prácticas en empresa 25"
                }
            }
        });

        let request: CallToolRequest = serde_json::from_value(value).expect("valid request");

        assert!(request.method.is_none());
        assert_eq!(request.params.name, "search_pages");
        assert!(
            request
                .params
                .arguments
                .as_ref()
                .and_then(|args| args.get("query"))
                .is_some()
        );
    }
}
