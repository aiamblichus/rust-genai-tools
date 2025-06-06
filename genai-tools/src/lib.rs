//! # genai-tools
//!
//! Type-safe tool definition macros for the genai crate.
//!
//! This crate provides ergonomic macros and utilities for defining LLM tools
//! with automatic JSON schema generation from Rust types.
//!
//! ## Example
//!
//! ```ignore
//! use genai_tools::{tool_function, ToolRegistry};
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//!
//! #[derive(Deserialize, JsonSchema)]
//! pub struct WeatherParams {
//!     /// The city name
//!     pub city: String,
//!     /// The country of the city
//!     pub country: String,
//! }
//!
//! #[derive(Serialize)]
//! pub struct WeatherResult {
//!     pub temperature: f64,
//!     pub condition: String,
//! }
//!
//! #[tool_function(
//!     name = "get_weather",
//!     description = "Get the current weather for a location"
//! )]
//! pub async fn get_weather(params: WeatherParams) -> Result<WeatherResult, Box<dyn std::error::Error + Send + Sync>> {
//!     Ok(WeatherResult {
//!         temperature: 22.5,
//!         condition: "Sunny".to_string(),
//!     })
//! }
//!
//! // Usage
//! let mut registry = ToolRegistry::new();
//! registry.register_function(get_weather);
//! ```

mod registry;
mod traits;

pub use registry::ToolRegistry;
pub use traits::*;

// Re-export the proc macro
pub use genai_tools_macros::tool_function;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use schemars::JsonSchema;


    // Test types
    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    struct SimpleParams {
        /// A simple string parameter
        pub message: String,
    }

    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    struct ComplexParams {
        /// The name field
        pub name: String,
        /// The age field
        pub age: u32,
        /// Optional email
        pub email: Option<String>,
        /// A list of tags
        pub tags: Vec<String>,
        /// Operation type
        pub operation: Operation,
    }

    #[derive(Debug, Deserialize, JsonSchema, PartialEq, Clone)]
    pub enum Operation {
        #[serde(rename = "create")]
        Create,
        #[serde(rename = "update")]
        Update,
        #[serde(rename = "delete")]
        Delete,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SimpleResult {
        pub success: bool,
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ComplexResult {
        pub id: u64,
        pub name: String,
        pub created_at: String,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum TestError {
        #[error("Invalid input: {msg}")]
        InvalidInput { msg: String }
    }

    // Test functions
    async fn simple_tool_impl(params: SimpleParams) -> Result<SimpleResult, TestError> {
        if params.message.is_empty() {
            return Err(TestError::InvalidInput { 
                msg: "Message cannot be empty".to_string() 
            });
        }
        
        Ok(SimpleResult {
            success: true,
            message: format!("Processed: {}", params.message),
        })
    }

    async fn complex_tool_impl(params: ComplexParams) -> Result<ComplexResult, TestError> {
        if params.age > 150 {
            return Err(TestError::InvalidInput { 
                msg: "Age too high".to_string() 
            });
        }

        Ok(ComplexResult {
            id: 12345,
            name: params.name,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        })
    }

    #[test]
    fn test_schema_generation_for_simple_params() {
        // Test that we can generate schema from types
        let schema = schemars::schema_for!(SimpleParams);
        
        // Convert to serde_json::Value to test contents
        let schema_json = serde_json::to_value(&schema).unwrap();
        
        // Check that it's an object with the expected structure
        assert!(schema_json.is_object());
        assert!(schema_json.get("$schema").is_some());
        
        // Check that the main schema definition exists
        if let Some(schema_def) = schema_json.get("properties") {
            let props = schema_def.as_object().unwrap();
            assert!(props.contains_key("message"));
        }
    }

    #[test]
    fn test_schema_generation_for_complex_params() {
        let schema = schemars::schema_for!(ComplexParams);
        
        // Convert to serde_json::Value to test contents
        let schema_json = serde_json::to_value(&schema).unwrap();
        
        assert!(schema_json.is_object());
        
        // Check that the main schema definition exists
        if let Some(schema_def) = schema_json.get("properties") {
            let props = schema_def.as_object().unwrap();
            
            // Check all expected properties exist
            assert!(props.contains_key("name"));
            assert!(props.contains_key("age"));
            assert!(props.contains_key("email"));
            assert!(props.contains_key("tags"));
            assert!(props.contains_key("operation"));
        }
    }

    #[tokio::test]
    async fn test_basic_function_execution() {
        let params = SimpleParams {
            message: "Hello, World!".to_string(),
        };
        
        let result = simple_tool_impl(params).await.unwrap();
        
        assert_eq!(result.success, true);
        assert_eq!(result.message, "Processed: Hello, World!");
    }

    #[tokio::test]
    async fn test_function_error_handling() {
        let params = SimpleParams {
            message: "".to_string(), // Empty message should trigger error
        };
        
        let result = simple_tool_impl(params).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Message cannot be empty"));
    }

    #[tokio::test]
    async fn test_complex_function_execution() {
        let params = ComplexParams {
            name: "Alice".to_string(),
            age: 25,
            email: Some("alice@example.com".to_string()),
            tags: vec!["admin".to_string(), "active".to_string()],
            operation: Operation::Create,
        };
        
        let result = complex_tool_impl(params).await.unwrap();
        
        assert_eq!(result.id, 12345);
        assert_eq!(result.name, "Alice");
        assert_eq!(result.created_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert_eq!(registry.tool_names().len(), 0);
    }

    #[test]
    fn test_enum_serialization_in_schema() {
        let schema = schemars::schema_for!(Operation);
        
        // Check that the Operation enum is properly defined in the schema
        let schema_str = serde_json::to_string(&schema).unwrap();
        assert!(schema_str.contains("create"));
        assert!(schema_str.contains("update"));
        assert!(schema_str.contains("delete"));
    }
} 