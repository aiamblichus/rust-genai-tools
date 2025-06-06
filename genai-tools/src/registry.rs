use crate::traits::{ToolFunction, ToolHandler};
use genai::chat::{Tool, ToolCall, ToolResponse};

use std::collections::HashMap;
use std::error::Error;

/// A registry for managing and executing tool functions.
///
/// The registry stores tool functions and provides methods to:
/// - Register functions dynamically
/// - Generate `genai::chat::Tool` definitions for LLM consumption
/// - Execute tool calls received from LLMs
///
/// # Example
///
/// ```ignore
/// use genai_tools::ToolRegistry;
/// 
/// let mut registry = ToolRegistry::new();
/// registry.register_function(my_tool_function);
/// 
/// // Get tools for LLM
/// let tools = registry.get_tools();
/// 
/// // Execute a tool call
/// let response = registry.execute_call(&tool_call).await?;
/// ```
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolHandler>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool function in the registry.
    ///
    /// The function must implement the `ToolFunction` trait, which is typically
    /// done automatically by the `#[tool_function]` macro.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut registry = ToolRegistry::new();
    /// registry.register_function(get_weather);
    /// ```
    pub fn register_function<T>(&mut self, tool: T) -> &mut Self
    where
        T: ToolFunction,
    {
        let name = tool.name().to_string();
        self.tools.insert(name, Box::new(tool));
        self
    }

    /// Register multiple tool functions at once.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut registry = ToolRegistry::new();
    /// registry.register_functions(vec![get_weather, search_web, calculate]);
    /// ```
    pub fn register_functions<T>(&mut self, tools: Vec<T>) -> &mut Self
    where
        T: ToolFunction,
    {
        for tool in tools {
            self.register_function(tool);
        }
        self
    }

    /// Get all registered tools as `genai::chat::Tool` objects.
    ///
    /// This method converts the registered tool functions into the format
    /// expected by the genai library for sending to LLMs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tools = registry.get_tools();
    /// let chat_req = ChatRequest::new(messages).with_tools(tools);
    /// ```
    pub fn get_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .map(|handler| {
                Tool::new(handler.name())
                    .with_description(handler.description())
                    .with_schema(handler.schema())
            })
            .collect()
    }

    /// Execute a tool call received from an LLM.
    ///
    /// This method takes a `ToolCall` from the LLM response, finds the
    /// corresponding registered function, executes it with the provided
    /// arguments, and returns a `ToolResponse`.
    ///
    /// # Arguments
    ///
    /// * `tool_call` - The tool call from the LLM containing the function name and arguments
    ///
    /// # Returns
    ///
    /// A `ToolResponse` containing the result of the function execution, or an error
    /// if the tool is not found or execution fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tool_calls = chat_response.into_tool_calls().unwrap();
    /// for tool_call in tool_calls {
    ///     let response = registry.execute_call(&tool_call).await?;
    ///     // Handle the response...
    /// }
    /// ```
    pub async fn execute_call(&self, tool_call: &ToolCall) -> Result<ToolResponse, Box<dyn Error + Send + Sync>> {
        let handler = self.tools
            .get(&tool_call.fn_name)
            .ok_or_else(|| format!("Tool '{}' not found in registry", tool_call.fn_name))?;

        let result = handler.call_json(tool_call.fn_arguments.clone()).await?;

        Ok(ToolResponse::new(
            tool_call.call_id.clone(),
            serde_json::to_string(&result)?,
        ))
    }

    /// Execute multiple tool calls concurrently.
    ///
    /// This is more efficient than calling `execute_call` in a loop when you have
    /// multiple tool calls to process.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tool_calls = chat_response.into_tool_calls().unwrap();
    /// let responses = registry.execute_calls(&tool_calls).await?;
    /// ```
    pub async fn execute_calls(&self, tool_calls: &[ToolCall]) -> Result<Vec<ToolResponse>, Box<dyn Error + Send + Sync>> {
        let futures: Vec<_> = tool_calls
            .iter()
            .map(|call| self.execute_call(call))
            .collect();

        futures::future::try_join_all(futures).await
    }

    /// Get the names of all registered tools.
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a tool with the given name is registered.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Remove a tool from the registry.
    ///
    /// Returns `true` if the tool was found and removed, `false` otherwise.
    pub fn remove_tool(&mut self, name: &str) -> bool {
        self.tools.remove(name).is_some()
    }

    /// Clear all tools from the registry.
    pub fn clear(&mut self) {
        self.tools.clear();
    }

    /// Merge another registry into this one.
    ///
    /// Tools from the other registry will be added to this one.
    /// If there are name conflicts, the tools from the other registry will overwrite
    /// the existing ones.
    pub fn merge(&mut self, other: ToolRegistry) -> &mut Self {
        self.tools.extend(other.tools);
        self
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tool_count", &self.tools.len())
            .field("tool_names", &self.tool_names())
            .finish()
    }
} 