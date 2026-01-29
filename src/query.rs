//! Query module for JSON data navigation and extraction
//!
//! This module provides functions to query and navigate JSON data structures,
//! inspired by cJSON's query functionality. All functions that can fail return
//! `Result<T, JsonError>` with detailed error information.

use crate::error::{JsonError, Result};
use crate::types::JsonNode;

/// Get the size of an array
///
/// # Arguments
/// * `node` - Reference to a JsonNode that should be an array
///
/// # Returns
/// * `Ok(usize)` - The number of elements in the array
/// * `Err(JsonError::InvalidType)` - If the node is not an array
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_array_size};
///
/// let arr = JsonNode::Array(vec![
///     JsonNode::Number(1.0),
///     JsonNode::Number(2.0),
/// ]);
/// assert_eq!(get_array_size(&arr).unwrap(), 2);
/// ```
pub fn get_array_size(node: &JsonNode) -> Result<usize> {
    match node {
        JsonNode::Array(arr) => Ok(arr.len()),
        _ => Err(JsonError::InvalidType {
            expected: "Array".to_string(),
            found: node.type_name().to_string(),
        }),
    }
}

/// Get an element from an array by index
///
/// # Arguments
/// * `node` - Reference to a JsonNode that should be an array
/// * `index` - The index of the element to retrieve
///
/// # Returns
/// * `Ok(&JsonNode)` - Reference to the element at the specified index
/// * `Err(JsonError::InvalidType)` - If the node is not an array
/// * `Err(JsonError::IndexOutOfBounds)` - If the index is out of bounds
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_array_item};
///
/// let arr = JsonNode::Array(vec![
///     JsonNode::Number(1.0),
///     JsonNode::String("hello".to_string()),
/// ]);
/// assert_eq!(get_array_item(&arr, 1).unwrap(), &JsonNode::String("hello".to_string()));
/// ```
pub fn get_array_item<'a>(node: &'a JsonNode, index: usize) -> Result<&'a JsonNode> {
    match node {
        JsonNode::Array(arr) => arr.get(index).ok_or(JsonError::IndexOutOfBounds {
            index,
            length: arr.len(),
        }),
        _ => Err(JsonError::InvalidType {
            expected: "Array".to_string(),
            found: node.type_name().to_string(),
        }),
    }
}

/// Get an element from an object by key (case-insensitive)
///
/// # Arguments
/// * `node` - Reference to a JsonNode that should be an object
/// * `key` - The key to search for
///
/// # Returns
/// * `Ok(&JsonNode)` - Reference to the value associated with the key
/// * `Err(JsonError::InvalidType)` - If the node is not an object
/// * `Err(JsonError::KeyNotFound)` - If the key does not exist in the object
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_object_item};
///
/// let obj = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("John".to_string())),
///     ("age".to_string(), JsonNode::Number(30.0)),
/// ]);
/// assert_eq!(get_object_item(&obj, "name").unwrap(), &JsonNode::String("John".to_string()));
/// ```
pub fn get_object_item<'a>(node: &'a JsonNode, key: &str) -> Result<&'a JsonNode> {
    match node {
        JsonNode::Object(pairs) => pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
            .ok_or(JsonError::KeyNotFound {
                key: key.to_string(),
            }),
        _ => Err(JsonError::InvalidType {
            expected: "Object".to_string(),
            found: node.type_name().to_string(),
        }),
    }
}

/// Get an element from an object by key (case-sensitive)
///
/// # Arguments
/// * `node` - Reference to a JsonNode that should be an object
/// * `key` - The key to search for (exact match, case-sensitive)
///
/// # Returns
/// * `Ok(&JsonNode)` - Reference to the value associated with the key
/// * `Err(JsonError::InvalidType)` - If the node is not an object
/// * `Err(JsonError::KeyNotFound)` - If the key does not exist in the object
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_object_item_case_sensitive};
///
/// let obj = JsonNode::Object(vec![
///     ("Name".to_string(), JsonNode::String("John".to_string())),
///     ("name".to_string(), JsonNode::String("Jane".to_string())),
/// ]);
/// // These are different keys due to case sensitivity
/// assert_eq!(get_object_item_case_sensitive(&obj, "Name").unwrap(), &JsonNode::String("John".to_string()));
/// assert_eq!(get_object_item_case_sensitive(&obj, "name").unwrap(), &JsonNode::String("Jane".to_string()));
/// ```
pub fn get_object_item_case_sensitive<'a>(node: &'a JsonNode, key: &str) -> Result<&'a JsonNode> {
    match node {
        JsonNode::Object(pairs) => pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .ok_or(JsonError::KeyNotFound {
                key: key.to_string(),
            }),
        _ => Err(JsonError::InvalidType {
            expected: "Object".to_string(),
            found: node.type_name().to_string(),
        }),
    }
}

