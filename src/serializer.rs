use crate::types::JsonNode;

/// Serialize a JsonNode to a formatted JSON string with indentation
pub fn print(node: &JsonNode) -> String {
    let mut serializer = Serializer::new(true, 2);
    serializer.serialize(node)
}

/// Serialize a JsonNode to a compact JSON string (no whitespace)
pub fn print_unformatted(node: &JsonNode) -> String {
    let mut serializer = Serializer::new(false, 0);
    serializer.serialize(node)
}

/// JSON Serializer
struct Serializer {
    formatted: bool,
    indent_size: usize,
}

impl Serializer {
    fn new(formatted: bool, indent_size: usize) -> Self {
        Serializer {
            formatted,
            indent_size,
        }
    }

    fn serialize(&mut self, node: &JsonNode) -> String {
        let mut result = String::new();
        self.serialize_node(node, 0, &mut result);
        result
    }

    fn serialize_node(&mut self, node: &JsonNode, indent: usize, result: &mut String) {
        match node {
            JsonNode::Null => result.push_str("null"),
            JsonNode::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
            JsonNode::Number(n) => {
                if n.fract() == 0.0_f64 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    result.push_str(&(*n as i64).to_string());
                } else {
                    result.push_str(&n.to_string());
                }
            }
            JsonNode::String(s) => {
                result.push('"');
                for c in s.chars() {
                    match c {
                        '"' => result.push_str("\\\""),
                        '\\' => result.push_str("\\\\"),
                        '\x08' => result.push_str("\\b"),
                        '\x0c' => result.push_str("\\f"),
                        '\n' => result.push_str("\\n"),
                        '\r' => result.push_str("\\r"),
                        '\t' => result.push_str("\\t"),
                        '\0'..='\u{001f}' => {
                            result.push_str(&format!("\\u{:04x}", c as u32));
                        }
                        _ => result.push(c),
                    }
                }
                result.push('"');
            }
            JsonNode::Array(arr) => {
                result.push('[');
                if arr.is_empty() {
                    result.push(']');
                    return;
                }

                if self.formatted {
                    result.push('\n');
                }

                for (i, item) in arr.iter().enumerate() {
                    if self.formatted {
                        self.add_indent(indent.saturating_add(1), result);
                    }

                    self.serialize_node(item, indent.saturating_add(1), result);

                    if i.checked_add(1).is_some_and(|v| v < arr.len()) {
                        result.push(',');
                    }

                    if self.formatted {
                        result.push('\n');
                    }
                }

                if self.formatted {
                    self.add_indent(indent, result);
                }
                result.push(']');
            }
            JsonNode::Object(pairs) => {
                result.push('{');
                if pairs.is_empty() {
                    result.push('}');
                    return;
                }

                if self.formatted {
                    result.push('\n');
                }

                for (i, (key, value)) in pairs.iter().enumerate() {
                    if self.formatted {
                        self.add_indent(indent.saturating_add(1), result);
                    }

                    // Serialize key
                    result.push('"');
                    for c in key.chars() {
                        match c {
                            '"' => result.push_str("\\\""),
                            '\\' => result.push_str("\\\\"),
                            '\x08' => result.push_str("\\b"),
                            '\x0c' => result.push_str("\\f"),
                            '\n' => result.push_str("\\n"),
                            '\r' => result.push_str("\\r"),
                            '\t' => result.push_str("\\t"),
                            '\0'..='\u{001f}' => {
                                result.push_str(&format!("\\u{:04x}", c as u32));
                            }
                            _ => result.push(c),
                        }
                    }
                    result.push('"');

                    if self.formatted {
                        result.push_str(": ");
                    } else {
                        result.push(':');
                    }

                    self.serialize_node(value, indent.saturating_add(1), result);

                    if i.checked_add(1).is_some_and(|v| v < pairs.len()) {
                        result.push(',');
                    }

                    if self.formatted {
                        result.push('\n');
                    }
                }

                if self.formatted {
                    self.add_indent(indent, result);
                }
                result.push('}');
            }
            JsonNode::Raw(s) => result.push_str(s),
        }
    }

    fn add_indent(&self, level: usize, result: &mut String) {
        result.push_str(&" ".repeat(level.saturating_mul(self.indent_size)));
    }
}

