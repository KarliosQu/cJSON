//! FFI tests for C ABI layer
//!
//! This file contains unit tests for the C ABI functions in src/ffi.rs

use lx_json::ffi::*;
use std::ffi::{CString, CStr};
use std::ptr;

/// Test 1: Basic parsing and serialization through C ABI
#[test]
fn test_ffi_parse_and_print() {
    let json_str = r#"{"name": "John", "age": 30, "city": "New York"}"#;
    let c_json = CString::new(json_str).unwrap();
    
    let node = unsafe { lx_json_parse(c_json.as_ptr()) };
    assert!(!node.is_null(), "Parsing should succeed");
    
    let printed = unsafe { lx_json_print(node) };
    assert!(!printed.is_null(), "Print should succeed");
    
    let result_str = unsafe { CStr::from_ptr(printed).to_str().unwrap() };
    assert!(result_str.contains("name"));
    assert!(result_str.contains("John"));
    assert!(result_str.contains("age"));
    assert!(result_str.contains("30"));
    
    unsafe {
        lx_json_free_string(printed);
        lx_json_free(node);
    }
}

/// Test 2: Type checks and constructors through C ABI
#[test]
fn test_ffi_type_checks_and_constructors() {
    unsafe {
        let null_node = lx_json_create_null();
        assert!(lx_json_is_null(null_node) != 0, "Should be null");
        assert!(lx_json_is_bool(null_node) == 0, "Should not be bool");
        lx_json_free(null_node);
        
        let bool_node = lx_json_create_bool(1);
        assert!(lx_json_is_bool(bool_node) != 0, "Should be bool");
        assert!(lx_json_is_true(bool_node) != 0, "Should be true");
        assert!(lx_json_get_bool(bool_node) != 0, "Should get true");
        lx_json_free(bool_node);
        
        let false_node = lx_json_create_false();
        assert!(lx_json_is_false(false_node) != 0, "Should be false");
        assert!(lx_json_get_bool(false_node) == 0, "Should get false");
        lx_json_free(false_node);
        
        let num_node = lx_json_create_number(42.5);
        assert!(lx_json_is_number(num_node) != 0, "Should be number");
        assert!(lx_json_get_number(num_node) == 42.5, "Should get 42.5");
        lx_json_free(num_node);
        
        let c_string = CString::new("Hello").unwrap();
        let str_node = lx_json_create_string(c_string.as_ptr());
        assert!(lx_json_is_string(str_node) != 0, "Should be string");
        let str_val = lx_json_get_string_value(str_node);
        assert!(!str_val.is_null(), "String value should not be null");
        let result = CStr::from_ptr(str_val).to_str().unwrap();
        assert_eq!(result, "Hello");
        lx_json_free_string(str_val);
        lx_json_free(str_node);
        
        let arr_node = lx_json_create_array();
        assert!(lx_json_is_array(arr_node) != 0, "Should be array");
        assert!(lx_json_get_array_size(arr_node) == 0, "Should be empty");
        lx_json_free(arr_node);
        
        let obj_node = lx_json_create_object();
        assert!(lx_json_is_object(obj_node) != 0, "Should be object");
        lx_json_free(obj_node);
    }
}

