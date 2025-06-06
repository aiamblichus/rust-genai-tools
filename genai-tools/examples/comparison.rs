use genai::chat::Tool;
use genai_tools::{tool_function, ToolRegistry};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use serde_json::json;

// ===== NEW APPROACH WITH GENAI-TOOLS =====

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeatherParams {
    /// The city name
    pub city: String,
    /// The country of the city  
    pub country: String,
}

#[derive(Debug, Serialize)]
pub struct WeatherResult {
    pub temperature: f64,
    pub condition: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WeatherError {
    #[error("Weather service error")]
    ServiceError,
}

#[tool_function(
    name = "get_weather",
    description = "Get the current weather for a location"
)]
pub async fn get_weather(_params: WeatherParams) -> Result<WeatherResult, WeatherError> {
    // Simulate API call
    Ok(WeatherResult {
        temperature: 22.5,
        condition: "Sunny".to_string(),
    })
}

// ===== OLD APPROACH (MANUAL) =====

fn create_weather_tool_manually() -> Tool {
    Tool::new("get_weather")
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
                }
            },
            "required": ["city", "country"]
        }))
}

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("🔄 Tool Definition Comparison Example\n");
    
    // ===== NEW APPROACH =====
    println!("✨ NEW APPROACH (with genai-tools):");
    println!("1. Define params struct with derive macros");
    println!("2. Write async function with #[tool_function] attribute");
    println!("3. Register with one line: registry.register_function(get_weather_tool())");
    
    let mut registry = ToolRegistry::new();
    registry.register_function(get_weather_tool());
    
    let new_tools = registry.get_tools();
    println!("   Generated {} tool(s) automatically\n", new_tools.len());
    
    // ===== OLD APPROACH =====
    println!("🔧 OLD APPROACH (manual):");
    println!("1. Manually write JSON schema in json!() macro");
    println!("2. Manually extract and validate parameters");
    println!("3. Handle all error cases manually");
    
    let _manual_tool = create_weather_tool_manually();
    println!("   Created 1 tool manually\n");
    
    // ===== COMPARISON =====
    println!("📊 COMPARISON:");
    println!("   Lines of code:");
    println!("   - New approach: ~15 lines (struct + function)");
    println!("   - Old approach: ~25+ lines (schema + extraction + validation)");
    println!("   ");
    println!("   Type safety:");
    println!("   - New approach: ✅ Compile-time type checking");
    println!("   - Old approach: ❌ Runtime parsing with unwrap()");
    println!("   ");
    println!("   Schema maintenance:");
    println!("   - New approach: ✅ Automatic from Rust types");
    println!("   - Old approach: ❌ Manual sync required");
    println!("   ");
    println!("   Error handling:");
    println!("   - New approach: ✅ Automatic serde validation");
    println!("   - Old approach: ❌ Manual validation required");
    
    println!("\n🎯 The new approach provides:");
    println!("   • Type safety at compile time");
    println!("   • Automatic schema generation");
    println!("   • Less boilerplate code");
    println!("   • Easier testing and maintenance");
    println!("   • DRY principle - single source of truth");
    
    Ok(())
} 