/// Check if an object contains a specific key
///
/// # Arguments
/// * `node` - Reference to a JsonNode
/// * `key` - The key to check for
///
/// # Returns
/// * `true` - If the node is an object and contains the key
/// * `false` - If the node is not an object or does not contain the key
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, has_object_item};
///
/// let obj = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("John".to_string())),
///     ("age".to_string(), JsonNode::Number(30.0)),
/// ]);
/// assert!(has_object_item(&obj, "name"));
/// assert!(!has_object_item(&obj, "city"));
/// ```
pub fn has_object_item(node: &JsonNode, key: &str) -> bool {
    match node {
        JsonNode::Object(pairs) => pairs.iter().any(|(k, _)| k == key),
        _ => false,
    }
}

/// Get the string value from a JsonNode
///
/// # Arguments
/// * `node` - Reference to a JsonNode
///
/// # Returns
/// * `Some(&str)` - The string value if the node is a String type
/// * `None` - If the node is not a String type
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_string_value};
///
/// let str_node = JsonNode::String("hello".to_string());
/// assert_eq!(get_string_value(&str_node), Some("hello"));
///
/// let num_node = JsonNode::Number(42.0);
/// assert_eq!(get_string_value(&num_node), None);
/// ```
pub fn get_string_value(node: &JsonNode) -> Option<&str> {
    match node {
        JsonNode::String(s) => Some(s),
        _ => None,
    }
}

/// Get the number value from a JsonNode
///
/// # Arguments
/// * `node` - Reference to a JsonNode
///
/// # Returns
/// * `Some(f64)` - The number value if the node is a Number type
/// * `None` - If the node is not a Number type
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_number_value};
///
/// let num_node = JsonNode::Number(42.5);
/// assert_eq!(get_number_value(&num_node), Some(42.5));
///
/// let str_node = JsonNode::String("hello".to_string());
/// assert_eq!(get_number_value(&str_node), None);
/// ```
pub fn get_number_value(node: &JsonNode) -> Option<f64> {
    match node {
        JsonNode::Number(n) => Some(*n),
        _ => None,
    }
}

/// Get a value from a JSON structure using JSON Pointer (RFC 6901)
///
/// # Arguments
/// * `node` - Reference to the root JsonNode
/// * `pointer` - A JSON Pointer string (e.g., "/user/address/city")
///
/// # Returns
/// * `Ok(&JsonNode)` - Reference to the value at the specified path
/// * `Err(JsonError::InvalidType)` - If an intermediate node has the wrong type
/// * `Err(JsonError::IndexOutOfBounds)` - If an array index is out of bounds
/// * `Err(JsonError::KeyNotFound)` - If an object key does not exist
/// * `Err(JsonError::SyntaxError)` - If the pointer format is invalid
///
/// # JSON Pointer Format
/// - The pointer must start with `/`
/// - Path segments are separated by `/`
/// - `~0` represents `~` (tilde)
/// - `~1` represents `/` (forward slash)
/// - An empty pointer `/` refers to the entire document
/// - Array indices are zero-based
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_pointer};
///
/// let obj = JsonNode::Object(vec![
///     ("user".to_string(), JsonNode::Object(vec![
///         ("name".to_string(), JsonNode::String("John".to_string())),
///         ("age".to_string(), JsonNode::Number(30.0)),
///     ])),
///     ("items".to_string(), JsonNode::Array(vec![
///         JsonNode::String("item1".to_string()),
///         JsonNode::String("item2".to_string()),
///     ])),
/// ]);
///
/// // Get nested object value
/// assert_eq!(get_pointer(&obj, "/user/name").unwrap(), &JsonNode::String("John".to_string()));
///
/// // Get array element
/// assert_eq!(get_pointer(&obj, "/items/0").unwrap(), &JsonNode::String("item1".to_string()));
///
/// // Get root
/// assert_eq!(get_pointer(&obj, "/").unwrap(), &obj);
/// ```
pub fn get_pointer<'a>(node: &'a JsonNode, pointer: &str) -> Result<&'a JsonNode> {
    if pointer.is_empty() {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer cannot be empty".to_string(),
        });
    }

    if !pointer.starts_with('/') {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer must start with '/'".to_string(),
        });
    }

    // Empty pointer "/" refers to the entire document
    if pointer == "/" {
        return Ok(node);
    }

    let mut current = node;

    // Split the pointer into tokens, skipping the first empty element from the leading '/'
    for (_i, token) in pointer.split('/').skip(1).enumerate() {
        // Unescape the token according to RFC 6901
        let token = unescape_pointer_token(token);

        current = match current {
            JsonNode::Array(arr) => {
                // Try to parse the token as an array index
                let index = token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                    index: 0,
                    length: arr.len(),
                })?;
                arr.get(index)
                    .ok_or(JsonError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    })?
            }
            JsonNode::Object(pairs) => pairs
                .iter()
                .find(|(k, _)| k == &token)
                .map(|(_, v)| v)
                .ok_or(JsonError::KeyNotFound { key: token })?,
            _ => {
                return Err(JsonError::InvalidType {
                    expected: "Array or Object".to_string(),
                    found: current.type_name().to_string(),
                })
            }
        };
    }

    Ok(current)
}