/// Test 3: Array and object operations through C ABI
#[test]
fn test_ffi_array_and_object_operations() {
    unsafe {
        let arr = lx_json_create_array();
        
        let item1 = lx_json_create_number(1.0);
        let item2 = lx_json_create_number(2.0);
        
        let result = lx_json_add_item_to_array(arr, item1);
        assert!(result != 0, "Add should succeed");
        
        let result = lx_json_add_item_to_array(arr, item2);
        assert!(result != 0, "Add should succeed");
        
        assert_eq!(lx_json_get_array_size(arr), 2, "Array should have 2 items");
        
        let retrieved = lx_json_get_array_item(arr, 1);
        assert!(!retrieved.is_null(), "Retrieved should not be null");
        assert_eq!(lx_json_get_number(retrieved), 2.0, "Should get 2.0");
        lx_json_free(retrieved);
        
        let detached = lx_json_detach_item_from_array(arr, 0);
        assert!(!detached.is_null(), "Detached should not be null");
        assert_eq!(lx_json_get_number(detached), 1.0, "Should be 1.0");
        lx_json_free(detached);
        assert_eq!(lx_json_get_array_size(arr), 1, "Array should have 1 item");
        
        let new_item = lx_json_create_number(99.0);
        let result = lx_json_replace_item_in_array(arr, 0, new_item);
        assert!(result != 0, "Replace should succeed");
        
        let replaced = lx_json_get_array_item(arr, 0);
        assert_eq!(lx_json_get_number(replaced), 99.0, "Should be 99.0");
        lx_json_free(replaced);
        
        let obj = lx_json_create_object();
        let c_key = CString::new("name").unwrap();
        let c_value = CString::new("Alice").unwrap();
        let value_node = lx_json_create_string(c_value.as_ptr());
        
        let result = lx_json_add_item_to_object(obj, c_key.as_ptr(), value_node);
        assert!(result != 0, "Add should succeed");
        
        assert!(lx_json_has_object_item(obj, c_key.as_ptr()) != 0, "Should have key");
        
        let retrieved = lx_json_get_object_item(obj, c_key.as_ptr());
        assert!(!retrieved.is_null(), "Retrieved should not be null");
        assert!(lx_json_is_string(retrieved) != 0, "Should be string");
        lx_json_free(retrieved);
        
        lx_json_free(obj);
        lx_json_free(arr);
    }
}

/// Test 4: JSON Pointer query through C ABI
#[test]
fn test_ffi_json_pointer() {
    let json_str = r#"{"user": {"name": "Bob", "age": 25, "address": {"city": "Boston"}}}"#;
    let c_json = CString::new(json_str).unwrap();
    
    let node = unsafe { lx_json_parse(c_json.as_ptr()) };
    assert!(!node.is_null());
    
    let c_pointer = CString::new("/user/name").unwrap();
    let name_val = unsafe { lx_json_get_pointer(node, c_pointer.as_ptr()) };
    assert!(!name_val.is_null(), "Pointer should find value");
    assert!(lx_json_is_string(name_val) != 0, "Should be string");
    
    let name_str = unsafe { lx_json_get_string_value(name_val) };
    assert!(!name_str.is_null());
    let result = unsafe { CStr::from_ptr(name_str).to_str().unwrap() };
    assert_eq!(result, "Bob");
    unsafe { lx_json_free_string(name_str); }
    lx_json_free(name_val);
    
    let c_nested_pointer = CString::new("/user/address/city").unwrap();
    let city_val = unsafe { lx_json_get_pointer(node, c_nested_pointer.as_ptr()) };
    assert!(!city_val.is_null(), "Nested pointer should find value");
    
    let city_str = unsafe { lx_json_get_string_value(city_val) };
    assert!(!city_str.is_null());
    let result = unsafe { CStr::from_ptr(city_str).to_str().unwrap() };
    assert_eq!(result, "Boston");
    unsafe { lx_json_free_string(city_str); }
    lx_json_free(city_val);
    
    let c_invalid_pointer = CString::new("/invalid/path").unwrap();
    let invalid_val = unsafe { lx_json_get_pointer(node, c_invalid_pointer.as_ptr()) };
    assert!(invalid_val.is_null(), "Invalid pointer should return null");
    
    unsafe { lx_json_free(node); }
}

