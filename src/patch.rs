//! JSON Patch (RFC 6902) implementation
//!
//! This module provides functions to generate and apply JSON Patch operations
//! as specified in RFC 6902.

use crate::error::{JsonError, Result};
use crate::query::{add_value_at_pointer, get_pointer_mut, remove_at_pointer};
use crate::utils::duplicate;
use crate::JsonNode;

/// Generate a JSON Patch document comparing two JSON values
///
/// # Arguments
///
/// * `from` - The original JSON value
/// * `to` - The modified JSON value
/// * `case_sensitive` - Whether to use case-sensitive key matching
///
/// # Returns
///
/// A JSON array containing patch operations
///
/// # Example
///
/// ```rust
/// use lx_json::{JsonNode, patch::generate_patches};
///
/// let from = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("John".to_string())),
///     ("age".to_string(), JsonNode::Number(30.0)),
/// ]);
///
/// let to = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("Jane".to_string())),
///     ("age".to_string(), JsonNode::Number(30.0)),
/// ]);
///
/// let patches = generate_patches(&from, &to, true).unwrap();
/// ```
pub fn generate_patches(from: &JsonNode, to: &JsonNode, case_sensitive: bool) -> Result<JsonNode> {
    let mut patches = Vec::new();
    compare_and_generate("", from, to, &mut patches, case_sensitive)?;
    Ok(JsonNode::Array(patches))
}

/// Apply a JSON Patch document to a target JSON value
///
/// # Arguments
///
/// * `target` - The target JSON value to modify (mutable)
/// * `patches` - The JSON Patch document (array of patch operations)
/// * `case_sensitive` - Whether to use case-sensitive key matching
///
/// # Example
///
/// ```rust
/// use lx_json::{JsonNode, patch::apply_patches};
///
/// let mut target = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("John".to_string())),
/// ]);
///
/// let patches = JsonNode::Array(vec![
///     JsonNode::Object(vec![
///         ("op".to_string(), JsonNode::String("replace".to_string())),
///         ("path".to_string(), JsonNode::String("/name".to_string())),
///         ("value".to_string(), JsonNode::String("Jane".to_string())),
///     ]),
/// ]);
///
/// apply_patches(&mut target, &patches, true).unwrap();
/// ```
pub fn apply_patches(
    target: &mut JsonNode,
    patches: &JsonNode,
    case_sensitive: bool,
) -> Result<()> {
    match patches {
        JsonNode::Array(patch_array) => {
            for patch in patch_array {
                apply_single_patch(target, patch, case_sensitive)?;
            }
            Ok(())
        }
        _ => Err(JsonError::InvalidType {
            expected: "Array".to_string(),
            found: patches.type_name().to_string(),
        }),
    }
}

/// Add a single patch operation to a patch array
///
/// # Arguments
///
/// * `patches` - The patch array to add to (mutable)
/// * `op` - The operation type ("add", "remove", "replace", "move", "copy", "test")
/// * `path` - The JSON Pointer path
/// * `value` - Optional value for the operation
///
/// # Example
///
/// ```rust
/// use lx_json::{JsonNode, patch::add_patch_to_array};
///
/// let mut patches = JsonNode::Array(vec![]);
///
/// add_patch_to_array(
///     &mut patches,
///     "add",
///     "/baz",
///     Some(&JsonNode::String("qux".to_string())),
/// ).unwrap();
/// ```
pub fn add_patch_to_array(
    patches: &mut JsonNode,
    op: &str,
    path: &str,
    value: Option<&JsonNode>,
) -> Result<()> {
    match patches {
        JsonNode::Array(ref mut patch_array) => {
            let mut patch_obj = vec![
                ("op".to_string(), JsonNode::String(op.to_string())),
                ("path".to_string(), JsonNode::String(path.to_string())),
            ];

            if let Some(v) = value {
                patch_obj.push(("value".to_string(), duplicate(v, true)));
            }

            patch_array.push(JsonNode::Object(patch_obj));
            Ok(())
        }
        _ => Err(JsonError::InvalidType {
            expected: "Array".to_string(),
            found: patches.type_name().to_string(),
        }),
    }
}

