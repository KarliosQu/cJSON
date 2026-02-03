use std::fmt;

use crate::error::Result;
use crate::JsonError;

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

    /// Check if the value is true
    pub fn is_true(&self) -> bool {
        matches!(self, JsonNode::Bool(true))
    }

    /// Check if the value is false
    pub fn is_false(&self) -> bool {
        matches!(self, JsonNode::Bool(false))
    }

    /// Check if the value is invalid (always returns false in Rust)
    pub fn is_invalid(&self) -> bool {
        false
    }

    /// Check if the value is raw JSON
    pub fn is_raw(&self) -> bool {
        matches!(self, JsonNode::Raw(_))
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

    /// Get the type name of the JsonNode as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            JsonNode::Null => "Null",
            JsonNode::Bool(_) => "Bool",
            JsonNode::Number(_) => "Number",
            JsonNode::String(_) => "String",
            JsonNode::Array(_) => "Array",
            JsonNode::Object(_) => "Object",
            JsonNode::Raw(_) => "Raw",
        }
    }

    /// Add an item to the array
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    pub fn add_item_to_array(&mut self, item: JsonNode) -> Result<()> {
        match self {
            JsonNode::Array(arr) => {
                arr.push(item);
                Ok(())
            }
            _ => Err(JsonError::InvalidType {
                expected: "Array".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Add an item to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_item_to_object(&mut self, key: impl Into<String>, item: JsonNode) -> Result<()> {
        match self {
            JsonNode::Object(pairs) => {
                pairs.push((key.into(), item));
                Ok(())
            }
            _ => Err(JsonError::InvalidType {
                expected: "Object".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Add a string to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_string_to_object(&mut self, key: impl Into<String>, value: &str) -> Result<()> {
        self.add_item_to_object(key, JsonNode::String(value.to_string()))
    }

    /// Add a number to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_number_to_object(&mut self, key: impl Into<String>, value: f64) -> Result<()> {
        self.add_item_to_object(key, JsonNode::Number(value))
    }

    /// Add a boolean to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_bool_to_object(&mut self, key: impl Into<String>, value: bool) -> Result<()> {
        self.add_item_to_object(key, JsonNode::Bool(value))
    }

    /// Create an array from a slice of i64 values
    pub fn create_int_array(values: &[i64]) -> Self {
        JsonNode::Array(values.iter().map(|&v| JsonNode::Number(v as f64)).collect())
    }

    /// Create an array from a slice of f32 values
    pub fn create_float_array(values: &[f32]) -> Self {
        JsonNode::Array(values.iter().map(|&v| JsonNode::Number(v as f64)).collect())
    }

    /// Create an array from a slice of f64 values
    pub fn create_double_array(values: &[f64]) -> Self {
        JsonNode::Array(values.iter().map(|&v| JsonNode::Number(v)).collect())
    }

    /// Create an array from a slice of string references
    pub fn create_string_array(values: &[&str]) -> Self {
        JsonNode::Array(values.iter().map(|&v| JsonNode::String(v.to_string())).collect())
    }

    /// Delete an item from the array at the specified index
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is out of bounds
    pub fn delete_item_from_array(&mut self, index: usize) -> Result<()> {
        match self {
            JsonNode::Array(arr) => {
                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    });
                }
                arr.remove(index);
                Ok(())
            }
            _ => Err(JsonError::InvalidType {
                expected: "Array".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Delete an item from the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    /// Returns `Err(JsonError::KeyNotFound)` if the key does not exist
    pub fn delete_item_from_object(&mut self, key: &str) -> Result<()> {
        match self {
            JsonNode::Object(pairs) => {
                if let Some(pos) = pairs.iter().position(|(k, _)| k == key) {
                    pairs.remove(pos);
                    Ok(())
                } else {
                    Err(JsonError::KeyNotFound {
                        key: key.to_string(),
                    })
                }
            }
            _ => Err(JsonError::InvalidType {
                expected: "Object".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Detach an item from the array at the specified index and return it
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is out of bounds
    pub fn detach_item_from_array(&mut self, index: usize) -> Result<JsonNode> {
        match self {
            JsonNode::Array(arr) => {
                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    });
                }
                Ok(arr.remove(index))
            }
            _ => Err(JsonError::InvalidType {
                expected: "Array".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Detach an item from the object with the given key and return it
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    /// Returns `Err(JsonError::KeyNotFound)` if the key does not exist
    pub fn detach_item_from_object(&mut self, key: &str) -> Result<JsonNode> {
        match self {
            JsonNode::Object(pairs) => {
                if let Some(pos) = pairs.iter().position(|(k, _)| k == key) {
                    Ok(pairs.remove(pos).1)
                } else {
                    Err(JsonError::KeyNotFound {
                        key: key.to_string(),
                    })
                }
            }
            _ => Err(JsonError::InvalidType {
                expected: "Object".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Replace an item in the array at the specified index
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is out of bounds
    pub fn replace_item_in_array(&mut self, index: usize, new_item: JsonNode) -> Result<()> {
        match self {
            JsonNode::Array(arr) => {
                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    });
                }
                arr[index] = new_item;
                Ok(())
            }
            _ => Err(JsonError::InvalidType {
                expected: "Array".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Replace an item in the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    /// Returns `Err(JsonError::KeyNotFound)` if the key does not exist
    pub fn replace_item_in_object(&mut self, key: &str, new_item: JsonNode) -> Result<()> {
        match self {
            JsonNode::Object(pairs) => {
                if let Some(pos) = pairs.iter().position(|(k, _)| k == key) {
                    pairs[pos] = (key.to_string(), new_item);
                    Ok(())
                } else {
                    Err(JsonError::KeyNotFound {
                        key: key.to_string(),
                    })
                }
            }
            _ => Err(JsonError::InvalidType {
                expected: "Object".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Insert an item into the array at the specified position
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is out of bounds
    pub fn insert_item_in_array(&mut self, index: usize, item: JsonNode) -> Result<()> {
        match self {
            JsonNode::Array(arr) => {
                if index > arr.len() {
                    return Err(JsonError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    });
                }
                arr.insert(index, item);
                Ok(())
            }
            _ => Err(JsonError::InvalidType {
                expected: "Array".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }

    /// Sort the keys of an object
    ///
    /// # Arguments
    ///
    /// * `case_sensitive` - If true, sort with case sensitivity; otherwise, sort case-insensitively
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn sort_object(&mut self, case_sensitive: bool) -> Result<()> {
        match self {
            JsonNode::Object(pairs) => {
                if case_sensitive {
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                } else {
                    pairs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
                }
                Ok(())
            }
            _ => Err(JsonError::InvalidType {
                expected: "Object".to_string(),
                found: self.type_name().to_string(),
            }),
        }
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

    #[test]
    fn test_add_item_to_array() {
        let mut arr = JsonNode::new_array();
        arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
        arr.add_item_to_array(JsonNode::String("hello".to_string())).unwrap();
        arr.add_item_to_array(JsonNode::Bool(true)).unwrap();
        
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
        assert_eq!(arr.get_at(1), Some(&JsonNode::String("hello".to_string())));
        assert_eq!(arr.get_at(2), Some(&JsonNode::Bool(true)));
    }

    #[test]
    fn test_add_item_to_array_invalid_type() {
        use crate::error::JsonError;
        
        let mut node = JsonNode::new_object();
        let result = node.add_item_to_array(JsonNode::Number(1.0));
        
        assert!(result.is_err());
        match result {
            Err(JsonError::InvalidType { expected, found }) => {
                assert_eq!(expected, "Array");
                assert_eq!(found, "Object");
            }
            _ => panic!("Expected InvalidType error"),
        }
    }

    #[test]
    fn test_add_item_to_object() {
        let mut obj = JsonNode::new_object();
        obj.add_item_to_object("name", JsonNode::String("Alice".to_string())).unwrap();
        obj.add_item_to_object("age", JsonNode::Number(30.0)).unwrap();
        
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("name"), Some(&JsonNode::String("Alice".to_string())));
        assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
    }

    #[test]
    fn test_add_item_to_object_invalid_type() {
        use crate::error::JsonError;
        
        let mut node = JsonNode::new_array();
        let result = node.add_item_to_object("key", JsonNode::String("value".to_string()));
        
        assert!(result.is_err());
        match result {
            Err(JsonError::InvalidType { expected, found }) => {
                assert_eq!(expected, "Object");
                assert_eq!(found, "Array");
            }
            _ => panic!("Expected InvalidType error"),
        }
    }

    #[test]
    fn test_convenience_add_methods() {
        let mut obj = JsonNode::new_object();
        obj.add_string_to_object("name", "Alice").unwrap();
        obj.add_number_to_object("age", 30.0).unwrap();
        obj.add_bool_to_object("active", true).unwrap();
        
        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get("name"), Some(&JsonNode::String("Alice".to_string())));
        assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
        assert_eq!(obj.get("active"), Some(&JsonNode::Bool(true)));
    }

    #[test]
    fn test_create_int_array() {
        let values: &[i64] = &[1, 2, 3, 4, 5];
        let arr = JsonNode::create_int_array(values);
        
        assert_eq!(arr.len(), 5);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
        assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.0)));
        assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.0)));
        assert_eq!(arr.get_at(3), Some(&JsonNode::Number(4.0)));
        assert_eq!(arr.get_at(4), Some(&JsonNode::Number(5.0)));
    }

    #[test]
    fn test_create_float_array() {
        let values: &[f32] = &[1.1, 2.2, 3.3];
        let arr = JsonNode::create_float_array(values);

        assert_eq!(arr.len(), 3);
        // Use approximate comparison for floating point values
        match arr.get_at(0) {
            Some(JsonNode::Number(n)) => assert!((n - 1.1_f64).abs() < 1e-6, "Expected ~1.1, got {}", n),
            _ => panic!("Expected Number"),
        }
        match arr.get_at(1) {
            Some(JsonNode::Number(n)) => assert!((n - 2.2_f64).abs() < 1e-6, "Expected ~2.2, got {}", n),
            _ => panic!("Expected Number"),
        }
        match arr.get_at(2) {
            Some(JsonNode::Number(n)) => assert!((n - 3.3_f64).abs() < 1e-6, "Expected ~3.3, got {}", n),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_create_double_array() {
        let values: &[f64] = &[1.5, 2.5, 3.5, 4.5];
        let arr = JsonNode::create_double_array(values);
        
        assert_eq!(arr.len(), 4);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.5)));
        assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.5)));
        assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.5)));
        assert_eq!(arr.get_at(3), Some(&JsonNode::Number(4.5)));
    }

    #[test]
    fn test_create_string_array() {
        let values: &[&str] = &["hello", "world", "rust"];
        let arr = JsonNode::create_string_array(values);
        
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_at(0), Some(&JsonNode::String("hello".to_string())));
        assert_eq!(arr.get_at(1), Some(&JsonNode::String("world".to_string())));
        assert_eq!(arr.get_at(2), Some(&JsonNode::String("rust".to_string())));
    }
}