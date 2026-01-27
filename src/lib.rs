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
pub use serializer::{minify, print, print_unformatted, print_buffered, print_preallocated};
pub use types::JsonNode;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests;