use genai_tools::{tool_function, ToolRegistry, ToolFunction};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use serde_json::json;

// Integration test for the macro expansion
#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
pub struct IntegrationParams {
    /// A required string field
    pub name: String,
    /// An optional integer field
    pub count: Option<i32>,
    /// A vector of strings
    pub items: Vec<String>,
    /// A nested enum
    pub status: Status,
}

#[derive(Debug, Deserialize, JsonSchema, PartialEq, Clone)]
pub enum Status {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "pending")]
    Pending,
}

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct IntegrationResult {
        pub processed: bool,
        pub name: String,
        pub item_count: usize,
        pub status_text: String,
    }

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Validation failed: {message}")]
    ValidationFailed { message: String },
    #[error("Processing error")]
    ProcessingError,
}

#[tool_function(
    name = "integration_test_tool",
    description = "A comprehensive integration test tool"
)]
pub async fn integration_test_tool(params: IntegrationParams) -> Result<IntegrationResult, IntegrationError> {
    if params.name.is_empty() {
        return Err(IntegrationError::ValidationFailed {
            message: "Name cannot be empty".to_string(),
        });
    }

    if params.items.len() > 10 {
        return Err(IntegrationError::ValidationFailed {
            message: "Too many items".to_string(),
        });
    }

    let status_text = match params.status {
        Status::Active => "Currently active",
        Status::Inactive => "Currently inactive", 
        Status::Pending => "Awaiting activation",
    };

    Ok(IntegrationResult {
        processed: true,
        name: params.name,
        item_count: params.items.len(),
        status_text: status_text.to_string(),
    })
}

#[tool_function(description = "Tool with minimal configuration")]
pub async fn minimal_tool(params: IntegrationParams) -> Result<IntegrationResult, IntegrationError> {
    Ok(IntegrationResult {
        processed: true,
        name: params.name,
        item_count: params.items.len(),
        status_text: "minimal".to_string(),
    })
}

#[tokio::test]
async fn test_macro_generates_correct_struct() {
    let tool = integration_test_tool_tool();
    
    // Test that the macro correctly implements ToolFunction trait
    assert_eq!(tool.name(), "integration_test_tool");
    assert_eq!(tool.description(), "A comprehensive integration test tool");
    
    // Test that schema is generated
    let schema = tool.schema();
    assert!(schema.is_object());
    
    // Verify schema contains expected properties
    let schema_obj = schema.as_object().unwrap();
    let properties = schema_obj["properties"].as_object().unwrap();
    
    assert!(properties.contains_key("name"));
    assert!(properties.contains_key("count"));
    assert!(properties.contains_key("items"));
    assert!(properties.contains_key("status"));
    
    // Verify required fields
    let required = schema_obj["required"].as_array().unwrap();
    assert!(required.contains(&json!("name")));
    assert!(required.contains(&json!("items")));
    assert!(required.contains(&json!("status")));
    assert!(!required.contains(&json!("count"))); // Optional field
}

#[tokio::test]
async fn test_tool_execution_with_all_parameter_types() {
    let tool = integration_test_tool_tool();
    
    let params = IntegrationParams {
        name: "Test Tool".to_string(),
        count: Some(42),
        items: vec!["item1".to_string(), "item2".to_string(), "item3".to_string()],
        status: Status::Active,
    };
    
    let result = tool.call(params).await.unwrap();
    
    assert_eq!(result.processed, true);
    assert_eq!(result.name, "Test Tool");
    assert_eq!(result.item_count, 3);
    assert_eq!(result.status_text, "Currently active");
}

#[tokio::test]
async fn test_tool_execution_with_optional_params() {
    let tool = integration_test_tool_tool();
    
    let params = IntegrationParams {
        name: "Minimal Test".to_string(),
        count: None, // Optional field omitted
        items: vec![], // Empty vector
        status: Status::Pending,
    };
    
    let result = tool.call(params).await.unwrap();
    
    assert_eq!(result.processed, true);
    assert_eq!(result.name, "Minimal Test");
    assert_eq!(result.item_count, 0);
    assert_eq!(result.status_text, "Awaiting activation");
}

#[tokio::test]
async fn test_tool_error_handling() {
    let tool = integration_test_tool_tool();
    
    // Test empty name validation
    let params = IntegrationParams {
        name: "".to_string(), // Empty name should cause error
        count: Some(1),
        items: vec!["test".to_string()],
        status: Status::Active,
    };
    
    let result = tool.call(params).await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Name cannot be empty"));
}

#[tokio::test]
async fn test_tool_error_handling_too_many_items() {
    let tool = integration_test_tool_tool();
    
    // Test too many items validation
    let params = IntegrationParams {
        name: "Test".to_string(),
        count: Some(1),
        items: (0..15).map(|i| format!("item{}", i)).collect(), // Too many items
        status: Status::Active,
    };
    
    let result = tool.call(params).await;
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Too many items"));
}