/// Recursively compare two JSON values and generate patch operations
fn compare_and_generate(
    path: &str,
    from: &JsonNode,
    to: &JsonNode,
    patches: &mut Vec<JsonNode>,
    case_sensitive: bool,
) -> Result<()> {
    match (from, to) {
        // Both null - no change needed
        (JsonNode::Null, JsonNode::Null) => Ok(()),

        // Both bools
        (JsonNode::Bool(a), JsonNode::Bool(b)) => {
            if a != b {
                patches.push(generate_replace_patch(path, to)?);
            }
            Ok(())
        }

        // Both numbers
        (JsonNode::Number(a), JsonNode::Number(b)) => {
            if a != b {
                patches.push(generate_replace_patch(path, to)?);
            }
            Ok(())
        }

        // Both strings
        (JsonNode::String(a), JsonNode::String(b)) => {
            if a != b {
                patches.push(generate_replace_patch(path, to)?);
            }
            Ok(())
        }

        // Both arrays - compare element by element
        (JsonNode::Array(arr_a), JsonNode::Array(arr_b)) => {
            let len_a = arr_a.len();
            let len_b = arr_b.len();

            for i in 0..len_a.min(len_b) {
                let new_path = format!("{}/{}", path, i);
                compare_and_generate(&new_path, &arr_a[i], &arr_b[i], patches, case_sensitive)?;
            }

            // Handle removal of extra elements
            for i in (len_b..len_a).rev() {
                let new_path = format!("{}/{}", path, i);
                patches.push(generate_remove_patch(&new_path));
            }

            // Handle addition of new elements
            for i in len_a..len_b {
                let new_path = format!("{}/{}", path, i);
                patches.push(generate_add_patch(&new_path, &arr_b[i])?);
            }

            Ok(())
        }

        // Both objects - compare key by key
        (JsonNode::Object(obj_a), JsonNode::Object(obj_b)) => {
            let map_a: std::collections::HashMap<String, JsonNode> =
                obj_a.iter().cloned().collect();
            let map_b: std::collections::HashMap<String, JsonNode> =
                obj_b.iter().cloned().collect();

            let keys_a: std::collections::HashSet<String> = map_a.keys().cloned().collect();
            let keys_b: std::collections::HashSet<String> = map_b.keys().cloned().collect();

            // Check for removed keys
            for key in &keys_a {
                if !keys_b.contains(key) {
                    let new_path = format!("{}/{}", path, escape_pointer(key));
                    patches.push(generate_remove_patch(&new_path));
                }
            }

            // Check for added and modified keys
            for key in &keys_b {
                let new_path = format!("{}/{}", path, escape_pointer(key));
                if let Some(val_a) = map_a.get(key) {
                    let val_b = &map_b[key];
                    compare_and_generate(&new_path, val_a, val_b, patches, case_sensitive)?;
                } else {
                    patches.push(generate_add_patch(&new_path, &map_b[key])?);
                }
            }

            Ok(())
        }

        // Different types - replace
        _ => {
            patches.push(generate_replace_patch(path, to)?);
            Ok(())
        }
    }
}

/// Apply a single patch operation
fn apply_single_patch(
    target: &mut JsonNode,
    patch: &JsonNode,
    _case_sensitive: bool,
) -> Result<()> {
    let op = patch
        .get("op")
        .and_then(|v| v.as_string())
        .ok_or_else(|| JsonError::InvalidType {
            expected: "String".to_string(),
            found: "missing op".to_string(),
        })?;

    let path = patch
        .get("path")
        .and_then(|v| v.as_string())
        .ok_or_else(|| JsonError::InvalidType {
            expected: "String".to_string(),
            found: "missing path".to_string(),
        })?;

    match op {
        "add" => {
            let value = patch
                .get("value")
                .ok_or_else(|| JsonError::InvalidType {
                    expected: "Object".to_string(),
                    found: "missing value".to_string(),
                })?;
            add_value_at_pointer(target, path, duplicate(value, true))
        }
        "remove" => {
            remove_at_pointer(target, path)?;
            Ok(())
        }
        "replace" => {
            let value = patch
                .get("value")
                .ok_or_else(|| JsonError::InvalidType {
                    expected: "Object".to_string(),
                    found: "missing value".to_string(),
                })?;
            remove_at_pointer(target, path)?;
            add_value_at_pointer(target, path, duplicate(value, true))
        }
        "move" => {
            let from_path = patch
                .get("from")
                .and_then(|v| v.as_string())
                .ok_or_else(|| JsonError::InvalidType {
                    expected: "String".to_string(),
                    found: "missing from".to_string(),
                })?;
            let value = get_pointer_mut(target, from_path)?;
            let value_copy = duplicate(value, true);
            remove_at_pointer(target, from_path)?;
            add_value_at_pointer(target, path, value_copy)
        }
        "copy" => {
            let from_path = patch
                .get("from")
                .and_then(|v| v.as_string())
                .ok_or_else(|| JsonError::InvalidType {
                    expected: "String".to_string(),
                    found: "missing from".to_string(),
                })?;
            // Clone the value first to avoid borrow conflicts
            let value_clone = {
                let value = get_pointer_mut(target, from_path)?;
                duplicate(value, true)
            };
            add_value_at_pointer(target, path, value_clone)
        }
        "test" => {
            let value = patch
                .get("value")
                .ok_or_else(|| JsonError::InvalidType {
                    expected: "Object".to_string(),
                    found: "missing value".to_string(),
                })?;
            let current = get_pointer_mut(target, path)?;
            if current != value {
                return Err(JsonError::PatchTestFailed {
                    path: path.to_string(),
                    expected: duplicate(value, true),
                    found: duplicate(current, true),
                });
            }
            Ok(())
        }
        _ => Err(JsonError::InvalidPatchOperation {
            operation: op.to_string(),
        }),
    }
}

