//! # LX-json - A Lightweight JSON Parser in Rust
//!
//! This library provides a simple, efficient JSON parser inspired by cJSON.
//! It supports all standard JSON types and provides type-safe parsing and serialization.
//!
//! ## Features
//!
//! - Parse JSON strings into Rust types
//! - Serialize Rust types back to JSON
//! - Support for all JSON types: null, boolean, number, string, array, object
//! - Type-safe API with comprehensive error handling
//! - Configurable parsing options (nesting limits, etc.)
//! - No external dependencies
//!
//! ## Example
//!
//! ```rust
//! use lx_json::{parse, JsonNode};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let json = r#"{"name": "John", "age": 30}"#;
//!     let node = parse(json)?;
//!     
//!     if let Some(name) = node.get("name").and_then(|v| v.as_string()) {
//!         println!("Name: {}", name);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod parser;
pub mod serializer;
pub mod types;

// Re-export commonly used types and functions for convenience
pub use error::{JsonError, Result};
pub use parser::{parse, parse_with_length, parse_with_opts, ParseOptions};
pub use serializer::{minify, print, print_unformatted};
pub use types::JsonNode;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_basic_parsing() {
        let json = r#"{"key": "value"}"#;
        let result = parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_serialization() {
        let node = JsonNode::Object(vec![
            ("key".to_string(), JsonNode::String("value".to_string())),
        ]);
        let output = print(&node);
        assert!(output.contains("key"));
        assert!(output.contains("value"));
    }
}