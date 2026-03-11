use crate::error::{JsonError, Result};
use crate::types::JsonNode;
use std::iter::Peekable;
use std::str::Chars;

/// Parser configuration options
#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    /// Maximum nesting depth for JSON structures
    pub nesting_limit: usize,
    /// Whether to require null termination
    pub require_null_terminated: bool,
}

impl ParseOptions {
    /// Create new parse options with defaults
    pub fn new() -> Self {
        Self {
            nesting_limit: 1000,
            require_null_terminated: false,
        }
    }
}

/// JSON parser
#[derive(Debug)]
pub struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
    position: usize,
    options: ParseOptions,
    current_depth: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser
    pub fn new(input: &'a str, options: ParseOptions) -> Self {
        Self {
            chars: input.chars().peekable(),
            position: 0,
            options,
            current_depth: 0,
        }
    }

    /// Check if we're at end of input
    fn is_eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    /// Consume the current character
    fn consume(&mut self) -> Option<char> {
        let c = self.chars.next();
        if c.is_some() {
            self.position = self.position.wrapping_add(1);
        }
        c
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    /// Parse a value
    fn parse_value(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();

        match self.chars.peek() {
            Some(&'n') => self.parse_null(),
            Some(&'t') => self.parse_true(),
            Some(&'f') => self.parse_false(),
            Some(&'"') => self.parse_string(),
            Some(&c) if c.is_ascii_digit() || c == '-' => self.parse_number(),
            Some(&'[') => self.parse_array(),
            Some(&'{') => self.parse_object(),
            Some(&c) => Err(JsonError::UnexpectedCharacter {
                character: c,
                position: self.position,
                expected: "value".to_string(),
            }),
            None => Err(JsonError::UnexpectedEndOfInput {
                position: self.position,
                expected: "value".to_string(),
            }),
        }
    }

    /// Parse null
    fn parse_null(&mut self) -> Result<JsonNode> {
        self.expect_literal("null")?;
        Ok(JsonNode::Null)
    }

    /// Parse true
    fn parse_true(&mut self) -> Result<JsonNode> {
        self.expect_literal("true")?;
        Ok(JsonNode::Bool(true))
    }

    /// Parse false
    fn parse_false(&mut self) -> Result<JsonNode> {
        self.expect_literal("false")?;
        Ok(JsonNode::Bool(false))
    }

    /// Expect a specific literal
    fn expect_literal(&mut self, literal: &str) -> Result<()> {
        for expected in literal.chars() {
            match self.consume() {
                Some(c) if c == expected => continue,
                Some(c) => {
                    return Err(JsonError::UnexpectedCharacter {
                        character: c,
                        position: self.position.wrapping_sub(1),
                        expected: literal.to_string(),
                    })
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        position: self.position,
                        expected: literal.to_string(),
                    })
                }
            }
        }
        Ok(())
    }

    /// Parse a string
    fn parse_string(&mut self) -> Result<JsonNode> {
        self.consume(); // Skip opening quote

        let mut result = String::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                '"' => {
                    self.consume();
                    return Ok(JsonNode::String(result));
                }
                '\\' => {
                    self.consume();
                    let escaped = self.parse_escape()?;
                    result.push(escaped);
                }
                _ => {
                    result.push(c);
                    self.consume();
                }
            }
        }

        Err(JsonError::InvalidString {
            position: self.position,
            reason: "Unterminated string".to_string(),
        })
    }

    /// Parse an escape sequence
    fn parse_escape(&mut self) -> Result<char> {
        match self.consume() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\x08'),
            Some('f') => Ok('\x0c'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => self.parse_unicode_escape(),
            Some(c) => Err(JsonError::InvalidEscapeSequence {
                position: self.position.wrapping_sub(1),
                sequence: c.to_string(),
            }),
            None => Err(JsonError::InvalidEscapeSequence {
                position: self.position,
                sequence: "Unexpected end of input".to_string(),
            }),
        }
    }

    /// Parse a Unicode escape sequence
    fn parse_unicode_escape(&mut self) -> Result<char> {
        let mut code_point = 0u32;

        for _ in 0_i32..4_i32 {
            let digit = self.consume().ok_or(JsonError::InvalidString {
                position: self.position,
                reason: "Unexpected end of input in unicode escape".to_string(),
            })?;

            let value = digit.to_digit(16).ok_or(JsonError::InvalidString {
                position: self.position.wrapping_sub(1),
                reason: format!("Invalid hex digit '{}'", digit),
            })?;

            code_point = (code_point << 4_i32) | value;
        }

        // Handle UTF-16 surrogate pairs
        if (0xD800..=0xDBFF).contains(&code_point) {
            let high = code_point;
            // Expect \u followed by low surrogate
            if self.consume() != Some('\\') || self.consume() != Some('u') {
                return Err(JsonError::InvalidString {
                    position: self.position,
                    reason: "Expected low surrogate after high surrogate".to_string(),
                });
            }
            let mut low = 0u32;
            for _ in 0..4 {
                let digit = self.consume().ok_or(JsonError::InvalidString {
                    position: self.position,
                    reason: "Unexpected end of input in surrogate pair".to_string(),
                })?;
                let value = digit.to_digit(16).ok_or(JsonError::InvalidString {
                    position: self.position.wrapping_sub(1),
                    reason: format!("Invalid hex digit '{}'", digit),
                })?;
                low = (low << 4) | value;
            }
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err(JsonError::InvalidString {
                    position: self.position,
                    reason: format!("Invalid low surrogate: U+{:04X}", low),
                });
            }
            code_point = ((high - 0xD800) << 10) + (low - 0xDC00) + 0x10000;
        } else if (0xDC00..=0xDFFF).contains(&code_point) {
            return Err(JsonError::InvalidString {
                position: self.position,
                reason: "Unexpected low surrogate without high surrogate".to_string(),
            });
        }

        char::from_u32(code_point).ok_or(JsonError::InvalidString {
            position: self.position,
            reason: format!("Invalid Unicode code point: {}", code_point),
        })
    }

    /// Parse a number
    fn parse_number(&mut self) -> Result<JsonNode> {
        let start = self.position;
        let mut num_str = String::new();

        // Parse optional minus sign
        if let Some(&'-') = self.chars.peek() {
            if let Some(c) = self.consume() {
                num_str.push(c);
            }
        }

        // Parse integer part
        if let Some(&'0') = self.chars.peek() {
            if let Some(c) = self.consume() {
                num_str.push(c);
            }
        } else if let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_digit() {
                        if let Some(ch) = self.consume() {
                            num_str.push(ch);
                        }
                    } else {
                        break;
                    }
                }
            } else {
                return Err(JsonError::InvalidNumber {
                    position: self.position,
                    reason: "Expected digit".to_string(),
                });
            }
        } else {
            return Err(JsonError::InvalidNumber {
                position: self.position,
                reason: "Expected digit".to_string(),
            });
        }

        // Parse fractional part
        if let Some(&'.') = self.chars.peek() {
            if let Some(c) = self.consume() {
                num_str.push(c);
            }
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    if let Some(ch) = self.consume() {
                        num_str.push(ch);
                    }
                } else {
                    break;
                }
            }
        }

        // Parse exponent
        if let Some(&'e' | &'E') = self.chars.peek() {
            if let Some(c) = self.consume() {
                num_str.push(c);
            }
            if let Some(&'+' | &'-') = self.chars.peek() {
                if let Some(c) = self.consume() {
                    num_str.push(c);
                }
            }
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() {
                    if let Some(ch) = self.consume() {
                        num_str.push(ch);
                    }
                } else {
                    break;
                }
            }
        }

        num_str
            .parse::<f64>()
            .map(JsonNode::Number)
            .map_err(|_| JsonError::InvalidNumber {
                position: start,
                reason: "Failed to parse number".to_string(),
            })
    }

    /// Parse an array
    fn parse_array(&mut self) -> Result<JsonNode> {
        if self.current_depth >= self.options.nesting_limit {
            return Err(JsonError::NestingLimitExceeded {
                position: self.position,
                limit: self.options.nesting_limit,
            });
        }
        self.current_depth = self.current_depth.saturating_add(1);

        self.consume(); // Skip opening bracket
        self.skip_whitespace();

        let mut array = Vec::new();

        if let Some(&']') = self.chars.peek() {
            self.consume();
            self.current_depth = self.current_depth.saturating_sub(1);
            return Ok(JsonNode::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();

            match self.consume() {
                Some(',') => {
                    self.skip_whitespace();
                    if let Some(&']') = self.chars.peek() {
                        self.consume();
                        break;
                    }
                    continue;
                }
                Some(']') => break,
                Some(c) => {
                    self.current_depth = self.current_depth.saturating_sub(1);
                    return Err(JsonError::UnexpectedCharacter {
                        character: c,
                        position: self.position.saturating_sub(1),
                        expected: "',' or ']'".to_string(),
                    })
                }
                None => {
                    self.current_depth = self.current_depth.saturating_sub(1);
                    return Err(JsonError::UnexpectedEndOfInput {
                        position: self.position,
                        expected: "',' or ']'".to_string(),
                    });
                }
            }
        }

        self.current_depth = self.current_depth.saturating_sub(1);
        Ok(JsonNode::Array(array))
    }

    /// Parse an object
    fn parse_object(&mut self) -> Result<JsonNode> {
        if self.current_depth >= self.options.nesting_limit {
            return Err(JsonError::NestingLimitExceeded {
                position: self.position,
                limit: self.options.nesting_limit,
            });
        }
        self.current_depth = self.current_depth.saturating_add(1);

        self.consume(); // Skip opening brace
        self.skip_whitespace();

        let mut object = Vec::new();
        let mut keys = std::collections::HashSet::new();

        if let Some(&'}') = self.chars.peek() {
            self.consume();
            self.current_depth = self.current_depth.saturating_sub(1);
            return Ok(JsonNode::Object(object));
        }

        loop {
            // Parse key
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonNode::String(s) => s,
                _ => unreachable!(),
            };

            // Check for duplicate key
            if keys.contains(&key) {
                self.current_depth = self.current_depth.saturating_sub(1);
                return Err(JsonError::DuplicateKey {
                    key: key.clone(),
                    position: self.position,
                });
            }
            keys.insert(key.clone());

            // Parse colon
            self.skip_whitespace();
            match self.consume() {
                Some(':') => {}
                Some(c) => {
                    self.current_depth = self.current_depth.saturating_sub(1);
                    return Err(JsonError::UnexpectedCharacter {
                        character: c,
                        position: self.position.saturating_sub(1),
                        expected: ":".to_string(),
                    })
                }
                None => {
                    self.current_depth = self.current_depth.saturating_sub(1);
                    return Err(JsonError::UnexpectedEndOfInput {
                        position: self.position,
                        expected: ":".to_string(),
                    })
                }
            }

            // Parse value
            let value = self.parse_value()?;
            object.push((key, value));

            self.skip_whitespace();

            match self.consume() {
                Some(',') => {
                    self.skip_whitespace();
                    if let Some(&'}') = self.chars.peek() {
                        self.consume();
                        break;
                    }
                    continue;
                }
                Some('}') => break,
                Some(c) => {
                    self.current_depth = self.current_depth.saturating_sub(1);
                    return Err(JsonError::UnexpectedCharacter {
                        character: c,
                        position: self.position.saturating_sub(1),
                        expected: "',' or '}'".to_string(),
                    })
                }
                None => {
                    self.current_depth = self.current_depth.saturating_sub(1);
                    return Err(JsonError::UnexpectedEndOfInput {
                        position: self.position,
                        expected: "',' or '}'".to_string(),
                    })
                }
            }
        }

        self.current_depth = self.current_depth.saturating_sub(1);
        Ok(JsonNode::Object(object))
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
            // Even when require_null_terminated is false, we should not allow
            // non-whitespace characters after the JSON value
            return Err(JsonError::TrailingCharacters {
                position: self.position,
            });
        }

        Ok(value)
    }
}

