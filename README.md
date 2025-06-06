# genai-tools

**Type-safe tool definition macros for the genai crate.**

This crate provides ergonomic macros and utilities for defining LLM tools with automatic JSON schema generation from Rust types, eliminating the need for manual schema definition and parameter extraction.

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
genai = "0.3"
genai-tools = "0.1"
serde = { version = "1.0", features = ["derive"] }
schemars = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["macros"] }
```

## ✨ Features

- **Type-safe**: Compile-time checking of tool parameters
- **Automatic schema generation**: Uses `schemars` to generate JSON schemas from Rust types
- **Zero boilerplate**: Define tools with a simple attribute macro
- **Runtime flexibility**: Dynamic tool registration and execution
- **Seamless integration**: Works with existing `genai::chat::Tool` API

## 📖 Usage

### Define a Tool Function

```rust
use genai_tools::{tool_function, ToolRegistry};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
pub struct WeatherParams {
    /// The city name
    pub city: String,
    /// The country of the city
    pub country: String,
    /// Temperature unit
    pub unit: TemperatureUnit,
}

#[derive(Deserialize, JsonSchema)]
pub enum TemperatureUnit {
    #[serde(rename = "C")]
    Celsius,
    #[serde(rename = "F")]
    Fahrenheit,
}

#[derive(Serialize)]
pub struct WeatherResult {
    pub temperature: f64,
    pub condition: String,
}

#[tool_function(
    name = "get_weather",
    description = "Get the current weather for a location"
)]
pub async fn get_weather(params: WeatherParams) -> Result<WeatherResult, WeatherError> {
    // Your implementation here
    Ok(WeatherResult {
        temperature: 22.5,
        condition: "Sunny".to_string(),
    })
}
```

### Register and Use Tools

```rust
use genai::{Client, chat::{ChatMessage, ChatRequest}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry and register tools
    let mut registry = ToolRegistry::new();
    registry.register_function(get_weather_tool());
    
    // Use with genai
    let client = Client::default();
    let tools = registry.get_tools(); // Convert to genai::chat::Tool
    
    let chat_req = ChatRequest::new(vec![
        ChatMessage::user("What's the weather like in Tokyo?")
    ]).with_tools(tools);
    
    let chat_res = client.exec_chat("gpt-4", chat_req, None).await?;
    
    // Execute tool calls
    if let Some(tool_calls) = chat_res.into_tool_calls() {
        for tool_call in tool_calls {
            let response = registry.execute_call(&tool_call).await?;
            println!("Tool response: {}", response.content);
        }
    }
    
    Ok(())
}
```

## 🔄 Before vs After

### Before (Manual approach)

```rust
// 25+ lines of manual schema definition
let weather_tool = Tool::new("get_weather")
    .with_description("Get the current weather for a location")
    .with_schema(json!({
        "type": "object",
        "properties": {
            "city": {
                "type": "string",
                "description": "The city name"
            },
            "country": {
                "type": "string", 
                "description": "The country of the city"
            },
            "unit": {
                "type": "string",
                "enum": ["C", "F"],
                "description": "Temperature unit"
            }
        },
        "required": ["city", "country", "unit"]
    }));

// Manual parameter extraction with runtime errors
let args = tool_call.fn_arguments;
let city = args["city"].as_str().unwrap(); // ❌ Can panic
let country = args["country"].as_str().unwrap(); // ❌ Can panic
let unit = args["unit"].as_str().unwrap(); // ❌ Can panic
```

### After (genai-tools approach)

```rust
// 10 lines with full type safety
#[derive(Deserialize, JsonSchema)]
pub struct WeatherParams {
    /// The city name
    pub city: String,
    /// The country of the city
    pub country: String,
    /// Temperature unit
    pub unit: TemperatureUnit,
}

#[tool_function(name = "get_weather", description = "Get the current weather")]
pub async fn get_weather(params: WeatherParams) -> Result<WeatherResult, Error> {
    // ✅ Type-safe parameters, automatic validation
    // ✅ Schema generated automatically
    // ✅ Compile-time guarantees
}
```

## 🛠️ Advanced Usage

### Multiple Tools

```rust
let mut registry = ToolRegistry::new();

registry
    .register_function(get_weather_tool())
    .register_function(calculate_tool())
    .register_function(search_web_tool());
```

### Conditional Registration

```rust
let mut registry = ToolRegistry::new();

registry.register_function(get_weather_tool());

if user_has_premium {
    registry.register_function(advanced_search_tool());
}

if config.enable_calculator {
    registry.register_function(calculator_tool());
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum WeatherError {
    #[error("City not found: {city}")]
    CityNotFound { city: String },
    #[error("Weather service unavailable")]
    ServiceUnavailable,
}

#[tool_function(name = "get_weather", description = "Get weather")]
pub async fn get_weather(params: WeatherParams) -> Result<WeatherResult, WeatherError> {
    if !is_valid_city(&params.city) {
        return Err(WeatherError::CityNotFound { 
            city: params.city 
        });
    }
    // ... rest of implementation
}
```

## 📚 Examples

Run the examples to see the crate in action:

```bash
# Basic weather tool example
cargo run --example weather_tool

# Comparison between old and new approaches
cargo run --example comparison
```

## 🔧 Requirements

Tool functions must:

- Be `async`
- Take exactly one parameter implementing `serde::de::DeserializeOwned + schemars::JsonSchema`
- Return `Result<T, E>` where:
  - `T: serde::Serialize` 
  - `E: std::error::Error + Send + Sync`

Parameter types must implement:
- `serde::Deserialize` for JSON parsing
- `schemars::JsonSchema` for automatic schema generation

## 🏗️ Architecture

This crate consists of two parts:

1. **`genai-tools-macros`**: Proc macro crate that generates the boilerplate
2. **`genai-tools`**: Runtime crate with `ToolRegistry` and traits

The `#[tool_function]` macro generates:
- A struct implementing `ToolFunction` 
- Automatic schema generation using `schemars`
- Type-safe parameter handling
- Integration with the `ToolRegistry`

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license. 