/// Test 5: Memory management and NULL safety through C ABI
#[test]
fn test_ffi_memory_management_and_null_safety() {
    unsafe {
        let null_result = lx_json_parse(ptr::null());
        assert!(null_result.is_null(), "Parse with null should return null");
        
        let error = lx_json_get_last_error();
        assert!(!error.is_null(), "Should have error set");
        let error_msg = CStr::from_ptr(error).to_str().unwrap();
        assert!(error_msg.contains("non-null") || error_msg.contains("null"), "Error should mention null");
        lx_json_free_string(error);
        
        let null_str_result = lx_json_print(ptr::null_mut());
        assert!(null_str_result.is_null(), "Print null node should return null");
        
        lx_json_create_null();
        lx_json_free_string(ptr::null_mut());
        lx_json_free(ptr::null_mut());
        
        let arr = lx_json_create_array();
        let result = lx_json_add_item_to_array(arr, ptr::null_mut());
        assert!(result == 0, "Add null item should fail");
        assert!(!lx_json_get_last_error().is_null(), "Should have error set");
        lx_json_free_string(ptr::null_mut());
        
        let obj = lx_json_create_object();
        let c_key = CString::new("key").unwrap();
        let result = lx_json_add_item_to_object(obj, c_key.as_ptr(), ptr::null_mut());
        assert!(result == 0, "Add null item to object should fail");
        
        lx_json_free(arr);
        lx_json_free(obj);
    }
}

/// Test 6: JSON Patch and Merge functionality through C ABI
#[test]
fn test_ffi_patch_and_merge() {
    let source_json = r#"{"name": "Charlie", "age": 35}"#;
    let target_json = r#"{"name": "Charlie", "age": 36, "city": "Seattle"}"#;
    
    let c_source = CString::new(source_json).unwrap();
    let c_target = CString::new(target_json).unwrap();
    
    let source_node = unsafe { lx_json_parse(c_source.as_ptr()) };
    let target_node = unsafe { lx_json_parse(c_target.as_ptr()) };
    
    assert!(!source_node.is_null());
    assert!(!target_node.is_null());
    
    let patches = unsafe { lx_json_generate_patches(source_node, target_node) };
    assert!(!patches.is_null());
    assert!(lx_json_is_array(patches) != 0, "Patches should be an array");
    
    let patch_size = unsafe { lx_json_get_array_size(patches) };
    assert!(patch_size > 0, "Should have at least one patch");
    
    let patched = unsafe { lx_json_apply_patches(source_node, patches) };
    assert!(!patched.is_null());
    
    let c_name = CString::new("name").unwrap();
    let name_val = unsafe { lx_json_get_object_item(patched, c_name.as_ptr()) };
    assert!(!name_val.is_null());
    let name = unsafe { lx_json_get_string_value(name_val) };
    assert!(!name.is_null());
    let name_result = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    assert_eq!(name_result, "Charlie");
    unsafe {
        lx_json_free_string(name);
        lx_json_free(name_val);
    }
    
    let c_city = CString::new("city").unwrap();
    let city_val = unsafe { lx_json_get_object_item(patched, c_city.as_ptr()) };
    assert!(!city_val.is_null(), "Patched should have city");
    unsafe {
        lx_json_free(city_val);
    }
    
    let merge_json = r#"{"job": "Engineer"}"#;
    let c_merge = CString::new(merge_json).unwrap();
    let merge_node = unsafe { lx_json_parse(c_merge.as_ptr()) };
    assert!(!merge_node.is_null());
    
    let merged = unsafe { lx_json_merge_patch(patched, merge_node) };
    assert!(!merged.is_null());
    
    let c_job = CString::new("job").unwrap();
    let job_val = unsafe { lx_json_get_object_item(merged, c_job.as_ptr()) };
    assert!(!job_val.is_null(), "Merged should have job");
    let job = unsafe { lx_json_get_string_value(job_val) };
    assert!(!job.is_null());
    let job_result = unsafe { CStr::from_ptr(job).to_str().unwrap() };
    assert_eq!(job_result, "Engineer");
    unsafe {
        lx_json_free_string(job);
        lx_json_free(job_val);
        lx_json_free(merged);
        lx_json_free(merge_node);
        lx_json_free(patched);
        lx_json_free(patches);
        lx_json_free(target_node);
        lx_json_free(source_node);
    }
}

