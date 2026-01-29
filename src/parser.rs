use crate::error::{JsonError, Result};
use crate::types::JsonNode;

const DEFAULT_NESTING_LIMIT: usize = 1000;
const DEFAULT_CIRCULAR_LIMIT: usize = 10000;

/// Parse options for JSON parsing
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Maximum nesting depth to prevent stack overflow
    pub nesting_limit: usize,
    /// Require null terminator at the end of JSON
    pub require_null_terminated: bool,
}

impl ParseOptions {
    /// Create default parse options
    pub fn new() -> Self {
        ParseOptions {
            nesting_limit: DEFAULT_NESTING_LIMIT,
            require_null_terminated: false,
        }
    }

    /// Set the nesting limit
    pub fn with_nesting_limit(mut self, limit: usize) -> Self {
        self.nesting_limit = limit;
        self
    }

    /// Set whether null terminator is required
    pub fn with_null_terminated(mut self, required: bool) -> Self {
        self.require_null_terminated = required;
        self
    }
}

/// JSON Parser
pub struct Parser<'a> {
    input: &'a str,
    position: usize,
    options: ParseOptions,
    current_depth: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser with the given input and options
    fn new(input: &'a str, options: ParseOptions) -> Self {
        Parser {
            input,
            position: 0,
            options,
            current_depth: 0,
        }
    }

    /// Get the current character
    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    /// Consume the current character and advance
    fn consume(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.position += c.unwrap().len_utf8();
        }
        c
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    /// Check if we've reached the end of input
    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Expect a specific character
    fn expect_char(&mut self, expected: char) -> Result<()> {
        self.skip_whitespace();
        if let Some(c) = self.peek() {
            if c == expected {
                self.consume();
                return Ok(());
            }
            return Err(JsonError::UnexpectedCharacter {
                character: c,
                position: self.position,
                expected: expected.to_string(),
            });
        }
        Err(JsonError::UnexpectedEndOfInput {
            position: self.position,
            expected: expected.to_string(),
        })
    }

    /// Parse a null value
    fn parse_null(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();
        
        let remaining = &self.input[self.position..];
        if remaining.starts_with("null") {
            self.position += 4;
            Ok(JsonNode::Null)
        } else {
            let c = self.peek().unwrap_or('\0');
            Err(JsonError::UnexpectedCharacter {
                character: c,
                position: self.position,
                expected: "null".to_string(),
            })
        }
    }

    /// Parse a boolean value (true or false)
    fn parse_bool(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();
        
        let remaining = &self.input[self.position..];
        if remaining.starts_with("true") {
            self.position += 4;
            Ok(JsonNode::Bool(true))
        } else if remaining.starts_with("false") {
            self.position += 5;
            Ok(JsonNode::Bool(false))
        } else {
            let c = self.peek().unwrap_or('\0');
            Err(JsonError::UnexpectedCharacter {
                character: c,
                position: self.position,
                expected: "true or false".to_string(),
            })
        }
    }

    /// Parse a number
    fn parse_number(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();
        let start = self.position;

        // Optional minus sign
        if let Some('-') = self.peek() {
            self.consume();
        }

        // Integer part (at least one digit)
        if let Some('0') = self.peek() {
            self.consume();
        } else if let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.consume();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        self.consume();
                    } else {
                        break;
                    }
                }
            } else {
                return Err(JsonError::UnexpectedCharacter {
                    character: c,
                    position: self.position,
                    expected: "digit".to_string(),
                });
            }
        } else {
            return Err(JsonError::UnexpectedEndOfInput {
                position: self.position,
                expected: "digit".to_string(),
            });
        }

        // Fractional part
        if let Some('.') = self.peek() {
            self.consume();
            let mut has_digit = false;
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.consume();
                    has_digit = true;
                } else {
                    break;
                }
            }
            if !has_digit {
                return Err(JsonError::InvalidNumber {
                    position: self.position - 1,
                    reason: "Expected digits after decimal point".to_string(),
                });
            }
        }

        // Exponent part
        if let Some(c) = self.peek() {
            if c == 'e' || c == 'E' {
                self.consume();
                // Optional sign
                if let Some(c) = self.peek() {
                    if c == '+' || c == '-' {
                        self.consume();
                    }
                }
                // At least one digit
                let mut has_digit = false;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        self.consume();
                        has_digit = true;
                    } else {
                        break;
                    }
                }
                if !has_digit {
                    return Err(JsonError::InvalidNumber {
                        position: self.position,
                        reason: "Expected digits in exponent".to_string(),
                    });
                }
            }
        }

        let num_str = &self.input[start..self.position];
        match num_str.parse::<f64>() {
            Ok(n) => Ok(JsonNode::Number(n)),
            Err(_) => Err(JsonError::InvalidNumber {
                position: start,
                reason: format!("Failed to parse number: {}", num_str),
            }),
        }
    }

    /// Parse a string
    fn parse_string(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();
        self.expect_char('"')?;

        let mut result = String::new();
        let _start = self.position;

        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.consume();
                    return Ok(JsonNode::String(result));
                }
                '\\' => {
                    self.consume();
                    result.push(self.parse_escape_sequence()?);
                }
                '\0'..='\u{001f}' => {
                    return Err(JsonError::InvalidString {
                        position: self.position,
                        reason: "Control character in string must be escaped".to_string(),
                    });
                }
                _ => {
                    self.consume();
                    result.push(c);
                }
            }
        }

        Err(JsonError::UnexpectedEndOfInput {
            position: self.position,
            expected: "\"".to_string(),
        })
    }

    /// Parse an escape sequence
    fn parse_escape_sequence(&mut self) -> Result<char> {
        let c = self.peek().ok_or(JsonError::UnexpectedEndOfInput {
            position: self.position,
            expected: "escape character".to_string(),
        })?;

        let escaped = match c {
            '"' => '"',
            '\\' => '\\',
            '/' => '/',
            'b' => '\x08',
            'f' => '\x0c',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => {
                self.consume();
                return self.parse_unicode_escape();
            }
            _ => {
                return Err(JsonError::InvalidEscapeSequence {
                    position: self.position,
                    sequence: c.to_string(),
                });
            }
        };

        self.consume();
        Ok(escaped)
    }

    /// Parse a Unicode escape sequence (\uXXXX)
    fn parse_unicode_escape(&mut self) -> Result<char> {
        let start = self.position;
        let mut code_point = 0u32;

        for _ in 0..4 {
            let c = self.peek().ok_or(JsonError::UnexpectedEndOfInput {
                position: self.position,
                expected: "hex digit".to_string(),
            })?;

            let digit = c.to_digit(16).ok_or(JsonError::InvalidEscapeSequence {
                position: self.position,
                sequence: format!("\\u{}", &self.input[start..self.position + 1]),
            })?;

            code_point = (code_point << 4) | digit;
            self.consume();
        }

        char::from_u32(code_point).ok_or(JsonError::InvalidEscapeSequence {
            position: start,
            sequence: format!("\\u{:04x}", code_point),
        })
    }

    /// Parse an array
    fn parse_array(&mut self) -> Result<JsonNode> {
        self.check_nesting_limit()?;
        self.current_depth += 1;

        self.expect_char('[')?;

        let mut items = Vec::new();
        let mut has_value = false;

        loop {
            self.skip_whitespace();

            if let Some(']') = self.peek() {
                self.consume();
                break;
            }

            if has_value {
                self.expect_char(',')?;
                self.skip_whitespace();
                
                // Check for trailing comma
                if let Some(']') = self.peek() {
                    return Err(JsonError::TrailingComma {
                        position: self.position,
                    });
                }
            }

            let value = self.parse_value()?;
            items.push(value);
            has_value = true;
        }

        self.current_depth -= 1;
        Ok(JsonNode::Array(items))
    }

    /// Parse an object
    fn parse_object(&mut self) -> Result<JsonNode> {
        self.check_nesting_limit()?;
        self.current_depth += 1;

        self.expect_char('{')?;

        let mut members = Vec::new();
        let mut has_value = false;

        loop {
            self.skip_whitespace();

            if let Some('}') = self.peek() {
                self.consume();
                break;
            }

            if has_value {
                self.expect_char(',')?;
                self.skip_whitespace();
                
                // Check for trailing comma
                if let Some('}') = self.peek() {
                    return Err(JsonError::TrailingComma {
                        position: self.position,
                    });
                }
            }

            // Parse key (must be a string)
            let key = match self.parse_value()? {
                JsonNode::String(s) => s,
                _ => {
                    return Err(JsonError::UnexpectedCharacter {
                        character: self.peek().unwrap_or('\0'),
                        position: self.position,
                        expected: "string key".to_string(),
                    });
                }
            };

            self.skip_whitespace();
            self.expect_char(':')?;

            let value = self.parse_value()?;

            // Check for duplicate keys
            if members.iter().any(|(k, _)| k == &key) {
                return Err(JsonError::DuplicateKey {
                    key: key.clone(),
                    position: self.position,
                });
            }

            members.push((key, value));
            has_value = true;
        }

        self.current_depth -= 1;
        Ok(JsonNode::Object(members))
    }

    /// Check nesting limit
    fn check_nesting_limit(&self) -> Result<()> {
        if self.current_depth >= self.options.nesting_limit {
            return Err(JsonError::NestingLimitExceeded {
                position: self.position,
                limit: self.options.nesting_limit,
            });
        }
        Ok(())
    }

    /// Parse a JSON value
    fn parse_value(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();

        let c = self.peek().ok_or(JsonError::UnexpectedEndOfInput {
            position: self.position,
            expected: "value".to_string(),
        })?;

        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '-' | '0'..='9' => self.parse_number(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            _ => Err(JsonError::UnexpectedCharacter {
                character: c,
                position: self.position,
                expected: "value".to_string(),
            }),
        }
    }

    /// Parse the complete JSON document
    fn parse_document(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();

        if self.is_eof() {
            return Err(JsonError::UnexpectedEndOfInput {
                position: self.position,
                expected: "value".to_string(),
            });
        }

        let value = self.parse_value()?;

        // Skip trailing whitespace
        self.skip_whitespace();

        // Check if we've consumed the entire input
        if !self.is_eof() {
            if self.options.require_null_terminated {
                return Err(JsonError::ExpectedNullTerminator {
                    position: self.position,
                });
            }
            // When require_null_terminated is false, we allow trailing characters
            // Just return the parsed value without checking for extra content
        }

        Ok(value)
    }
}