/// Unescape a JSON Pointer token according to RFC 6901
///
/// - `~0` becomes `~`
/// - `~1` becomes `/`
fn unescape_pointer_token(token: &str) -> String {
    let mut result = String::with_capacity(token.len());
    let mut chars = token.chars();

    while let Some(c) = chars.next() {
        if c == '~' {
            match chars.next() {
                Some('0') => result.push('~'),
                Some('1') => result.push('/'),
                _ => {
                    // Invalid escape sequence, preserve the tilde
                    result.push('~');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Get a mutable reference to a JSON node using a JSON Pointer (RFC 6901)
///
/// # Arguments
/// * `node` - Mutable reference to the root JSON node
/// * `pointer` - The JSON Pointer string (e.g., "/users/0/name")
///
/// # Returns
/// * `Ok(&mut JsonNode)` - Mutable reference to the target node
/// * `Err(JsonError::SyntaxError)` - If the pointer is invalid
/// * `Err(JsonError::IndexOutOfBounds)` - If an array index is out of range
/// * `Err(JsonError::KeyNotFound)` - If an object key doesn't exist
/// * `Err(JsonError::InvalidType)` - If trying to traverse a non-container type
///
/// # Example
/// ```rust
/// use lx_json::{JsonNode, get_pointer_mut};
///
/// let mut obj = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("John".to_string())),
/// ]);
///
/// if let Ok(name) = get_pointer_mut(&mut obj, "/name") {
///     *name = JsonNode::String("Jane".to_string());
/// }
/// ```
pub fn get_pointer_mut<'a>(node: &'a mut JsonNode, pointer: &str) -> Result<&'a mut JsonNode> {
    if pointer.is_empty() {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer cannot be empty".to_string(),
        });
    }

    if !pointer.starts_with('/') {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer must start with '/'".to_string(),
        });
    }

    // Empty pointer "/" refers to the entire document
    if pointer == "/" {
        return Ok(node);
    }

    let tokens: Vec<String> = pointer.split('/').skip(1).map(|t| unescape_pointer_token(t)).collect();

    if tokens.is_empty() {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "Invalid pointer".to_string(),
        });
    }

    // Use a helper function to traverse and get the final mutable reference
    // We'll track the path using indices to avoid lifetime issues
    let mut current = node;

    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;

        if is_last {
            // This is the last token, return the mutable reference directly
            return match current {
                JsonNode::Array(arr) => {
                    let index = token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                        index: 0,
                        length: arr.len(),
                    })?;

                    if index >= arr.len() {
                        return Err(JsonError::IndexOutOfBounds {
                            index,
                            length: arr.len(),
                        });
                    }

                    Ok(&mut arr[index])
                }
                JsonNode::Object(pairs) => {
                    let found_idx = pairs.iter().position(|(k, _)| k == token)
                        .ok_or(JsonError::KeyNotFound { key: token.clone() })?;

                    Ok(&mut pairs[found_idx].1)
                }
                _ => Err(JsonError::InvalidType {
                    expected: "Array or Object".to_string(),
                    found: current.type_name().to_string(),
                }),
            };
        } else {
            // Not the last token, advance to the next level
            current = match current {
                JsonNode::Array(arr) => {
                    let len = arr.len();
                    let index = token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                        index: 0,
                        length: len,
                    })?;

                    if index >= len {
                        return Err(JsonError::IndexOutOfBounds {
                            index,
                            length: len,
                        });
                    }

                    arr.get_mut(index).ok_or(JsonError::IndexOutOfBounds {
                        index,
                        length: len,
                    })?
                }
                JsonNode::Object(pairs) => {
                    let found_idx = pairs.iter().position(|(k, _)| k == token)
                        .ok_or(JsonError::KeyNotFound { key: token.clone() })?;

                    &mut pairs[found_idx].1
                }
                _ => {
                    return Err(JsonError::InvalidType {
                        expected: "Array or Object".to_string(),
                        found: current.type_name().to_string(),
                    })
                }
            };
        }
    }

    // Should not reach here
    Err(JsonError::SyntaxError {
        position: 0,
        message: "Unexpected end of traversal".to_string(),
    })
}

