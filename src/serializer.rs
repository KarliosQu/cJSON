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

/// Serialize a JsonNode to a JSON string with pre-allocated buffer
pub fn print_buffered(node: &JsonNode, prebuffer: usize, fmt: bool) -> String {
    let mut serializer = Serializer::new(fmt, if fmt { 2 } else { 0 });
    let mut result = String::with_capacity(prebuffer);
    serializer.serialize_to_string(node, &mut result);
    result
}

/// Serialize a JsonNode to a JSON string, checking if it fits in the specified length
///
/// Returns None if the serialized string exceeds the specified length
pub fn print_preallocated(node: &JsonNode, length: usize, format: bool) -> Option<String> {
    let result = if format {
        print(node)
    } else {
        print_unformatted(node)
    };
    
    if result.len() <= length {
        Some(result)
    } else {
        None
    }
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

    fn serialize_to_string(&mut self, node: &JsonNode, result: &mut String) {
        self.serialize_node(node, 0, result);
    }

    fn serialize_node(&mut self, node: &JsonNode, indent: usize, result: &mut String) {
        match node {
            JsonNode::Null => result.push_str("null"),
            JsonNode::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
            JsonNode::Number(n) => {
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
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
                        self.add_indent(indent + 1, result);
                    }

                    self.serialize_node(item, indent + 1, result);

                    if i < arr.len() - 1 {
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
                        self.add_indent(indent + 1, result);
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

                    self.serialize_node(value, indent + 1, result);

                    if i < pairs.len() - 1 {
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
        for _ in 0..level * self.indent_size {
            result.push(' ');
        }
    }
}

/// Minify a JSON string by removing unnecessary whitespace
pub fn minify(json: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_escape = false;

    for (_i, c) in json.chars().enumerate() {
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