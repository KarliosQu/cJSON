use std::fmt;

/// JSON value types
#[derive(Debug, Clone, PartialEq)]
pub enum JsonNode {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonNode>),
    Object(Vec<(String, JsonNode)>),
    Raw(String),
}

impl JsonNode {
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, JsonNode::Null)
    }

    /// Check if the value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonNode::Bool(_))
    }

    /// Check if the value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, JsonNode::Number(_))
    }

    /// Check if the value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, JsonNode::String(_))
    }

    /// Check if the value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, JsonNode::Array(_))
    }

    /// Check if the value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, JsonNode::Object(_))
    }

    /// Get the value as a string reference, if it is a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            JsonNode::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as a number, if it is a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonNode::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the value as a boolean, if it is a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonNode::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the value as an array reference, if it is an array
    pub fn as_array(&self) -> Option<&Vec<JsonNode>> {
        match self {
            JsonNode::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get the value as an object reference, if it is an object
    pub fn as_object(&self) -> Option<&Vec<(String, JsonNode)>> {
        match self {
            JsonNode::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Create a new null value
    pub fn new_null() -> Self {
        JsonNode::Null
    }

    /// Create a new boolean value
    pub fn new_bool(b: bool) -> Self {
        JsonNode::Bool(b)
    }

    /// Create a new true value
    pub fn new_true() -> Self {
        JsonNode::Bool(true)
    }

    /// Create a new false value
    pub fn new_false() -> Self {
        JsonNode::Bool(false)
    }

    /// Create a new number value
    pub fn new_number(n: f64) -> Self {
        JsonNode::Number(n)
    }

    /// Create a new string value
    pub fn new_string(s: impl Into<String>) -> Self {
        JsonNode::String(s.into())
    }

    /// Create a new empty array
    pub fn new_array() -> Self {
        JsonNode::Array(Vec::new())
    }

    /// Create a new array with values
    pub fn create_array(values: Vec<JsonNode>) -> Self {
        JsonNode::Array(values)
    }

    /// Create a new empty object
    pub fn new_object() -> Self {
        JsonNode::Object(Vec::new())
    }

    /// Create a new object with key-value pairs
    pub fn create_object(pairs: Vec<(String, JsonNode)>) -> Self {
        JsonNode::Object(pairs)
    }

    /// Create a new raw JSON value
    pub fn new_raw(s: impl Into<String>) -> Self {
        JsonNode::Raw(s.into())
    }

    /// Get a value from an object by key
    pub fn get(&self, key: &str) -> Option<&JsonNode> {
        match self {
            JsonNode::Object(pairs) => {
                pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// Get a mutable value from an object by key
    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonNode> {
        match self {
            JsonNode::Object(pairs) => {
                pairs.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// Get a value from an array by index
    pub fn get_at(&self, index: usize) -> Option<&JsonNode> {
        match self {
            JsonNode::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    /// Get the size of an array or object
    pub fn len(&self) -> usize {
        match self {
            JsonNode::Array(arr) => arr.len(),
            JsonNode::Object(pairs) => pairs.len(),
            _ => 0,
        }
    }

    /// Check if the array or object is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for JsonNode {
    fn default() -> Self {
        JsonNode::Null
    }
}

impl fmt::Display for JsonNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonNode::Null => write!(f, "null"),
            JsonNode::Bool(b) => write!(f, "{}", b),
            JsonNode::Number(n) => write!(f, "{}", n),
            JsonNode::String(s) => write!(f, "\"{}\"", s),
            JsonNode::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JsonNode::Object(pairs) => {
                write!(f, "{{")?;
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
            JsonNode::Raw(s) => write!(f, "{}", s),
        }
    }
}

impl Eq for JsonNode {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_checks() {
        let null = JsonNode::Null;
        let bool_val = JsonNode::Bool(true);
        let num = JsonNode::Number(42.0);
        let str_val = JsonNode::String("hello".to_string());
        let arr = JsonNode::Array(vec![]);
        let obj = JsonNode::Object(vec![]);

        assert!(null.is_null());
        assert!(bool_val.is_bool());
        assert!(num.is_number());
        assert!(str_val.is_string());
        assert!(arr.is_array());
        assert!(obj.is_object());
    }

    #[test]
    fn test_value_accessors() {
        let str_val = JsonNode::String("hello".to_string());
        assert_eq!(str_val.as_string(), Some("hello"));

        let num = JsonNode::Number(42.0);
        assert_eq!(num.as_number(), Some(42.0));

        let bool_val = JsonNode::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));

        let arr = JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]);
        assert_eq!(arr.len(), 2);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));

        let obj = JsonNode::Object(vec![("key".to_string(), JsonNode::Number(42.0))]);
        assert_eq!(obj.len(), 1);
        assert_eq!(obj.get("key"), Some(&JsonNode::Number(42.0)));
    }

    #[test]
    fn test_constructors() {
        assert_eq!(JsonNode::new_null(), JsonNode::Null);
        assert_eq!(JsonNode::new_true(), JsonNode::Bool(true));
        assert_eq!(JsonNode::new_false(), JsonNode::Bool(false));
        assert_eq!(JsonNode::new_bool(true), JsonNode::Bool(true));
        assert_eq!(JsonNode::new_number(42.0), JsonNode::Number(42.0));
        assert_eq!(JsonNode::new_string("hello"), JsonNode::String("hello".to_string()));
        assert_eq!(JsonNode::new_array(), JsonNode::Array(vec![]));
        assert_eq!(JsonNode::new_object(), JsonNode::Object(vec![]));
    }
}