/// Add a value at a specific JSON Pointer path
///
/// # Arguments
/// * `node` - Mutable reference to the root JSON node
/// * `pointer` - The JSON Pointer string where the value should be added
/// * `value` - The value to add
///
/// # Returns
/// * `Ok(())` - Value added successfully
/// * `Err(JsonError)` - If the path is invalid or cannot be traversed
pub fn add_value_at_pointer(node: &mut JsonNode, pointer: &str, value: JsonNode) -> Result<()> {
    if pointer.is_empty() {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer cannot be empty".to_string(),
        });
    }

    if !pointer.starts_with('/') {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer must start with '/'".to_string(),
        });
    }

    if pointer == "/" {
        return Err(JsonError::InvalidType {
            expected: "Non-root path".to_string(),
            found: "Root path".to_string(),
        });
    }

    let tokens: Vec<String> = pointer.split('/').skip(1).map(|t| unescape_pointer_token(t)).collect();

    if tokens.is_empty() {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "Invalid pointer".to_string(),
        });
    }

    let mut current = node;

    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;

        if is_last {
            // This is the last token, set the value here
            match current {
                JsonNode::Array(arr) => {
                    let index = token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                        index: 0,
                        length: arr.len(),
                    })?;

                    if index > arr.len() {
                        return Err(JsonError::IndexOutOfBounds {
                            index,
                            length: arr.len(),
                        });
                    }
                    if index == arr.len() {
                        arr.push(value.clone());
                    } else {
                        arr[index] = value.clone();
                    }
                }
                JsonNode::Object(pairs) => {
                    let found_idx = pairs.iter().position(|(k, _)| k == token);

                    if let Some(idx) = found_idx {
                        pairs[idx].1 = value.clone();
                    } else {
                        pairs.push((token.clone(), value.clone()));
                    }
                }
                _ => {
                    return Err(JsonError::InvalidType {
                        expected: "Array or Object".to_string(),
                        found: current.type_name().to_string(),
                    })
                }
            }
        } else {
            // Not the last token, advance to the next level
            current = match current {
                JsonNode::Array(arr) => {
                    let len = arr.len();
                    let index = token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                        index: 0,
                        length: len,
                    })?;

                    if index >= len {
                        return Err(JsonError::IndexOutOfBounds {
                            index,
                            length: len,
                        });
                    }

                    arr.get_mut(index).ok_or(JsonError::IndexOutOfBounds {
                        index,
                        length: len,
                    })?
                }
                JsonNode::Object(pairs) => {
                    let found_idx = pairs.iter().position(|(k, _)| k == token);

                    if let Some(idx) = found_idx {
                        &mut pairs[idx].1
                    } else {
                        return Err(JsonError::KeyNotFound { key: token.clone() });
                    }
                }
                _ => {
                    return Err(JsonError::InvalidType {
                        expected: "Array or Object".to_string(),
                        found: current.type_name().to_string(),
                    })
                }
            };
        }
    }

    Ok(())
}