/// Generate an "add" patch
fn generate_add_patch(path: &str, value: &JsonNode) -> Result<JsonNode> {
    Ok(JsonNode::Object(vec![
        ("op".to_string(), JsonNode::String("add".to_string())),
        ("path".to_string(), JsonNode::String(path.to_string())),
        ("value".to_string(), duplicate(value, true)),
    ]))
}

/// Generate a "remove" patch
fn generate_remove_patch(path: &str) -> JsonNode {
    JsonNode::Object(vec![
        ("op".to_string(), JsonNode::String("remove".to_string())),
        ("path".to_string(), JsonNode::String(path.to_string())),
    ])
}

/// Generate a "replace" patch
fn generate_replace_patch(path: &str, value: &JsonNode) -> Result<JsonNode> {
    Ok(JsonNode::Object(vec![
        ("op".to_string(), JsonNode::String("replace".to_string())),
        ("path".to_string(), JsonNode::String(path.to_string())),
        ("value".to_string(), duplicate(value, true)),
    ]))
}

/// Escape special characters in JSON Pointer tokens
fn escape_pointer(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_patches_simple_replace() {
        let from = JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
        ]);
        let to = JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("Jane".to_string())),
        ]);

        let patches = generate_patches(&from, &to, true).unwrap();
        assert_eq!(patches.len(), 1);

        if let JsonNode::Array(arr) = patches {
            let patch = &arr[0];
            assert_eq!(patch.get("op").unwrap(), &JsonNode::String("replace".to_string()));
            assert_eq!(patch.get("path").unwrap(), &JsonNode::String("/name".to_string()));
        }
    }

    #[test]
    fn test_generate_patches_add_remove() {
        let from = JsonNode::Object(vec![
            ("a".to_string(), JsonNode::Number(1.0)),
        ]);
        let to = JsonNode::Object(vec![
            ("b".to_string(), JsonNode::Number(2.0)),
        ]);

        let patches = generate_patches(&from, &to, true).unwrap();
        assert_eq!(patches.len(), 2);
    }

    #[test]
    fn test_apply_patches_add() {
        let mut target = JsonNode::Object(vec![]);
        let patches = JsonNode::Array(vec![
            JsonNode::Object(vec![
                ("op".to_string(), JsonNode::String("add".to_string())),
                ("path".to_string(), JsonNode::String("/name".to_string())),
                ("value".to_string(), JsonNode::String("John".to_string())),
            ]),
        ]);

        assert!(apply_patches(&mut target, &patches, true).is_ok());
    }

    #[test]
    fn test_apply_patches_remove() {
        let mut target = JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
        ]);
        let patches = JsonNode::Array(vec![
            JsonNode::Object(vec![
                ("op".to_string(), JsonNode::String("remove".to_string())),
                ("path".to_string(), JsonNode::String("/name".to_string())),
            ]),
        ]);

        assert!(apply_patches(&mut target, &patches, true).is_ok());
    }

    #[test]
    fn test_apply_patches_invalid_operation() {
        let mut target = JsonNode::Object(vec![]);
        let patches = JsonNode::Array(vec![
            JsonNode::Object(vec![
                ("op".to_string(), JsonNode::String("invalid".to_string())),
                ("path".to_string(), JsonNode::String("/name".to_string())),
            ]),
        ]);

        assert!(apply_patches(&mut target, &patches, true).is_err());
    }

    #[test]
    fn test_add_patch_to_array() {
        let mut patches = JsonNode::Array(vec![]);
        let result = add_patch_to_array(
            &mut patches,
            "add",
            "/baz",
            Some(&JsonNode::String("qux".to_string())),
        );

        assert!(result.is_ok());
        if let JsonNode::Array(arr) = patches {
            assert_eq!(arr.len(), 1);
        }
    }
}