/// Test 7: lx_json_print_buffered function
#[test]
fn test_ffi_print_buffered() {
    let json_str = r#"{"name": "Alice", "age": 25}"#;
    let c_json = CString::new(json_str).unwrap();
    
    let node = unsafe { lx_json_parse(c_json.as_ptr()) };
    assert!(!node.is_null(), "Parsing should succeed");
    
    // Test with different prebuffer sizes
    let printed_small = unsafe { lx_json_print_buffered(node, 10, 1) };
    assert!(!printed_small.is_null(), "Print buffered with small prebuffer should succeed");
    let result_small = unsafe { CStr::from_ptr(printed_small).to_str().unwrap() };
    assert!(result_small.contains("name"));
    assert!(result_small.contains("Alice"));
    unsafe { lx_json_free_string(printed_small); }
    
    let printed_large = unsafe { lx_json_print_buffered(node, 1000, 1) };
    assert!(!printed_large.is_null(), "Print buffered with large prebuffer should succeed");
    let result_large = unsafe { CStr::from_ptr(printed_large).to_str().unwrap() };
    assert!(result_large.contains("name"));
    unsafe { lx_json_free_string(printed_large); }
    
    // Test without formatting
    let printed_unfmt = unsafe { lx_json_print_buffered(node, 50, 0) };
    assert!(!printed_unfmt.is_null(), "Print buffered unformatted should succeed");
    let result_unfmt = unsafe { CStr::from_ptr(printed_unfmt).to_str().unwrap() };
    assert!(!result_unfmt.contains('\n'), "Should not have newlines");
    unsafe { lx_json_free_string(printed_unfmt); }
    
    unsafe { lx_json_free(node); }
}

/// Test 8: lx_json_print_preallocated function
#[test]
fn test_ffi_print_preallocated() {
    let json_str = r#"{"test": "value"}"#;
    let c_json = CString::new(json_str).unwrap();
    
    let node = unsafe { lx_json_parse(c_json.as_ptr()) };
    assert!(!node.is_null(), "Parsing should succeed");
    
    // First query required length
    let mut required_length: usize = 0;
    let result = unsafe { lx_json_print_preallocated(node, ptr::null_mut(), 0, 1, &mut required_length) };
    assert_eq!(result, 0, "Should return 0 when buffer is NULL");
    assert!(required_length > 0, "Should set required_length");
    
    // Allocate exact size buffer
    let mut buffer: Vec<std::os::raw::c_char> = 
        vec![0; required_length + 1];
    
    // Print into the buffer
    let result = unsafe { lx_json_print_preallocated(
        node,
        buffer.as_mut_ptr(),
        buffer.len(),
        1,
        &mut required_length
    ) };
    assert_eq!(result, 1, "Should succeed with correct buffer size");
    
    // Verify output
    let output = unsafe { CStr::from_ptr(buffer.as_ptr()).to_str().unwrap() };
    assert!(output.contains("test"));
    assert!(output.contains("value"));
    
    unsafe { lx_json_free(node); }
}

/// Test 9: lx_json_add_patch_to_array function
#[test]
fn test_ffi_add_patch_to_array() {
    // Create a patch array
    let patches = unsafe { lx_json_create_array() };
    assert!(!patches.is_null(), "Should create patch array");
    
    // Add add operation
    let c_op_add = CString::new("add").unwrap();
    let c_path1 = CString::new("/foo").unwrap();
    let value = unsafe { lx_json_create_number(42.0) };
    assert!(!value.is_null());
    
    let result = unsafe { lx_json_add_patch_to_array(
        patches,
        c_op_add.as_ptr(),
        c_path1.as_ptr(),
        value
    ) };
    assert_eq!(result, 1, "Should successfully add patch operation");
    
    // Add remove operation (no value needed)
    let c_op_remove = CString::new("remove").unwrap();
    let c_path2 = CString::new("/bar").unwrap();
    
    let result = unsafe { lx_json_add_patch_to_array(
        patches,
        c_op_remove.as_ptr(),
        c_path2.as_ptr(),
        ptr::null()
    ) };
    assert_eq!(result, 1, "Should successfully add remove operation");
    
    // Verify array size
    let size = unsafe { lx_json_get_array_size(patches) };
    assert_eq!(size, 2, "Patch array should have 2 elements");
    
    // Get first patch and verify structure
    let first_patch = unsafe { lx_json_get_array_item(patches, 0) };
    assert!(!first_patch.is_null());
    
    let c_op_key = CString::new("op").unwrap();
    let op_node = unsafe { lx_json_get_object_item(first_patch, c_op_key.as_ptr()) };
    assert!(!op_node.is_null());
    let op_val = unsafe { lx_json_get_string_value(op_node) };
    assert!(!op_val.is_null());
    let op_str = unsafe { CStr::from_ptr(op_val).to_str().unwrap() };
    assert_eq!(op_str, "add");
    
    let c_path_key = CString::new("path").unwrap();
    let path_node = unsafe { lx_json_get_object_item(first_patch, c_path_key.as_ptr()) };
    assert!(!path_node.is_null());
    let path_val = unsafe { lx_json_get_string_value(path_node) };
    assert!(!path_val.is_null());
    let path_str = unsafe { CStr::from_ptr(path_val).to_str().unwrap() };
    assert_eq!(path_str, "/foo");
    
    unsafe {
        lx_json_free_string(op_val);
        lx_json_free_string(path_val);
        lx_json_free(first_patch);
        lx_json_free(value);
        lx_json_free(patches);
    }
}

