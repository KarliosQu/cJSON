//! # Utility Functions for JSON Operations
//!
//! This module provides utility functions for JSON manipulation, including
//! deep copy and comparison operations, inspired by cJSON's utility functions.

use crate::types::JsonNode;

/// Creates a copy of a JSON node.
///
/// # Arguments
///
/// * `node` - A reference to the JSON node to copy
/// * `recurse` - If true, recursively copies all child nodes (deep copy).
///   If false, only copies the current node, and arrays/objects become empty containers (shallow copy).
///
/// # Returns
///
/// A new JsonNode that is a copy of the input.
///
/// # Examples
///
/// ```
/// use lx_json::{JsonNode, duplicate};
///
/// let original = JsonNode::String("hello".to_string());
/// let copy = duplicate(&original, true);
/// assert_eq!(original, copy);
///
/// let arr = JsonNode::Array(vec![
///     JsonNode::Number(1.0),
///     JsonNode::Number(2.0),
/// ]);
/// let shallow = duplicate(&arr, false);
/// assert!(matches!(shallow, JsonNode::Array(_)));
/// assert_eq!(shallow.len(), 0); // Empty array due to shallow copy
/// ```
pub fn duplicate(node: &JsonNode, recurse: bool) -> JsonNode {
    if recurse {
        // Deep copy: use Clone trait (already implemented for JsonNode)
        node.clone()
    } else {
        // Shallow copy: copy only the current node, containers become empty
        match node {
            JsonNode::Null => JsonNode::Null,
            JsonNode::Bool(b) => JsonNode::Bool(*b),
            JsonNode::Number(n) => JsonNode::Number(*n),
            JsonNode::String(s) => JsonNode::String(s.clone()),
            JsonNode::Array(_) => JsonNode::Array(vec![]),
            JsonNode::Object(_) => JsonNode::Object(vec![]),
            JsonNode::Raw(s) => JsonNode::Raw(s.clone()),
        }
    }
}

