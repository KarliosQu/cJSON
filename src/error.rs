use std::fmt;

/// JSON parsing error types
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// Unexpected end of input
    UnexpectedEndOfInput {
        position: usize,
        expected: String,
    },
    /// Unexpected character encountered
    UnexpectedCharacter {
        character: char,
        position: usize,
        expected: String,
    },
    /// Invalid number format
    InvalidNumber {
        position: usize,
        reason: String,
    },
    /// Invalid string format
    InvalidString {
        position: usize,
        reason: String,
    },
    /// General syntax error
    SyntaxError {
        position: usize,
        message: String,
    },
    /// Nesting limit exceeded
    NestingLimitExceeded {
        position: usize,
        limit: usize,
    },
    /// Invalid escape sequence
    InvalidEscapeSequence {
        position: usize,
        sequence: String,
    },
    /// Duplicate object key
    DuplicateKey {
        key: String,
        position: usize,
    },
    /// Trailing comma in array or object
    TrailingComma {
        position: usize,
    },
    /// Expected null terminator
    ExpectedNullTerminator {
        position: usize,
    },
    /// Invalid type for operation
    InvalidType {
        expected: String,
        found: String,
    },
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedEndOfInput { position, expected } => {
                write!(
                    f,
                    "Unexpected end of input at position {}: expected '{}'",
                    position, expected
                )
            }
            JsonError::UnexpectedCharacter { character, position, expected } => {
                write!(
                    f,
                    "Unexpected character '{}' at position {}: expected '{}'",
                    character, position, expected
                )
            }
            JsonError::InvalidNumber { position, reason } => {
                write!(f, "Invalid number at position {}: {}", position, reason)
            }
            JsonError::InvalidString { position, reason } => {
                write!(f, "Invalid string at position {}: {}", position, reason)
            }
            JsonError::SyntaxError { position, message } => {
                write!(f, "Syntax error at position {}: {}", position, message)
            }
            JsonError::NestingLimitExceeded { position, limit } => {
                write!(
                    f,
                    "Nesting limit exceeded at position {}: maximum depth is {}",
                    position, limit
                )
            }
            JsonError::InvalidEscapeSequence { position, sequence } => {
                write!(
                    f,
                    "Invalid escape sequence at position {}: '{}'",
                    position, sequence
                )
            }
            JsonError::DuplicateKey { key, position } => {
                write!(
                    f,
                    "Duplicate key '{}' at position {}",
                    key, position
                )
            }
            JsonError::TrailingComma { position } => {
                write!(f, "Trailing comma at position {}", position)
            }
            JsonError::ExpectedNullTerminator { position } => {
                write!(
                    f,
                    "Expected null terminator at position {}",
                    position
                )
            }
            JsonError::InvalidType { expected, found } => {
                write!(
                    f,
                    "Invalid type: expected '{}', found '{}'",
                    expected, found
                )
            }
        }
    }
}

impl std::error::Error for JsonError {}

/// Result type alias for JSON operations
pub type Result<T> = std::result::Result<T, JsonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = JsonError::UnexpectedEndOfInput {
            position: 10,
            expected: "value".to_string(),
        };
        assert!(err.to_string().contains("Unexpected end of input"));
        assert!(err.to_string().contains("position 10"));

        let err2 = JsonError::InvalidNumber {
            position: 5,
            reason: "Invalid digit".to_string(),
        };
        assert!(err2.to_string().contains("Invalid number"));
        assert!(err2.to_string().contains("position 5"));
    }
}