/// Parse a JSON string into a JsonNode
///
/// # Errors
///
/// Returns `JsonError` if the JSON string is invalid or contains syntax errors.
pub fn parse(json: &str) -> Result<JsonNode> {
    parse_with_opts(json, ParseOptions::new())
}

/// Parse a JSON string with options
///
/// # Errors
///
/// Returns `JsonError` if the JSON string is invalid, contains syntax errors,
/// or exceeds the nesting limit specified in the options.
pub fn parse_with_opts(json: &str, opts: ParseOptions) -> Result<JsonNode> {
    let mut parser = Parser::new(json, opts);
    parser.parse_document()
}

/// Parse a JSON string with length limit
///
/// # Errors
///
/// Returns `JsonError` if the JSON string is invalid or contains syntax errors.
pub fn parse_with_length(json: &str, length: usize) -> Result<JsonNode> {
    let truncated = if json.len() > length {
        &json[..length]
    } else {
        json
    };
    parse(truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert!(parse("").is_err());
    }

    #[test]
    fn test_invalid_json() {
        assert!(parse("123abc").is_err());
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(parse("null"), Ok(JsonNode::Null));
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true"), Ok(JsonNode::Bool(true)));
        assert_eq!(parse("false"), Ok(JsonNode::Bool(false)));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse("123"), Ok(JsonNode::Number(123.0)));
        assert_eq!(parse("-123"), Ok(JsonNode::Number(-123.0)));
        assert_eq!(parse("123.456"), Ok(JsonNode::Number(123.456)));
        assert_eq!(parse("123.456e7"), Ok(JsonNode::Number(123.456e7)));
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
        assert_eq!(parse(r#""\t\\""#), Ok(JsonNode::String("\t\\".to_string())));
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
        let result = parse("[[1, 2], [3, 4]]");
        assert_eq!(
            result,
            Ok(JsonNode::Array(vec![
                JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]),
                JsonNode::Array(vec![JsonNode::Number(3.0), JsonNode::Number(4.0)]),
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
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
            "is_student": false,
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
    fn test_parse_with_options() {
        let mut opts = ParseOptions::new();
        opts.require_null_terminated = true;
        assert!(parse_with_opts("123", opts).is_ok());
    }

    #[test]
    fn test_parse_with_length() {
        let json = r#"{"key": "value", "extra": "data"}"#;
        let result = parse_with_length(json, 15);
        assert!(result.is_err()); // Should fail because truncated JSON
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(parse("  null  "), Ok(JsonNode::Null));
        assert_eq!(
            parse("  [  1  ,  2  ,  3  ]  "),
            Ok(JsonNode::Array(vec![
                JsonNode::Number(1.0),
                JsonNode::Number(2.0),
                JsonNode::Number(3.0),
            ]))
        );
    }

    #[test]
    fn test_trailing_comma() {
        // Trailing commas should be allowed
        let result = parse("[1, 2, 3,]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_key() {
        // Duplicate keys should return an error
        let result = parse(r#"{"key": "value1", "key": "value2"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_escape() {
        let result = parse(r#""\u0048\u0065\u006c\u006c\u006f""#);
        assert_eq!(result, Ok(JsonNode::String("Hello".to_string())));
    }

    #[test]
    fn test_nesting_limit() {
        let mut opts = ParseOptions::new();
        opts.nesting_limit = 2;

        // Simple nesting should work
        let result = parse_with_opts(r#"{"a": {"b": 1}}"#, opts.clone());
        assert!(result.is_ok());

        // Exceeding limit should fail
        let result = parse_with_opts(r#"{"a": {"b": {"c": 1}}}"#, opts);
        assert!(result.is_err());
    }
}