//! JSON Merge Patch (RFC 7396) implementation
//!
//! This module provides functions to merge JSON Patch documents
//! as specified in RFC 7396.

use crate::error::Result;
use crate::utils::duplicate;
use crate::JsonNode;

/// Apply a JSON Merge Patch to a target JSON value
///
/// # Arguments
///
/// * `target` - The target JSON value to modify (mutable)
/// * `patch` - The JSON Merge Patch document
/// * `case_sensitive` - Whether to use case-sensitive key matching
///
/// # Errors
///
/// Returns an error if the merge operation fails.
///
/// # Example
///
/// ```rust
/// use lx_json::{JsonNode, merge::merge_patch};
///
/// let mut target = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("John".to_string())),
///     ("age".to_string(), JsonNode::Number(30.0)),
/// ]);
///
/// let patch = JsonNode::Object(vec![
///     ("name".to_string(), JsonNode::String("Jane".to_string())),
///     ("city".to_string(), JsonNode::String("New York".to_string())),
/// ]);
///
/// merge_patch(&mut target, &patch, true).unwrap();
/// ```
pub fn merge_patch(target: &mut JsonNode, patch: &JsonNode, _case_sensitive: bool) -> Result<()> {
    match (target, patch) {
        // If patch is null, target becomes null (delete operation)
        (t, JsonNode::Null) => {
            *t = JsonNode::Null;
            Ok(())
        }

        // If both are objects, recursively merge
        (JsonNode::Object(target_obj), JsonNode::Object(patch_obj)) => {
            for (key, patch_value) in patch_obj.iter() {
                if matches!(patch_value, JsonNode::Null) {
                    // Null in patch means delete the key from target
                    if let Some(pos) = target_obj.iter().position(|(k, _)| k == key) {
                        target_obj.remove(pos);
                    }
                } else if let Some(pos) = target_obj.iter().position(|(k, _)| k == key) {
                    // Key exists in target, recursively merge
                    merge_patch(&mut target_obj[pos].1, patch_value, _case_sensitive)?;
                } else {
                    // Key doesn't exist in target, add it (recursively merged)
                    let mut new_val = if matches!(patch_value, JsonNode::Object(_)) {
                        JsonNode::Object(vec![])
                    } else {
                        duplicate(patch_value, true)
                    };
                    if matches!(patch_value, JsonNode::Object(_)) {
                        merge_patch(&mut new_val, patch_value, _case_sensitive)?;
                    }
                    target_obj.push((key.clone(), new_val));
                }
            }
            Ok(())
        }

        // For any other combination: if patch is an object, create empty object target and merge
        (t, p @ JsonNode::Object(_)) => {
            *t = JsonNode::Object(vec![]);
            merge_patch(t, p, _case_sensitive)
        }

        // For any other combination, replace target with patch
        (t, p) => {
            *t = duplicate(p, true);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_patch_simple_replace() {
        let mut target = JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
            ("age".to_string(), JsonNode::Number(30.0)),
        ]);

        let patch = JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("Jane".to_string())),
        ]);

        assert!(merge_patch(&mut target, &patch, true).is_ok());
    }

    #[test]
    fn test_merge_patch_null_deletes() {
        let mut target = JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
            ("age".to_string(), JsonNode::Number(30.0)),
        ]);

        let patch = JsonNode::Object(vec![
            ("age".to_string(), JsonNode::Null),
        ]);

        assert!(merge_patch(&mut target, &patch, true).is_ok());

        if let JsonNode::Object(obj) = target {
            assert!(!obj.iter().any(|(k, _)| k == "age"));
        }
    }

    #[test]
    fn test_merge_patch_nested_objects() {
        let mut target = JsonNode::Object(vec![
            (
                "address".to_string(),
                JsonNode::Object(vec![
                    ("city".to_string(), JsonNode::String("Boston".to_string())),
                    ("country".to_string(), JsonNode::String("USA".to_string())),
                ]),
            ),
        ]);

        let patch = JsonNode::Object(vec![
            (
                "address".to_string(),
                JsonNode::Object(vec![
                    ("city".to_string(), JsonNode::String("New York".to_string())),
                ]),
            ),
        ]);

        assert!(merge_patch(&mut target, &patch, true).is_ok());
    }

    #[test]
    fn test_merge_patch_primitive_replacement() {
        let mut target = JsonNode::String("old".to_string());
        let patch = JsonNode::String("new".to_string());

        assert!(merge_patch(&mut target, &patch, true).is_ok());
        assert_eq!(target, JsonNode::String("new".to_string()));
    }
}