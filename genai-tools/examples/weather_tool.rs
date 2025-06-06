use genai_tools::{tool_function, ToolRegistry};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeatherParams {
    /// The city name
    pub city: String,
    /// The country of the city
    pub country: String,
    /// Temperature unit (C for Celsius, F for Fahrenheit)
    pub unit: TemperatureUnit,
}

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub enum TemperatureUnit {
    #[serde(rename = "C")]
    Celsius,
    #[serde(rename = "F")]
    Fahrenheit,
}

#[derive(Debug, Serialize)]
pub struct WeatherResult {
    pub temperature: f64,
    pub condition: String,
    pub humidity: u32,
    pub unit: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WeatherError {
    #[error("City not found: {city}")]
    CityNotFound { city: String },
    #[error("Weather service unavailable")]
    ServiceUnavailable,
}

#[tool_function(
    name = "get_weather",
    description = "Get the current weather for a location"
)]
pub async fn get_weather(params: WeatherParams) -> Result<WeatherResult, WeatherError> {
    // Simulate API call
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Mock weather data
    let temperature = match params.unit {
        TemperatureUnit::Celsius => 22.5,
        TemperatureUnit::Fahrenheit => 72.5,
    };
    
    let unit = match params.unit {
        TemperatureUnit::Celsius => "°C",
        TemperatureUnit::Fahrenheit => "°F",
    };

    Ok(WeatherResult {
        temperature,
        condition: "Sunny".to_string(),
        humidity: 65,
        unit: unit.to_string(),
    })
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateParams {
    /// First number
    pub a: f64,
    /// Second number
    pub b: f64,
    /// Operation to perform
    pub operation: Operation,
}

#[derive(Debug, Deserialize, JsonSchema, Clone)]
pub enum Operation {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "subtract")]
    Subtract,
    #[serde(rename = "multiply")]
    Multiply,
    #[serde(rename = "divide")]
    Divide,
}

#[derive(Debug, Serialize)]
pub struct CalculateResult {
    pub result: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum CalculateError {
    #[error("Division by zero")]
    DivisionByZero,
}

#[tool_function(
    name = "calculate",
    description = "Perform basic arithmetic operations"
)]
pub async fn calculate(params: CalculateParams) -> Result<CalculateResult, CalculateError> {
    let result = match params.operation {
        Operation::Add => params.a + params.b,
        Operation::Subtract => params.a - params.b,
        Operation::Multiply => params.a * params.b,
        Operation::Divide => {
            if params.b == 0.0 {
                return Err(CalculateError::DivisionByZero);
            }
            params.a / params.b
        }
    };

    Ok(CalculateResult { result })
}

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("🛠️  GenAI Tools Example");
    
    // Create a tool registry
    let mut registry = ToolRegistry::new();
    
    // Register our tools
    registry.register_function(get_weather_tool());
    registry.register_function(calculate_tool());
    
    println!("📋 Registered {} tools:", registry.len());
    for name in registry.tool_names() {
        println!("  - {}", name);
    }
    
    // Get tools for LLM (this is what you'd pass to ChatRequest)
    let tools = registry.get_tools();
    println!("\n🔧 Generated {} tool definitions for LLM", tools.len());
    
    for tool in &tools {
        println!("  📄 Tool: {}", tool.name);
        println!("     Description: {}", tool.description.as_ref().unwrap_or(&"No description".to_string()));
        if let Some(schema) = &tool.schema {
            println!("     Schema: {}", serde_json::to_string_pretty(schema).map_err(|e| e.to_string())?);
        }
        println!();
    }
    
    // Simulate tool execution (this would normally come from LLM)
    println!("🚀 Simulating tool execution...");
    
    // Create a mock tool call
    let mock_weather_call = genai::chat::ToolCall {
        call_id: "call_123".to_string(),
        fn_name: "get_weather".to_string(),
        fn_arguments: serde_json::json!({
            "city": "Tokyo",
            "country": "Japan", 
            "unit": "C"
        }),
    };
    
    let mock_calc_call = genai::chat::ToolCall {
        call_id: "call_456".to_string(),
        fn_name: "calculate".to_string(),
        fn_arguments: serde_json::json!({
            "a": 15.5,
            "b": 7.2,
            "operation": "add"
        }),
    };
    
    // Execute the calls
    let weather_response = registry.execute_call(&mock_weather_call).await
        .map_err(|e| e.to_string())?;
    let calc_response = registry.execute_call(&mock_calc_call).await
        .map_err(|e| e.to_string())?;
    
    println!("✅ Weather response: {}", weather_response.content);
    println!("✅ Calculate response: {}", calc_response.content);
    
    println!("\n🎉 All tools working correctly!");
    
    Ok(())
} 