/// Test 10: lx_json_get_number_value function
#[test]
fn test_ffi_get_number_value() {
    // Test with number node
    let number_node = unsafe { lx_json_create_number(42.5) };
    assert!(!number_node.is_null());
    
    let value = unsafe { lx_json_get_number_value(number_node) };
    assert_eq!(value, 42.5, "Should get correct number value");
    
    unsafe { lx_json_free(number_node); }
    
    // Test with non-number node
    let string_node = unsafe { lx_json_create_string(CString::new("hello").unwrap().as_ptr()) };
    assert!(!string_node.is_null());
    
    let value = unsafe { lx_json_get_number_value(string_node) };
    assert_eq!(value, 0.0, "Should return 0.0 for non-number node");
    
    unsafe { lx_json_free(string_node); }
    
    // Test with NULL node
    let value = unsafe { lx_json_get_number_value(ptr::null()) };
    assert_eq!(value, 0.0, "Should return 0.0 for NULL node");
}

/// Test 11: lx_json_get_pointer_mut function
#[test]
fn test_ffi_get_pointer_mut() {
    // Create a simple object
    let json_str = r#"{"name": "John", "age": 30}"#;
    let c_json = CString::new(json_str).unwrap();
    
    let node = unsafe { lx_json_parse(c_json.as_ptr()) };
    assert!(!node.is_null(), "Parsing should succeed");
    
    // Test with root pointer "/"
    let c_root = CString::new("/").unwrap();
    let result = unsafe { lx_json_get_pointer_mut(node, c_root.as_ptr()) };
    assert_eq!(result, node, "Should return the original node for root pointer");
    
    // Test with non-root pointer (should return NULL)
    let c_path = CString::new("/name").unwrap();
    let result = unsafe { lx_json_get_pointer_mut(node, c_path.as_ptr()) };
    assert!(result.is_null(), "Should return NULL for non-root pointer");
    
    // Test with NULL node
    let result = unsafe { lx_json_get_pointer_mut(ptr::null_mut(), c_root.as_ptr()) };
    assert!(result.is_null(), "Should return NULL for NULL node");
    
    // Test with NULL pointer
    let result = unsafe { lx_json_get_pointer_mut(node, ptr::null()) };
    assert!(result.is_null(), "Should return NULL for NULL pointer");
    
    unsafe { lx_json_free(node); }
}
/// Test: Sort object keys through C ABI
#[test]
fn test_ffi_sort_object() {
    unsafe {
        let obj = lx_json_create_object();
        let key1 = CString::new("zebra").unwrap();
        let key2 = CString::new("apple").unwrap();
        let key3 = CString::new("banana").unwrap();
        let val = lx_json_create_number(1.0);
        
        lx_json_add_item_to_object(obj, key1.as_ptr(), val);
        let val2 = lx_json_create_number(2.0);
        lx_json_add_item_to_object(obj, key2.as_ptr(), val2);
        let val3 = lx_json_create_number(3.0);
        lx_json_add_item_to_object(obj, key3.as_ptr(), val3);
        
        // Test case-sensitive sort
        let result = lx_json_sort_object(obj, 1);
        assert_eq!(result, 1, "Sort should succeed");
        
        // Verify order
        let keys = lx_json_get_object_keys(obj);
        assert!(!keys.is_null());
        let first_key = lx_json_get_array_item(keys, 0);
        let key_str = lx_json_get_string_value(first_key);
        let key_result = CStr::from_ptr(key_str).to_str().unwrap();
        assert_eq!(key_result, "apple");
        
        lx_json_free_string(key_str);
        lx_json_free(first_key);
        lx_json_free(keys);
        lx_json_free(obj);
    }
}