/// Remove a value at a specific JSON Pointer path
///
/// # Arguments
/// * `node` - Mutable reference to the root JSON node
/// * `pointer` - The JSON Pointer string where the value should be removed
///
/// # Returns
/// * `Ok(JsonNode)` - The removed value
/// * `Err(JsonError)` - If the path is invalid or value doesn't exist
pub fn remove_at_pointer(node: &mut JsonNode, pointer: &str) -> Result<JsonNode> {
    if pointer.is_empty() {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer cannot be empty".to_string(),
        });
    }

    if !pointer.starts_with('/') {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "JSON pointer must start with '/'".to_string(),
        });
    }

    if pointer == "/" {
        return Err(JsonError::InvalidType {
            expected: "Non-root path".to_string(),
            found: "Root path".to_string(),
        });
    }

    let tokens: Vec<String> = pointer.split('/').skip(1).map(|t| unescape_pointer_token(t)).collect();

    if tokens.is_empty() {
        return Err(JsonError::SyntaxError {
            position: 0,
            message: "Invalid pointer".to_string(),
        });
    }

    if tokens.len() == 1 {
        // Simple case: remove from root
        match node {
            JsonNode::Array(arr) => {
                let token = &tokens[0];
                let index = token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                    index: 0,
                    length: arr.len(),
                })?;

                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    });
                }
                Ok(arr.remove(index))
            }
            JsonNode::Object(pairs) => {
                let token = &tokens[0];
                let found_idx = pairs.iter().position(|(k, _)| k == token)
                    .ok_or(JsonError::KeyNotFound { key: token.clone() })?;

                Ok(pairs.remove(found_idx).1)
            }
            _ => Err(JsonError::InvalidType {
                expected: "Array or Object".to_string(),
                found: node.type_name().to_string(),
            }),
        }
    } else {
        // Need to traverse to parent
        let parent_tokens = &tokens[..tokens.len() - 1];
        let last_token = &tokens[tokens.len() - 1];

        // Navigate to parent node
        let mut current = node;
        for token in parent_tokens {
            current = match current {
                JsonNode::Array(arr) => {
                    let len = arr.len();
                    let index = token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                        index: 0,
                        length: len,
                    })?;

                    if index >= len {
                        return Err(JsonError::IndexOutOfBounds {
                            index,
                            length: len,
                        });
                    }

                    arr.get_mut(index).ok_or(JsonError::IndexOutOfBounds {
                        index,
                        length: len,
                    })?
                }
                JsonNode::Object(pairs) => {
                    let found_idx = pairs.iter().position(|(k, _)| k == token)
                        .ok_or(JsonError::KeyNotFound { key: token.clone() })?;

                    &mut pairs[found_idx].1
                }
                _ => {
                    return Err(JsonError::InvalidType {
                        expected: "Array or Object".to_string(),
                        found: current.type_name().to_string(),
                    })
                }
            };
        }

        // Now current is the parent, remove from it
        match current {
            JsonNode::Array(arr) => {
                let index = last_token.parse::<usize>().map_err(|_| JsonError::IndexOutOfBounds {
                    index: 0,
                    length: arr.len(),
                })?;

                if index >= arr.len() {
                    return Err(JsonError::IndexOutOfBounds {
                        index,
                        length: arr.len(),
                    });
                }
                Ok(arr.remove(index))
            }
            JsonNode::Object(pairs) => {
                let found_idx = pairs.iter().position(|(k, _)| k == last_token)
                    .ok_or(JsonError::KeyNotFound { key: last_token.clone() })?;

                Ok(pairs.remove(found_idx).1)
            }
            _ => Err(JsonError::InvalidType {
                expected: "Array or Object".to_string(),
                found: current.type_name().to_string(),
            }),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unescape_pointer_token() {
        assert_eq!(unescape_pointer_token("simple"), "simple");
        assert_eq!(unescape_pointer_token("with~0tilde"), "with~tilde");
        assert_eq!(unescape_pointer_token("with~1slash"), "with/slash");
        assert_eq!(
            unescape_pointer_token("both~0and~1"),
            "both~and/"
        );
        assert_eq!(unescape_pointer_token("~0~1"), "~/");
    }
}