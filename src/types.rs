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
    fn type_name(&self) -> &'static str {
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

    /// Add a null value to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_null_to_object(&mut self, key: impl Into<String>) -> Result<()> {
        self.add_item_to_object(key, JsonNode::Null)
    }

    /// Add a true value to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_true_to_object(&mut self, key: impl Into<String>) -> Result<()> {
        self.add_item_to_object(key, JsonNode::Bool(true))
    }

    /// Add a false value to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_false_to_object(&mut self, key: impl Into<String>) -> Result<()> {
        self.add_item_to_object(key, JsonNode::Bool(false))
    }

    /// Add raw JSON string to the object with the given key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    pub fn add_raw_to_object(&mut self, key: impl Into<String>, raw: &str) -> Result<()> {
        self.add_item_to_object(key, JsonNode::Raw(raw.to_string()))
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

    /// Delete an item from an array at the specified index
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is out of bounds
    pub fn delete_item_from_array(&mut self, index: usize) -> Result<()> {
        match self {
            JsonNode::Array(arr) => {
                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds { index, length: arr.len() });
                }
                arr.remove(index);
                Ok(())
            }
            _ => Err(JsonError::InvalidType { expected: "Array".to_string(), found: self.type_name().to_string() }),
        }
    }

    /// Detach an item from an array at the specified index, removing it and returning ownership
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is out of bounds
    pub fn detach_item_from_array(&mut self, index: usize) -> Result<JsonNode> {
        match self {
            JsonNode::Array(arr) => {
                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds { index, length: arr.len() });
                }
                Ok(arr.remove(index))
            }
            _ => Err(JsonError::InvalidType { expected: "Array".to_string(), found: self.type_name().to_string() }),
        }
    }

    /// Delete an item from an object with the specified key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    /// Returns `Err(JsonError::KeyNotFound)` if the key does not exist
    pub fn delete_item_from_object(&mut self, key: &str) -> Result<()> {
        match self {
            JsonNode::Object(pairs) => {
                let pos = pairs.iter().position(|(k, _)| k == key);
                match pos {
                    Some(idx) => { pairs.remove(idx); Ok(()) }
                    None => Err(JsonError::KeyNotFound { key: key.to_string() }),
                }
            }
            _ => Err(JsonError::InvalidType { expected: "Object".to_string(), found: self.type_name().to_string() }),
        }
    }

    /// Detach an item from an object with the specified key, removing it and returning ownership
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    /// Returns `Err(JsonError::KeyNotFound)` if the key does not exist
    pub fn detach_item_from_object(&mut self, key: &str) -> Result<JsonNode> {
        match self {
            JsonNode::Object(pairs) => {
                let pos = pairs.iter().position(|(k, _)| k == key);
                match pos {
                    Some(idx) => {
                        let (_, value) = pairs.remove(idx);
                        Ok(value)
                    }
                    None => Err(JsonError::KeyNotFound { key: key.to_string() }),
                }
            }
            _ => Err(JsonError::InvalidType { expected: "Object".to_string(), found: self.type_name().to_string() }),
        }
    }

    /// Replace an item in an array at the specified index
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is out of bounds
    pub fn replace_item_in_array(&mut self, index: usize, newitem: JsonNode) -> Result<()> {
        match self {
            JsonNode::Array(arr) => {
                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds { index, length: arr.len() });
                }
                arr[index] = newitem;
                Ok(())
            }
            _ => Err(JsonError::InvalidType { expected: "Array".to_string(), found: self.type_name().to_string() }),
        }
    }

    /// Replace an item in an object with the specified key
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an object
    /// Returns `Err(JsonError::KeyNotFound)` if the key does not exist
    pub fn replace_item_in_object(&mut self, key: &str, newitem: JsonNode) -> Result<()> {
        match self {
            JsonNode::Object(pairs) => {
                let pos = pairs.iter().position(|(k, _)| k == key);
                match pos {
                    Some(idx) => {
                        pairs[idx] = (key.to_string(), newitem);
                        Ok(())
                    }
                    None => Err(JsonError::KeyNotFound { key: key.to_string() }),
                }
            }
            _ => Err(JsonError::InvalidType { expected: "Object".to_string(), found: self.type_name().to_string() }),
        }
    }

    /// Insert an item into an array at the specified index
    ///
    /// # Errors
    ///
    /// Returns `Err(JsonError::InvalidType)` if the node is not an array
    /// Returns `Err(JsonError::IndexOutOfBounds)` if the index is greater than the length
    pub fn insert_item_in_array(&mut self, index: usize, newitem: JsonNode) -> Result<()> {
        match self {
            JsonNode::Array(arr) => {
                if index > arr.len() {
                    return Err(JsonError::IndexOutOfBounds { index, length: arr.len() });
                }
                arr.insert(index, newitem);
                Ok(())
            }
            _ => Err(JsonError::InvalidType { expected: "Array".to_string(), found: self.type_name().to_string() }),
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