/// Test: Create int array through C ABI
#[test]
fn test_ffi_create_int_array() {
    unsafe {
        let values = vec![1i64, 2, 3, 4, 5];
        let arr = lx_json_create_int_array(values.as_ptr(), values.len());
        assert!(!arr.is_null(), "Array creation should succeed");
        assert_eq!(lx_json_get_array_size(arr), 5);
        
        let item = lx_json_get_array_item(arr, 2);
        assert_eq!(lx_json_get_number(item), 3.0);
        
        lx_json_free(item);
        lx_json_free(arr);
    }
}

/// Test: Create float array through C ABI
#[test]
fn test_ffi_create_float_array() {
    unsafe {
        let values = vec![1.5f32, 2.5, 3.5];
        let arr = lx_json_create_float_array(values.as_ptr(), values.len());
        assert!(!arr.is_null(), "Array creation should succeed");
        assert_eq!(lx_json_get_array_size(arr), 3);
        
        let item = lx_json_get_array_item(arr, 1);
        assert_eq!(lx_json_get_number(item), 2.5);
        
        lx_json_free(item);
        lx_json_free(arr);
    }
}

/// Test: Create double array through C ABI
#[test]
fn test_ffi_create_double_array() {
    unsafe {
        let values = vec![1.1f64, 2.2, 3.3];
        let arr = lx_json_create_double_array(values.as_ptr(), values.len());
        assert!(!arr.is_null(), "Array creation should succeed");
        assert_eq!(lx_json_get_array_size(arr), 3);
        
        let item = lx_json_get_array_item(arr, 2);
        assert_eq!(lx_json_get_number(item), 3.3);
        
        lx_json_free(item);
        lx_json_free(arr);
    }
}

/// Test: Create string array through C ABI
#[test]
fn test_ffi_create_string_array() {
    unsafe {
        let s1 = CString::new("hello").unwrap();
        let s2 = CString::new("world").unwrap();
        let s3 = CString::new("rust").unwrap();
        let values = vec![s1.as_ptr(), s2.as_ptr(), s3.as_ptr()];
        
        let arr = lx_json_create_string_array(values.as_ptr(), values.len());
        assert!(!arr.is_null(), "Array creation should succeed");
        assert_eq!(lx_json_get_array_size(arr), 3);
        
        let item = lx_json_get_array_item(arr, 1);
        let str_val = lx_json_get_string_value(item);
        let result = CStr::from_ptr(str_val).to_str().unwrap();
        assert_eq!(result, "world");
        
        lx_json_free_string(str_val);
        lx_json_free(item);
        lx_json_free(arr);
    }
}

/// Test: NULL pointer handling for new functions
#[test]
fn test_ffi_null_pointer_handling() {
    unsafe {
        // Test sort_object with NULL
        assert_eq!(lx_json_sort_object(ptr::null_mut(), 1), 0);
        
        // Test create_int_array with NULL
        assert!(lx_json_create_int_array(ptr::null(), 5).is_null());
        
        // Test create_float_array with NULL
        assert!(lx_json_create_float_array(ptr::null(), 5).is_null());
        
        // Test create_double_array with NULL
        assert!(lx_json_create_double_array(ptr::null(), 5).is_null());
        
        // Test create_string_array with NULL
        assert!(lx_json_create_string_array(ptr::null(), 5).is_null());
    }
}