/// Compares two JSON nodes for equality.
///
/// # Arguments
///
/// * `a` - First JSON node to compare
/// * `b` - Second JSON node to compare
/// * `case_sensitive` - If true, object key comparisons are case-sensitive.
///   If false, object key comparisons are case-insensitive.
///   String values are always compared case-sensitively regardless of this flag.
///
/// # Returns
///
/// `true` if the nodes are equal according to the comparison rules, `false` otherwise.
///
/// # Examples
///
/// ```
/// use lx_json::{JsonNode, compare};
///
/// let node1 = JsonNode::String("hello".to_string());
/// let node2 = JsonNode::String("HELLO".to_string());
///
/// // String values are always case-sensitive
/// assert!(!compare(&node1, &node2, true));
/// assert!(!compare(&node1, &node2, false));
///
/// let arr1 = JsonNode::Array(vec![
///     JsonNode::Number(1.0),
///     JsonNode::Number(2.0),
/// ]);
/// let arr2 = JsonNode::Array(vec![
///     JsonNode::Number(1.0),
///     JsonNode::Number(2.0),
/// ]);
/// assert!(compare(&arr1, &arr2, true));
/// ```
pub fn compare(a: &JsonNode, b: &JsonNode, case_sensitive: bool) -> bool {
    match (a, b) {
        // Null values
        (JsonNode::Null, JsonNode::Null) => true,

        // Boolean values
        (JsonNode::Bool(b1), JsonNode::Bool(b2)) => b1 == b2,

        // Number values (direct comparison like cJSON)
        (JsonNode::Number(n1), JsonNode::Number(n2)) => n1 == n2,

        // String values - always case-sensitive (case_sensitive flag only affects object keys)
        (JsonNode::String(s1), JsonNode::String(s2)) => s1 == s2,

        // Raw JSON values
        (JsonNode::Raw(r1), JsonNode::Raw(r2)) => {
            if case_sensitive {
                r1 == r2
            } else {
                r1.to_lowercase() == r2.to_lowercase()
            }
        }

        // Array values - compare each element
        (JsonNode::Array(arr1), JsonNode::Array(arr2)) => {
            if arr1.len() != arr2.len() {
                return false;
            }
            arr1.iter()
                .zip(arr2.iter())
                .all(|(item1, item2)| compare(item1, item2, case_sensitive))
        }

        // Object values - compare key-value pairs
        (JsonNode::Object(obj1), JsonNode::Object(obj2)) => {
            if obj1.len() != obj2.len() {
                return false;
            }

            // Compare each key-value pair
            for (key1, val1) in obj1 {
                let found = obj2.iter().any(|(key2, val2)| {
                    let keys_equal = if case_sensitive {
                        key1 == key2
                    } else {
                        key1.to_lowercase() == key2.to_lowercase()
                    };
                    keys_equal && compare(val1, val2, case_sensitive)
                });

                if !found {
                    return false;
                }
            }
            true
        }

        // Different types are not equal
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_simple() {
        // Test simple types
        let null_node = JsonNode::Null;
        assert_eq!(duplicate(&null_node, true), null_node);

        let bool_node = JsonNode::Bool(true);
        assert_eq!(duplicate(&bool_node, true), bool_node);

        let number_node = JsonNode::Number(42.0);
        assert_eq!(duplicate(&number_node, true), number_node);

        let string_node = JsonNode::String("hello".to_string());
        assert_eq!(duplicate(&string_node, true), string_node);

        let raw_node = JsonNode::Raw("raw".to_string());
        assert_eq!(duplicate(&raw_node, true), raw_node);
    }

    #[test]
    fn test_duplicate_nested() {
        // Test nested structures with deep copy
        let arr = JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::String("test".to_string()),
            JsonNode::Array(vec![JsonNode::Number(2.0)]),
        ]);

        let copied = duplicate(&arr, true);
        assert_eq!(arr, copied);

        let obj = JsonNode::Object(vec![
            ("key1".to_string(), JsonNode::String("value1".to_string())),
            ("key2".to_string(), JsonNode::Number(42.0)),
        ]);

        let copied_obj = duplicate(&obj, true);
        assert_eq!(obj, copied_obj);
    }

    #[test]
    fn test_duplicate_recurse_false() {
        // Test shallow copy (recurse = false)
        let arr = JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::String("test".to_string()),
        ]);

        let shallow = duplicate(&arr, false);
        assert!(matches!(shallow, JsonNode::Array(_)));
        assert_eq!(shallow.len(), 0);

        let obj = JsonNode::Object(vec![
            ("key".to_string(), JsonNode::String("value".to_string())),
        ]);

        let shallow_obj = duplicate(&obj, false);
        assert!(matches!(shallow_obj, JsonNode::Object(_)));
        assert_eq!(shallow_obj.len(), 0);
    }

    #[test]
    fn test_compare_equal() {
        // Test equal values
        let node1 = JsonNode::String("hello".to_string());
        let node2 = JsonNode::String("hello".to_string());
        assert!(compare(&node1, &node2, true));

        let num1 = JsonNode::Number(42.0);
        let num2 = JsonNode::Number(42.0);
        assert!(compare(&num1, &num2, true));

        let arr1 = JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]);
        let arr2 = JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]);
        assert!(compare(&arr1, &arr2, true));
    }

    #[test]
    fn test_compare_not_equal() {
        // Test unequal values
        let node1 = JsonNode::String("hello".to_string());
        let node2 = JsonNode::String("world".to_string());
        assert!(!compare(&node1, &node2, true));

        let num1 = JsonNode::Number(42.0);
        let num2 = JsonNode::Number(43.0);
        assert!(!compare(&num1, &num2, true));

        let bool1 = JsonNode::Bool(true);
        let bool2 = JsonNode::Bool(false);
        assert!(!compare(&bool1, &bool2, true));
    }

    #[test]
    fn test_compare_case_sensitive() {
        // Test case-sensitive comparison
        let node1 = JsonNode::String("hello".to_string());
        let node2 = JsonNode::String("HELLO".to_string());
        assert!(!compare(&node1, &node2, true));

        let raw1 = JsonNode::Raw("test".to_string());
        let raw2 = JsonNode::Raw("TEST".to_string());
        assert!(!compare(&raw1, &raw2, true));
    }

    #[test]
    fn test_compare_case_insensitive() {
        // String values are always compared case-sensitively
        // The case_sensitive flag only affects object keys
        let node1 = JsonNode::String("hello".to_string());
        let node2 = JsonNode::String("HELLO".to_string());
        assert!(!compare(&node1, &node2, false));

        let raw1 = JsonNode::Raw("test".to_string());
        let raw2 = JsonNode::Raw("TEST".to_string());
        assert!(compare(&raw1, &raw2, false));
    }

    #[test]
    fn test_compare_arrays() {
        // Test array comparison
        let arr1 = JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::String("test".to_string()),
        ]);
        let arr2 = JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::String("test".to_string()),
        ]);
        assert!(compare(&arr1, &arr2, true));

        // Different lengths
        let arr3 = JsonNode::Array(vec![JsonNode::Number(1.0)]);
        assert!(!compare(&arr1, &arr3, true));

        // Nested arrays
        let nested1 = JsonNode::Array(vec![
            JsonNode::Array(vec![JsonNode::Number(1.0)]),
        ]);
        let nested2 = JsonNode::Array(vec![
            JsonNode::Array(vec![JsonNode::Number(1.0)]),
        ]);
        assert!(compare(&nested1, &nested2, true));
    }

    #[test]
    fn test_compare_objects() {
        // Test object comparison
        let obj1 = JsonNode::Object(vec![
            ("key1".to_string(), JsonNode::String("value1".to_string())),
            ("key2".to_string(), JsonNode::Number(42.0)),
        ]);
        let obj2 = JsonNode::Object(vec![
            ("key1".to_string(), JsonNode::String("value1".to_string())),
            ("key2".to_string(), JsonNode::Number(42.0)),
        ]);
        assert!(compare(&obj1, &obj2, true));

        // Different values
        let obj3 = JsonNode::Object(vec![
            ("key1".to_string(), JsonNode::String("different".to_string())),
            ("key2".to_string(), JsonNode::Number(42.0)),
        ]);
        assert!(!compare(&obj1, &obj3, true));

        // Nested objects
        let nested1 = JsonNode::Object(vec![
            ("outer".to_string(), JsonNode::Object(vec![
                ("inner".to_string(), JsonNode::String("value".to_string())),
            ])),
        ]);
        let nested2 = JsonNode::Object(vec![
            ("outer".to_string(), JsonNode::Object(vec![
                ("inner".to_string(), JsonNode::String("value".to_string())),
            ])),
        ]);
        assert!(compare(&nested1, &nested2, true));
    }

    #[test]
    fn test_compare_mismatched_types() {
        // Test comparison of different types
        let null_node = JsonNode::Null;
        let bool_node = JsonNode::Bool(true);
        assert!(!compare(&null_node, &bool_node, true));

        let string_node = JsonNode::String("test".to_string());
        let number_node = JsonNode::Number(42.0);
        assert!(!compare(&string_node, &number_node, true));

        let arr_node = JsonNode::Array(vec![]);
        let obj_node = JsonNode::Object(vec![]);
        assert!(!compare(&arr_node, &obj_node, true));
    }

    #[test]
    fn test_compare_object_keys_case_sensitive() {
        // Test object key comparison with case sensitivity
        let obj1 = JsonNode::Object(vec![
            ("Key".to_string(), JsonNode::String("value".to_string())),
        ]);
        let obj2 = JsonNode::Object(vec![
            ("key".to_string(), JsonNode::String("value".to_string())),
        ]);

        // Case-sensitive: keys are different
        assert!(!compare(&obj1, &obj2, true));

        // Case-insensitive: keys are the same
        assert!(compare(&obj1, &obj2, false));
    }
}