/// Minify a JSON string by removing unnecessary whitespace
///
/// # Errors
///
/// Returns an error if the JSON string contains an unclosed string.
pub fn minify(json: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_escape = false;

    for c in json.chars() {
        match c {
            '"' if !in_escape => {
                in_string = !in_string;
                result.push(c);
            }
            '\\' if in_string => {
                in_escape = true;
                result.push(c);
            }
            _ if in_string => {
                in_escape = false;
                result.push(c);
            }
            c if c.is_ascii_whitespace() => {
                // Skip whitespace outside strings
            }
            _ => {
                result.push(c);
            }
        }
    }

    if in_string {
        return Err(format!("Unclosed string at position {}", json.len()));
    }

    Ok(result)
}

/// Serialize a JsonNode to a JSON string with pre-allocated buffer capacity
/// 
/// # Arguments
/// * `node` - The JsonNode to serialize
/// * `prebuffer` - The initial buffer capacity to allocate
/// * `fmt` - Whether to format the output with indentation
/// 
/// # Returns
/// A String containing the serialized JSON
pub fn print_buffered(node: &JsonNode, prebuffer: usize, fmt: bool) -> String {
    let mut serializer = Serializer::new(fmt, 2);
    let mut result = String::with_capacity(prebuffer);
    serializer.serialize_node(node, 0, &mut result);
    result
}

/// Serialize a JsonNode into a pre-allocated buffer
///
/// # Arguments
/// * `node` - The JsonNode to serialize
/// * `buffer` - A mutable String buffer to write the result into
/// * `fmt` - Whether to format the output with indentation
///
/// # Returns
/// Ok(()) if successful, Err(JsonError) if the buffer is too small
///
/// # Errors
///
/// This function currently always returns Ok(()) as String automatically grows.
pub fn print_preallocated(node: &JsonNode, buffer: &mut String, fmt: bool) -> Result<(), crate::JsonError> {
    let mut serializer = Serializer::new(fmt, 2);
    buffer.clear();
    serializer.serialize_node(node, 0, buffer);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_null() {
        let node = JsonNode::Null;
        assert_eq!(print(&node), "null");
        assert_eq!(print_unformatted(&node), "null");
    }

    #[test]
    fn test_print_bool() {
        assert_eq!(print(&JsonNode::Bool(true)), "true");
        assert_eq!(print(&JsonNode::Bool(false)), "false");
    }

    #[test]
    fn test_print_number() {
        assert_eq!(print(&JsonNode::Number(42.0)), "42");
        assert_eq!(print(&JsonNode::Number(3.14)), "3.14");
        assert_eq!(print(&JsonNode::Number(-5.5)), "-5.5");
    }

    #[test]
    fn test_print_string() {
        assert_eq!(print(&JsonNode::String("hello".to_string())), r#""hello""#);
        assert_eq!(print(&JsonNode::String("hello\nworld".to_string())), r#""hello\nworld""#);
    }

    #[test]
    fn test_print_array() {
        let arr = JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::Number(2.0),
            JsonNode::Number(3.0),
        ]);
        let formatted = print(&arr);
        assert!(formatted.contains("["));
        assert!(formatted.contains("1"));
        assert!(formatted.contains("2"));
        assert!(formatted.contains("3"));

        let unformatted = print_unformatted(&arr);
        assert_eq!(unformatted, "[1,2,3]");
    }

    #[test]
    fn test_print_object() {
        let obj = JsonNode::Object(vec![
            ("key".to_string(), JsonNode::String("value".to_string())),
        ]);
        let formatted = print(&obj);
        assert!(formatted.contains("{"));
        assert!(formatted.contains("key"));
        assert!(formatted.contains("value"));

        let unformatted = print_unformatted(&obj);
        assert_eq!(unformatted, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_minify() {
        let json = r#"{
            "name": "John",
            "age": 30
        }"#;
        let minified = minify(json).unwrap();
        assert_eq!(minified, r#"{"name":"John","age":30}"#);
    }

    #[test]
    fn test_minify_preserves_string_whitespace() {
        let json = r#"{"name": "John Doe"}"#;
        let minified = minify(json).unwrap();
        assert_eq!(minified, r#"{"name":"John Doe"}"#);
    }

    #[test]
    fn test_minify_invalid() {
        let json = r#"{"unclosed": "string}"#;
        let result = minify(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip() {
        let original = r#"{"name": "John", "age": 30, "active": true}"#;
        let parsed = crate::parser::parse(original).unwrap();
        let serialized = print_unformatted(&parsed);
        let reparsed = crate::parser::parse(&serialized).unwrap();
        assert_eq!(parsed, reparsed);
    }
}