#[tokio::test]
async fn test_json_execution_roundtrip() {
    let tool = integration_test_tool_tool();
    
    let params_json = json!({
        "name": "JSON Test",
        "count": 123,
        "items": ["a", "b", "c"],
        "status": "inactive"
    });
    
    let result_json = tool.call_json(params_json).await.unwrap();
    
    // Parse the result back
    let result: IntegrationResult = serde_json::from_value(result_json).unwrap();
    
    assert_eq!(result.processed, true);
    assert_eq!(result.name, "JSON Test");
    assert_eq!(result.item_count, 3);
    assert_eq!(result.status_text, "Currently inactive");
}

#[tokio::test]
async fn test_json_execution_with_invalid_enum() {
    let tool = integration_test_tool_tool();
    
    let params_json = json!({
        "name": "Test",
        "items": [],
        "status": "invalid_status" // Invalid enum value
    });
    
    let result = tool.call_json(params_json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_json_execution_with_missing_required_field() {
    let tool = integration_test_tool_tool();
    
    let params_json = json!({
        "count": 42,
        "items": ["test"],
        "status": "active"
        // Missing required "name" field
    });
    
    let result = tool.call_json(params_json).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_minimal_tool_configuration() {
    let tool = minimal_tool_tool();
    
    // Should use function name when no name specified
    assert_eq!(tool.name(), "minimal_tool");
    assert_eq!(tool.description(), "Tool with minimal configuration");
}

#[tokio::test]
async fn test_registry_integration() {
    let mut registry = ToolRegistry::new();
    
    // Register multiple tools from this integration test
    registry.register_function(integration_test_tool_tool());
    registry.register_function(minimal_tool_tool());
    
    assert_eq!(registry.len(), 2);
    assert!(registry.has_tool("integration_test_tool"));
    assert!(registry.has_tool("minimal_tool"));
    
    // Get tools for genai integration
    let tools = registry.get_tools();
    assert_eq!(tools.len(), 2);
    
    // Test that schemas are properly generated
    for tool in &tools {
        assert!(tool.schema.is_some());
        let schema = tool.schema.as_ref().unwrap();
        assert!(schema.is_object());
    }
}

#[tokio::test]
async fn test_registry_execution_with_complex_params() {
    let mut registry = ToolRegistry::new();
    registry.register_function(integration_test_tool_tool());
    
    let tool_call = genai::chat::ToolCall {
        call_id: "complex-test-123".to_string(),
        fn_name: "integration_test_tool".to_string(),
        fn_arguments: json!({
            "name": "Registry Test",
            "count": 99,
            "items": ["x", "y", "z"],
            "status": "pending"
        }),
    };
    
    let response = registry.execute_call(&tool_call).await.unwrap();
    
    assert_eq!(response.call_id, "complex-test-123");
    
    let result: IntegrationResult = serde_json::from_str(&response.content).unwrap();
    assert_eq!(result.name, "Registry Test");
    assert_eq!(result.item_count, 3);
    assert_eq!(result.status_text, "Awaiting activation");
}

#[test]
fn test_schema_includes_descriptions() {
    let tool = integration_test_tool_tool();
    let schema = tool.schema();
    
    let schema_str = serde_json::to_string_pretty(&schema).unwrap();
    
    // Check that field descriptions from doc comments are included
    assert!(schema_str.contains("A required string field"));
    assert!(schema_str.contains("An optional integer field"));
    assert!(schema_str.contains("A vector of strings"));
    assert!(schema_str.contains("A nested enum"));
}

#[test]
fn test_enum_values_in_schema() {
    let tool = integration_test_tool_tool();
    let schema = tool.schema();
    
    let schema_str = serde_json::to_string(&schema).unwrap();
    
    // Check that enum values are properly included
    assert!(schema_str.contains("active"));
    assert!(schema_str.contains("inactive"));
    assert!(schema_str.contains("pending"));
}

#[tokio::test]
async fn test_concurrent_tool_execution() {
    let mut registry = ToolRegistry::new();
    registry.register_function(integration_test_tool_tool());
    
    // Create multiple tool calls to test concurrency
    let tool_calls = vec![
        genai::chat::ToolCall {
            call_id: "concurrent-1".to_string(),
            fn_name: "integration_test_tool".to_string(),
            fn_arguments: json!({
                "name": "Concurrent 1",
                "items": ["a"],
                "status": "active"
            }),
        },
        genai::chat::ToolCall {
            call_id: "concurrent-2".to_string(),
            fn_name: "integration_test_tool".to_string(),
            fn_arguments: json!({
                "name": "Concurrent 2",
                "items": ["b", "c"],
                "status": "inactive"
            }),
        },
        genai::chat::ToolCall {
            call_id: "concurrent-3".to_string(),
            fn_name: "integration_test_tool".to_string(),
            fn_arguments: json!({
                "name": "Concurrent 3",
                "items": [],
                "status": "pending"
            }),
        },
    ];
    
    let responses = registry.execute_calls(&tool_calls).await.unwrap();
    
    assert_eq!(responses.len(), 3);
    
    // Verify all responses
    for (i, response) in responses.iter().enumerate() {
        assert_eq!(response.call_id, format!("concurrent-{}", i + 1));
        
        let result: IntegrationResult = serde_json::from_str(&response.content).unwrap();
        assert_eq!(result.name, format!("Concurrent {}", i + 1));
        assert!(result.processed);
    }
} 