/// Parse a JSON string into a JsonNode
pub fn parse(json: &str) -> Result<JsonNode> {
    parse_with_opts(json, ParseOptions::new())
}

/// Parse a JSON string with a maximum length
pub fn parse_with_length(json: &str, length: usize) -> Result<JsonNode> {
    let truncated = if json.len() > length {
        &json[..length]
    } else {
        json
    };
    parse(truncated)
}

/// Parse a JSON string with custom options
pub fn parse_with_opts(json: &str, opts: ParseOptions) -> Result<JsonNode> {
    let mut parser = Parser::new(json, opts);
    parser.parse_document()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let result = parse("null");
        assert_eq!(result, Ok(JsonNode::Null));
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true"), Ok(JsonNode::Bool(true)));
        assert_eq!(parse("false"), Ok(JsonNode::Bool(false)));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse("42"), Ok(JsonNode::Number(42.0)));
        assert_eq!(parse("-42"), Ok(JsonNode::Number(-42.0)));
        assert_eq!(parse("3.14"), Ok(JsonNode::Number(3.14)));
        assert_eq!(parse("-3.14"), Ok(JsonNode::Number(-3.14)));
        assert_eq!(parse("1e5"), Ok(JsonNode::Number(100000.0)));
        assert_eq!(parse("1.5e-3"), Ok(JsonNode::Number(0.0015)));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse(r#""hello""#), Ok(JsonNode::String("hello".to_string())));
        assert_eq!(parse(r#""world""#), Ok(JsonNode::String("world".to_string())));
    }

    #[test]
    fn test_parse_string_with_escapes() {
        assert_eq!(parse(r#""hello\nworld""#), Ok(JsonNode::String("hello\nworld".to_string())));
        assert_eq!(parse(r#""\"quoted\"""#), Ok(JsonNode::String("\"quoted\"".to_string())));
        assert_eq!(parse(r#""\\t\\\\""#), Ok(JsonNode::String("\t\\".to_string())));
    }

    #[test]
    fn test_parse_array() {
        let result = parse("[]");
        assert_eq!(result, Ok(JsonNode::Array(vec![])));

        let result = parse("[1, 2, 3]");
        assert_eq!(
            result,
            Ok(JsonNode::Array(vec![
                JsonNode::Number(1.0),
                JsonNode::Number(2.0),
                JsonNode::Number(3.0),
            ]))
        );
    }

    #[test]
    fn test_parse_nested_array() {
        let result = parse("[[1], [2], [3]]");
        assert_eq!(
            result,
            Ok(JsonNode::Array(vec![
                JsonNode::Array(vec![JsonNode::Number(1.0)]),
                JsonNode::Array(vec![JsonNode::Number(2.0)]),
                JsonNode::Array(vec![JsonNode::Number(3.0)]),
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let result = parse("{}");
        assert_eq!(result, Ok(JsonNode::Object(vec![])));

        let result = parse(r#"{"key": "value"}"#);
        assert_eq!(
            result,
            Ok(JsonNode::Object(vec![(
                "key".to_string(),
                JsonNode::String("value".to_string()),
            )]))
        );
    }

    #[test]
    fn test_parse_nested_object() {
        let result = parse(r#"{"outer": {"inner": "value"}}"#);
        assert_eq!(
            result,
            Ok(JsonNode::Object(vec![(
                "outer".to_string(),
                JsonNode::Object(vec![(
                    "inner".to_string(),
                    JsonNode::String("value".to_string()),
                )]),
            )]))
        );
    }

    #[test]
    fn test_parse_complex_json() {
        let json = r#"{
            "name": "John",
            "age": 30,
            "isStudent": false,
            "hobbies": ["reading", "gaming"],
            "address": {
                "city": "New York",
                "country": "USA"
            }
        }"#;

        let result = parse(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_length() {
        let result = parse_with_length(r#"{"key": "value"}"#, 10);
        assert!(result.is_err()); // Truncated JSON should fail
    }

    #[test]
    fn test_parse_with_options() {
        // 要求 null 终止符
        let opts = ParseOptions::new().with_null_terminated(true);
        let result = parse_with_opts("null extra", opts);
        assert!(result.is_err());

        // 不要求 null 终止符
        let opts = ParseOptions::new().with_null_terminated(false);
        let result = parse_with_opts("null extra", opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        assert!(parse("{").is_err());
        assert!(parse("[").is_err());
        assert!(parse(r#""unclosed string"#).is_err());
        assert!(parse("123abc").is_err());
    }

    #[test]
    fn test_trailing_comma() {
        assert!(parse("[1, 2,]").is_err());
        assert!(parse(r#"{"a": 1,}"#).is_err());
    }

    #[test]
    fn test_nesting_limit() {
        let opts = ParseOptions::new().with_nesting_limit(2);
        let deep_json = "[[[1]]]"; // Nesting depth 3
        let result = parse_with_opts(deep_json, opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(parse("  null  "), Ok(JsonNode::Null));
        assert_eq!(parse("\ntrue\n"), Ok(JsonNode::Bool(true)));
        assert_eq!(parse("\t42\t"), Ok(JsonNode::Number(42.0)));
    }

    #[test]
    fn test_empty_input() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn test_unicode_escape() {
        let result = parse(r#""\u0041""#);
        assert_eq!(result, Ok(JsonNode::String("A".to_string())));
    }

    #[test]
    fn test_duplicate_key() {
        let result = parse(r#"{"key": 1, "key": 2}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        assert!(parse("{").is_err()); // 未闭合的对象
        assert!(parse("[").is_err()); // 未闭合的数组
        assert!(parse(r#""unclosed string"#).is_err()); // 未闭合的字符串
        assert!(parse("123abc").is_err()); // 无效的数字后缀
    }
}