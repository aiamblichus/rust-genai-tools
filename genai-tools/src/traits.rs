use serde_json::Value;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

/// A trait for types that can be used as tool function parameters.
///
/// This trait is automatically implemented for types that implement both
/// `serde::de::DeserializeOwned` and `schemars::JsonSchema`.
pub trait ToolParams: serde::de::DeserializeOwned + schemars::JsonSchema + Send + 'static {}

impl<T> ToolParams for T 
where 
    T: serde::de::DeserializeOwned + schemars::JsonSchema + Send + 'static 
{}

/// A trait for types that can be returned from tool functions.
///
/// This trait is automatically implemented for types that implement `serde::Serialize`.
pub trait ToolOutput: serde::Serialize + Send + 'static {}

impl<T> ToolOutput for T 
where 
    T: serde::Serialize + Send + 'static 
{}

/// A trait for errors that can be returned from tool functions.
pub trait ToolError: Error + Send + Sync + 'static {}

impl<T> ToolError for T 
where 
    T: Error + Send + Sync + 'static 
{}

/// The core trait that defines a tool function's metadata and execution.
///
/// This trait is implemented automatically by the `#[tool_function]` macro.
pub trait ToolFunction: Send + Sync + 'static {
    /// The parameter type for this tool
    type Params: ToolParams;
    
    /// The output type for this tool
    type Output: ToolOutput;
    
    /// The error type for this tool
    type Error: ToolError;

    /// Get the name of this tool
    fn name(&self) -> &'static str;
    
    /// Get the description of this tool
    fn description(&self) -> &'static str;
    
    /// Get the JSON schema for the parameters
    fn schema(&self) -> Value {
        let schema = schemars::schema_for!(Self::Params);
        serde_json::to_value(schema).expect("Failed to serialize schema")
    }
    
    /// Execute the tool with the given parameters
    fn call(&self, params: Self::Params) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send + '_>>;
    
    /// Execute the tool with raw JSON parameters
    fn call_json(&self, params: Value) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error + Send + Sync>>> + Send + '_>> {
        Box::pin(async move {
            let parsed_params: Self::Params = serde_json::from_value(params)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                
            let result = self.call(parsed_params).await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
                
            serde_json::to_value(result)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
        })
    }
}

/// A type-erased tool function for storage in the registry
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> Value;
    fn call_json(&self, params: Value) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error + Send + Sync>>> + Send + '_>>;
}

impl<T: ToolFunction> ToolHandler for T {
    fn name(&self) -> &str {
        ToolFunction::name(self)
    }
    
    fn description(&self) -> &str {
        ToolFunction::description(self)
    }
    
    fn schema(&self) -> Value {
        ToolFunction::schema(self)
    }
    
    fn call_json(&self, params: Value) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error + Send + Sync>>> + Send + '_>> {
        ToolFunction::call_json(self, params)
    }
} 