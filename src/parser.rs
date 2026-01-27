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
            Err(JsonError::InvalidValue {
                position: self.position,
                reason: "Expected 'null'".to_string(),
            })
        }
    }

    /// Parse a boolean value
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
            Err(JsonError::InvalidValue {
                position: self.position,
                reason: "Expected 'true' or 'false'".to_string(),
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
        
        // Integer part
        let mut has_digits = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.consume();
                has_digits = true;
            } else {
                break;
            }
        }
        
        if !has_digits {
            return Err(JsonError::InvalidNumber {
                position: start,
                reason: "Expected digits".to_string(),
            });
        }
        
        // Fractional part
        if let Some('.') = self.peek() {
            self.consume();
            has_digits = false;
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.consume();
                    has_digits = true;
                } else {
                    break;
                }
            }
            if !has_digits {
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
                has_digits = false;
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        self.consume();
                        has_digits = true;
                    } else {
                        break;
                    }
                }
                if !has_digits {
                    return Err(JsonError::InvalidNumber {
                        position: self.position - 1,
                        reason: "Expected digits in exponent".to_string(),
                    });
                }
            }
        }
        
        let num_str = &self.input[start..self.position];
        num_str.parse::<f64>().map(JsonNode::Number).map_err(|_| JsonError::InvalidNumber {
            position: start,
            reason: "Failed to parse number".to_string(),
        })
    }

    /// Parse a string
    fn parse_string(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();
        self.expect_char('"')?;
        
        let start = self.position;
        let mut result = String::new();
        
        while let Some(c) = self.peek() {
            if c == '"' {
                self.consume();
                return Ok(JsonNode::String(result));
            } else if c == '\\' {
                self.consume();
                match self.peek() {
                    Some('"') => { self.consume(); result.push('"'); }
                    Some('\\') => { self.consume(); result.push('\\'); }
                    Some('/') => { self.consume(); result.push('/'); }
                    Some('b') => { self.consume(); result.push('\x08'); }
                    Some('f') => { self.consume(); result.push('\x0c'); }
                    Some('n') => { self.consume(); result.push('\n'); }
                    Some('r') => { self.consume(); result.push('\r'); }
                    Some('t') => { self.consume(); result.push('\t'); }
                    Some('u') => {
                        self.consume();
                        let mut unicode = 0;
                        for _ in 0..4 {
                            let hex = self.peek().ok_or_else(|| JsonError::InvalidEscape {
                                position: self.position,
                                reason: "Incomplete Unicode escape".to_string(),
                            })?;
                            let digit = hex.to_digit(16).ok_or_else(|| JsonError::InvalidEscape {
                                position: self.position,
                                reason: format!("Invalid hex digit: {}", hex),
                            })?;
                            unicode = (unicode << 4) | digit;
                            self.consume();
                        }
                        if let Some(c) = char::from_u32(unicode) {
                            result.push(c);
                        }
                    }
                    Some(other) => {
                        return Err(JsonError::InvalidEscape {
                            position: self.position,
                            reason: format!("Invalid escape sequence: \\{}", other),
                        });
                    }
                    None => {
                        return Err(JsonError::UnexpectedEndOfInput {
                            position: self.position,
                            expected: "escape sequence character".to_string(),
                        });
                    }
                }
            } else if c.is_control() {
                return Err(JsonError::InvalidCharacter {
                    position: self.position,
                    character: c,
                    reason: "Control character in string".to_string(),
                });
            } else {
                self.consume();
                result.push(c);
            }
        }
        
        Err(JsonError::UnexpectedEndOfInput {
            position: self.position,
            expected: "closing quote".to_string(),
        })
    }

    /// Parse an array
    fn parse_array(&mut self) -> Result<JsonNode> {
        self.expect_char('[')?;
        self.current_depth += 1;
        if self.current_depth > self.options.nesting_limit {
            return Err(JsonError::NestingLimitExceeded {
                limit: self.options.nesting_limit,
            });
        }
        
        let mut items = Vec::new();
        
        loop {
            self.skip_whitespace();
            if let Some(']') = self.peek() {
                self.consume();
                self.current_depth -= 1;
                return Ok(JsonNode::Array(items));
            }
            
            let item = self.parse_value()?;
            items.push(item);
            
            self.skip_whitespace();
            if let Some(',') = self.peek() {
                self.consume();
                // Check for trailing comma
                self.skip_whitespace();
                if let Some(']') = self.peek() {
                    return Err(JsonError::UnexpectedCharacter {
                        character: ']',
                        position: self.position,
                        expected: "value after comma".to_string(),
                    });
                }
            } else if let Some(']') = self.peek() {
                continue;
            } else {
                return Err(JsonError::UnexpectedCharacter {
                    character: self.peek().unwrap_or('\0'),
                    position: self.position,
                    expected: "',' or ']'".to_string(),
                });
            }
        }
    }

    /// Parse an object
    fn parse_object(&mut self) -> Result<JsonNode> {
        self.expect_char('{')?;
        self.current_depth += 1;
        if self.current_depth > self.options.nesting_limit {
            return Err(JsonError::NestingLimitExceeded {
                limit: self.options.nesting_limit,
            });
        }
        
        let mut pairs = Vec::new();
        
        loop {
            self.skip_whitespace();
            if let Some('}') = self.peek() {
                self.consume();
                self.current_depth -= 1;
                return Ok(JsonNode::Object(pairs));
            }
            
            // Parse key (must be a string)
            let key = match self.parse_value()? {
                JsonNode::String(s) => s,
                _ => {
                    return Err(JsonError::ExpectedString {
                        position: self.position,
                    });
                }
            };
            
            // Check for duplicate key
            if pairs.iter().any(|(k, _)| k == &key) {
                return Err(JsonError::DuplicateKey {
                    key: key.clone(),
                });
            }
            
            // Expect colon
            self.expect_char(':')?;
            
            // Parse value
            let value = self.parse_value()?;
            pairs.push((key, value));
            
            self.skip_whitespace();
            if let Some(',') = self.peek() {
                self.consume();
                // Check for trailing comma
                self.skip_whitespace();
                if let Some('}') = self.peek() {
                    return Err(JsonError::UnexpectedCharacter {
                        character: '}',
                        position: self.position,
                        expected: "key after comma".to_string(),
                    });
                }
            } else if let Some('}') = self.peek() {
                continue;
            } else {
                return Err(JsonError::UnexpectedCharacter {
                    character: self.peek().unwrap_or('\0'),
                    position: self.position,
                    expected: "',' or '}'".to_string(),
                });
            }
        }
    }

    /// Parse any JSON value
    fn parse_value(&mut self) -> Result<JsonNode> {
        self.skip_whitespace();
        
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some('-') | Some(c) if c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(JsonError::UnexpectedCharacter {
                character: c,
                position: self.position,
                expected: "JSON value".to_string(),
            }),
            None => Err(JsonError::UnexpectedEndOfInput {
                position: self.position,
                expected: "JSON value".to_string(),
            }),
        }
    }

    /// Parse the entire document
    fn parse_document(&mut self) -> Result<JsonNode> {
        let result = self.parse_value()?;
        
        self.skip_whitespace();
        
        // Check for null terminator if required
        if self.options.require_null_terminated && !self.is_eof() {
            return Err(JsonError::UnexpectedCharacter {
                character: self.peek().unwrap_or('\0'),
                position: self.position,
                expected: "null terminator".to_string(),
            });
        }
        
        // Ensure we consumed all input
        if !self.is_eof() {
            return Err(JsonError::UnexpectedCharacter {
                character: self.peek().unwrap_or('\0'),
                position: self.position,
                expected: "end of input".to_string(),
            });
        }
        
        Ok(result)
    }
}

/// Parse a JSON string
pub fn parse(json: &str) -> Result<JsonNode> {
    parse_with_opts(json, ParseOptions::new())
}

/// Parse a JSON string with a maximum length
pub fn parse_with_length(json: &str, max_length: usize) -> Result<JsonNode> {
    if json.len() > max_length {
        return Err(JsonError::UnexpectedEndOfInput {
            position: max_length,
            expected: "more input".to_string(),
        });
    }
    parse(json)
}

/// Parse a JSON string with custom options
pub fn parse_with_opts(json: &str, opts: ParseOptions) -> Result<JsonNode> {
    let mut parser = Parser::new(json, opts);
    parser.parse_document()
}