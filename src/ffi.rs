//! C ABI bindings for LX-json
//!
//! This module provides C-compatible FFI interfaces that allow any language
//! that can call C functions to use LX-json functionality.

use std::ffi::{CString, CStr};
use std::ptr;
use libc::{c_char, size_t, c_int, c_double};
use crate::error::JsonError;
use crate::parser::ParseOptions;
use crate::types::JsonNode;

/// Opaque pointer type for JsonNode
#[repr(C)]
#[derive(Debug)]
pub struct LXJsonNode {
    _private: [u8; 0],
}

/// Opaque pointer type for ParseOptions
#[repr(C)]
#[derive(Debug)]
pub struct LXParseOptions {
    _private: [u8; 0],
}

// Thread-local storage for the last error.
// Each thread has its own error storage, making this thread-safe.
// The error is set by FFI functions when they encounter an error.
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<JsonError>> = const { std::cell::RefCell::new(None) };
}

/// Set the last error
fn set_last_error(err: JsonError) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(err));
}

/// Clear the last error
fn clear_last_error() {
    LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

/// Get the last error message as a C string
#[no_mangle]
pub extern "C" fn lx_json_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        if let Some(ref err) = *e.borrow() {
            let msg = format!("{}", err);
            match CString::new(msg) {
                Ok(s) => s.into_raw(),
                Err(_) => ptr::null(),
            }
        } else {
            ptr::null()
        }
    })
}

/// Free a string allocated by the library
///
/// # Safety
///
/// The `s` pointer must be either NULL or a valid pointer to a string
/// that was allocated by this library (e.g., from `lx_json_print()`).
/// Passing any other pointer results in undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn lx_json_free_string(s: *const i8) {
    if !s.is_null() {
        // Safety: s is guaranteed to be non-null at this point
        unsafe {
            let _ = CString::from_raw(s as *mut i8);
        }
    }
}

/// Free a JsonNode
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library (e.g., from `lx_json_parse()`).
/// Passing any other pointer results in undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn lx_json_free(node: *mut LXJsonNode) {
    if node.is_null() {
        return;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let _ = Box::<JsonNode>::from_raw(node as *mut JsonNode);
    }
}

/// Parse a JSON string
///
/// # Safety
///
/// The `json` pointer must be either NULL or a valid pointer to a
/// null-terminated C string. If NULL is passed, the function returns NULL
/// and sets an error.
///
/// The returned pointer must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_parse(json: *const c_char) -> *mut LXJsonNode {
    clear_last_error();
    if json.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return ptr::null_mut();
    }
    
    // Safety: json is guaranteed to be non-null at this point
    unsafe {
        let c_str = match CStr::from_ptr(json).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8".to_string(),
                });
                return ptr::null_mut();
            }
        };
        
        match crate::parser::parse(c_str) {
            Ok(node) => Box::into_raw(Box::new(node)) as *mut LXJsonNode,
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }
}

/// Parse a JSON string with length
///
/// # Safety
///
/// The `json` pointer must be either NULL or a valid pointer to a byte array
/// of at least `len` bytes. If NULL is passed, the function returns NULL
/// and sets an error.
///
/// The returned pointer must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_parse_with_length(json: *const c_char, len: size_t) -> *mut LXJsonNode {
    clear_last_error();
    if json.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return ptr::null_mut();
    }
    
    // Safety: json is guaranteed to be non-null at this point
    unsafe {
        let slice = std::slice::from_raw_parts(json as *const u8, len);
        match std::str::from_utf8(slice) {
            Ok(s) => {
                match crate::parser::parse_with_length(s, len) {
                    Ok(node) => Box::into_raw(Box::new(node)) as *mut LXJsonNode,
                    Err(e) => {
                        set_last_error(e);
                        ptr::null_mut()
                    }
                }
            },
            Err(_) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8".to_string(),
                });
                ptr::null_mut()
            }
        }
    }
}

/// Create default parse options
///
/// # Returns
///
/// A pointer to a new ParseOptions object. The returned pointer must be
/// freed with `lx_json_parse_options_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_parse_options_new() -> *mut LXParseOptions {
    let opts = Box::new(ParseOptions::new());
    Box::into_raw(opts) as *mut LXParseOptions
}

/// Free parse options
///
/// # Safety
///
/// The `opts` pointer must be either NULL or a valid pointer to a ParseOptions
/// object that was created by `lx_json_parse_options_new()`.
/// Passing any other pointer results in undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn lx_json_parse_options_free(opts: *mut LXParseOptions) {
    if opts.is_null() {
        return;
    }
    // Safety: opts is guaranteed to be non-null at this point
    unsafe {
        let _ = Box::<ParseOptions>::from_raw(opts as *mut ParseOptions);
    }
}

/// Set nesting limit in parse options
///
/// # Safety
///
/// The `opts` pointer must be either NULL or a valid pointer to a ParseOptions
/// object. If NULL is passed, this function does nothing.
#[no_mangle]
pub unsafe extern "C" fn lx_json_parse_options_set_nesting_limit(opts: *mut LXParseOptions, limit: size_t) {
    if !opts.is_null() {
        // Safety: opts is guaranteed to be non-null at this point
        unsafe {
            let real_opts = &mut *(opts as *mut ParseOptions);
            real_opts.nesting_limit = limit;
        }
    }
}

/// Set require null terminated flag
///
/// # Safety
///
/// The `opts` pointer must be either NULL or a valid pointer to a ParseOptions
/// object. If NULL is passed, this function does nothing.
#[no_mangle]
pub unsafe extern "C" fn lx_json_parse_options_set_require_null_terminated(opts: *mut LXParseOptions, value: c_int) {
    if !opts.is_null() {
        // Safety: opts is guaranteed to be non-null at this point
        unsafe {
            let real_opts = &mut *(opts as *mut ParseOptions);
            real_opts.require_null_terminated = value != 0_i32;
        }
    }
}

/// Parse with options
///
/// # Safety
///
/// The `json` pointer must be either NULL or a valid pointer to a
/// null-terminated C string. The `opts` pointer must be either NULL or a
/// valid pointer to a ParseOptions object. If either is NULL, the function
/// returns NULL and sets an error.
///
/// The returned pointer must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_parse_with_opts(json: *const c_char, opts: *mut LXParseOptions) -> *mut LXJsonNode {
    clear_last_error();
    if json.is_null() || opts.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return ptr::null_mut();
    }
    
    // Safety: Both json and opts are guaranteed to be non-null at this point
    unsafe {
        let c_str = match CStr::from_ptr(json).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8".to_string(),
                });
                return ptr::null_mut();
            }
        };
        
        let real_opts = &*(opts as *const ParseOptions);
        let options = real_opts.clone();
        match crate::parser::parse_with_opts(c_str, options) {
            Ok(node) => Box::into_raw(Box::new(node)) as *mut LXJsonNode,
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }
}

/// Print JsonNode to a formatted string
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned string must be freed with `lx_json_free_string()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_print(node: *mut LXJsonNode) -> *mut c_char {
    clear_last_error();
    if node.is_null() {
        return ptr::null_mut();
    }
    
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *mut JsonNode);
        let result = crate::serializer::print(json_node);
        match CString::new(result) {
            Ok(s) => s.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Print JsonNode to an unformatted string
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned string must be freed with `lx_json_free_string()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_print_unformatted(node: *mut LXJsonNode) -> *mut c_char {
    clear_last_error();
    if node.is_null() {
        return ptr::null_mut();
    }
    
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *mut JsonNode);
        let result = crate::serializer::print_unformatted(json_node);
        match CString::new(result) {
            Ok(s) => s.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Minify JsonNode
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned string must be freed with `lx_json_free_string()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_minify(node: *mut LXJsonNode) -> *mut c_char {
    clear_last_error();
    if node.is_null() {
        return ptr::null_mut();
    }
    
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *mut JsonNode);
        let json_str = crate::serializer::print(json_node);
        let result = crate::serializer::minify(&json_str);
        match result {
            Ok(minified) => match CString::new(minified) {
                Ok(s) => s.into_raw(),
                Err(_) => ptr::null_mut(),
            },
            Err(e) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: e,
                });
                ptr::null_mut()
            }
        }
    }
}

/// Print JsonNode to string with pre-allocated buffer capacity
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned string must be freed with `lx_json_free_string()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_print_buffered(
    node: *mut LXJsonNode,
    prebuffer: usize,
    fmt: c_int
) -> *mut c_char {
    clear_last_error();
    if node.is_null() {
        return ptr::null_mut();
    }
    
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *mut JsonNode);
        let formatted = fmt != 0;
        let result = crate::serializer::print_buffered(json_node, prebuffer, formatted);
        match CString::new(result) {
            Ok(s) => s.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Print JsonNode into a pre-allocated buffer
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
///
/// The `buffer` pointer can be NULL to query the required length, or must point
/// to a valid buffer of at least `buffer_len` bytes.
///
/// The `required_length` pointer can be NULL, or must point to a valid usize
/// where the required buffer length will be written.
#[no_mangle]
pub unsafe extern "C" fn lx_json_print_preallocated(
    node: *mut LXJsonNode,
    buffer: *mut c_char,
    buffer_len: usize,
    fmt: c_int,
    required_length: *mut usize
) -> c_int {
    clear_last_error();
    if node.is_null() {
        return 0;
    }
    
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *mut JsonNode);
        let formatted = fmt != 0;
        
        // First serialize to get the required length
        let mut temp_buffer = String::new();
        match crate::serializer::print_preallocated(json_node, &mut temp_buffer, formatted) {
            Ok(()) => {
                let required = temp_buffer.len();
                
                // Return required length
                if !required_length.is_null() {
                    *required_length = required;
                }
                
                // Copy to provided buffer if large enough
                if !buffer.is_null() && buffer_len > required {
                    let json_str = match CString::new(temp_buffer) {
                        Ok(s) => s,
                        Err(_) => return 0,
                    };
                    let bytes = json_str.as_bytes_with_nul();
                    ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len().min(buffer_len));
                    1
                } else {
                    0
                }
            },
            Err(e) => {
                set_last_error(e);
                0
            }
        }
    }
}

/// Check if node is null
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_null(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_null() { 1 } else { 0 }
    }
}

/// Check if node is bool
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_bool(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_bool() { 1 } else { 0 }
    }
}

/// Check if node is number
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_number(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_number() { 1 } else { 0 }
    }
}

/// Check if node is string
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_string(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_string() { 1 } else { 0 }
    }
}

/// Check if node is array
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_array(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_array() { 1 } else { 0 }
    }
}

/// Check if node is object
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_object(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_object() { 1 } else { 0 }
    }
}

/// Check if node is true
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub extern "C" fn lx_json_is_true(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_true() { 1 } else { 0 }
    }
}

/// Check if node is false
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_false(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_false() { 1 } else { 0 }
    }
}

/// Check if node is raw
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_is_raw(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        if json_node.is_raw() { 1 } else { 0 }
    }
}

/// Create a new null node
///
/// # Returns
///
/// A pointer to a new null JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_create_null() -> *mut LXJsonNode {
    Box::into_raw(Box::new(JsonNode::new_null())) as *mut LXJsonNode
}

/// Create a new bool node
///
/// # Returns
///
/// A pointer to a new bool JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_create_bool(value: c_int) -> *mut LXJsonNode {
    Box::into_raw(Box::new(JsonNode::new_bool(value != 0))) as *mut LXJsonNode
}

/// Create a new true node
///
/// # Returns
///
/// A pointer to a new true JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_create_true() -> *mut LXJsonNode {
    Box::into_raw(Box::new(JsonNode::new_true())) as *mut LXJsonNode
}

/// Create a new false node
///
/// # Returns
///
/// A pointer to a new false JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_create_false() -> *mut LXJsonNode {
    Box::into_raw(Box::new(JsonNode::new_false())) as *mut LXJsonNode
}

/// Create a new number node
///
/// # Returns
///
/// A pointer to a new number JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_create_number(value: c_double) -> *mut LXJsonNode {
    Box::into_raw(Box::new(JsonNode::new_number(value))) as *mut LXJsonNode
}

/// Create a new string node
///
/// # Safety
///
/// The `value` pointer must be either NULL or a valid pointer to a
/// null-terminated C string. If NULL is passed, the function returns NULL.
///
/// # Returns
///
/// A pointer to a new string JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_create_string(value: *const c_char) -> *mut LXJsonNode {
    if value.is_null() {
        return ptr::null_mut();
    }
    // Safety: value is guaranteed to be non-null at this point
    unsafe {
        let c_str = match CStr::from_ptr(value).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        Box::into_raw(Box::new(JsonNode::new_string(c_str.to_string()))) as *mut LXJsonNode
    }
}

/// Create a new array node
///
/// # Returns
///
/// A pointer to a new empty array JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_create_array() -> *mut LXJsonNode {
    Box::into_raw(Box::new(JsonNode::new_array())) as *mut LXJsonNode
}

/// Create a new object node
///
/// # Returns
///
/// A pointer to a new empty object JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub extern "C" fn lx_json_create_object() -> *mut LXJsonNode {
    Box::into_raw(Box::new(JsonNode::new_object())) as *mut LXJsonNode
}

/// Get bool value
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_bool(node: *const LXJsonNode) -> c_int {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        match json_node.as_bool() {
            Some(b) => if b { 1 } else { 0 },
            None => 0,
        }
    }
}

/// Get number value
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_number(node: *const LXJsonNode) -> c_double {
    if node.is_null() {
        return 0.0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        json_node.as_number().unwrap_or(0.0)
    }
}

/// Get string value (must be freed with lx_json_free_string)
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned string must be freed with `lx_json_free_string()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_string_value(node: *const LXJsonNode) -> *mut c_char {
    if node.is_null() {
        return ptr::null_mut();
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        match json_node.as_string() {
            Some(s) => match CString::new(s) {
                Ok(c) => c.into_raw(),
                Err(_) => ptr::null_mut(),
            },
            None => ptr::null_mut(),
        }
    }
}

/// Get array size
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_array_size(node: *const LXJsonNode) -> size_t {
    if node.is_null() {
        return 0;
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        crate::query::get_array_size(json_node).unwrap_or_default()
    }
}

/// Get array item at index
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_array_item(node: *const LXJsonNode, index: size_t) -> *mut LXJsonNode {
    if node.is_null() {
        return ptr::null_mut();
    }
    // Safety: node is guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        match crate::query::get_array_item(json_node, index) {
            Ok(item) => Box::into_raw(Box::new(item.clone())) as *mut LXJsonNode,
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Add item to array
///
/// # Safety
///
/// The `array` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `item` pointer must be either NULL or
/// a valid pointer to a JsonNode. If either is NULL, the function returns 0.
///
/// On success, ownership of `item` is transferred to the array and the caller
/// should not free it. On failure, the caller is responsible for freeing `item`.
#[no_mangle]
pub unsafe extern "C" fn lx_json_add_item_to_array(array: *mut LXJsonNode, item: *mut LXJsonNode) -> c_int {
    clear_last_error();
    if array.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null array".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    if item.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null item".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    
    // Safety: Both array and item are guaranteed to be non-null at this point
    unsafe {
        let json_array = &mut *(array as *mut JsonNode);
        let json_item = Box::<JsonNode>::from_raw(item as *mut JsonNode);
        let item_value = (*json_item).clone(); // Clone the value
        match json_array.add_item_to_array(item_value) {
            Ok(()) => 1,
            Err(e) => {
                // Reclaim ownership to prevent memory leak
                let _ = Box::into_raw(json_item);
                set_last_error(e);
                0
            }
        }
    }
}

/// Delete item from array
///
/// # Safety
///
/// The `array` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_delete_item_from_array(array: *mut LXJsonNode, index: size_t) -> c_int {
    clear_last_error();
    if array.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null array".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    
    // Safety: array is guaranteed to be non-null at this point
    unsafe {
        let json_array = &mut *(array as *mut JsonNode);
        match json_array.delete_item_from_array(index) {
            Ok(()) => 1,
            Err(e) => {
                set_last_error(e);
                0
            }
        }
    }
}

/// Detach item from array (caller must free the detached item)
///
/// # Safety
///
/// The `array` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_detach_item_from_array(array: *mut LXJsonNode, index: size_t) -> *mut LXJsonNode {
    clear_last_error();
    if array.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null array".to_string(),
            found: "null".to_string(),
        });
        return ptr::null_mut();
    }
    
    // Safety: array is guaranteed to be non-null at this point
    unsafe {
        let json_array = &mut *(array as *mut JsonNode);
        match json_array.detach_item_from_array(index) {
            Ok(item) => Box::into_raw(Box::new(item)) as *mut LXJsonNode,
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }
}

/// Replace item in array
///
/// # Safety
///
/// The `array` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `new_item` pointer must be either NULL
/// or a valid pointer to a JsonNode. If either is NULL, the function returns 0.
///
/// On success, ownership of `new_item` is transferred to the array and the caller
/// should not free it. On failure, the caller is responsible for freeing `new_item`.
#[no_mangle]
pub extern "C" fn lx_json_replace_item_in_array(array: *mut LXJsonNode, index: size_t, new_item: *mut LXJsonNode) -> c_int {
    clear_last_error();
    if array.is_null() || new_item.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    
    // Safety: Both array and new_item are guaranteed to be non-null at this point
    unsafe {
        let json_array = &mut *(array as *mut JsonNode);
        let json_item = Box::<JsonNode>::from_raw(new_item as *mut JsonNode);
        let item_value = (*json_item).clone(); // Clone the value
        match json_array.replace_item_in_array(index, item_value) {
            Ok(()) => 1,
            Err(e) => {
                // Reclaim ownership to prevent memory leak
                let _ = Box::into_raw(json_item);
                set_last_error(e);
                0
            }
        }
    }
}

/// Insert item in array at index
///
/// # Safety
///
/// The `array` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `new_item` pointer must be either NULL
/// or a valid pointer to a JsonNode. If either is NULL, the function returns 0.
///
/// On success, ownership of `new_item` is transferred to the array and the caller
/// should not free it. On failure, the caller is responsible for freeing `new_item`.
#[no_mangle]
pub extern "C" fn lx_json_insert_item_in_array(array: *mut LXJsonNode, index: size_t, new_item: *mut LXJsonNode) -> c_int {
    clear_last_error();
    if array.is_null() || new_item.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    
    // Safety: Both array and new_item are guaranteed to be non-null at this point
    unsafe {
        let json_array = &mut *(array as *mut JsonNode);
        let json_item = Box::<JsonNode>::from_raw(new_item as *mut JsonNode);
        match json_array.insert_item_in_array(index, *json_item) {
            Ok(()) => 1,
            Err(e) => {
                set_last_error(e);
                0
            }
        }
    }
}

/// Get object item by key (case-sensitive)
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `key` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. If either is NULL, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_object_item(node: *const LXJsonNode, key: *const c_char) -> *mut LXJsonNode {
    if node.is_null() || key.is_null() {
        return ptr::null_mut();
    }
    // Safety: Both node and key are guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        let c_key = match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        match crate::query::get_object_item_case_sensitive(json_node, c_key) {
            Ok(item) => Box::into_raw(Box::new(item.clone())) as *mut LXJsonNode,
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Get object item by key (case-insensitive)
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `key` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. If either is NULL, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_object_item_case_insensitive(node: *const LXJsonNode, key: *const c_char) -> *mut LXJsonNode {
    if node.is_null() || key.is_null() {
        return ptr::null_mut();
    }
    // Safety: Both node and key are guaranteed to be non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        let c_key = match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        match crate::query::get_object_item(json_node, c_key) {
            Ok(item) => Box::into_raw(Box::new(item.clone())) as *mut LXJsonNode,
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Check if object has key
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `key` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. If either is NULL, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_has_object_item(node: *const LXJsonNode, key: *const c_char) -> c_int {
    if node.is_null() || key.is_null() {
        return 0;
    }
    // Safety: Both node and key are guaranteed to be valid and non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        let c_key = match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };
        if crate::query::has_object_item(json_node, c_key) { 1 } else { 0 }
    }
}

/// Add item to object
///
/// # Safety
///
/// The `object` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `key` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. The `item` pointer must be
/// either NULL or a valid pointer to a JsonNode. If any is NULL, the function returns 0.
///
/// On success, ownership of `item` is transferred to the object and the caller
/// should not free it. On failure, the caller is responsible for freeing `item`.
#[no_mangle]
pub unsafe extern "C" fn lx_json_add_item_to_object(object: *mut LXJsonNode, key: *const c_char, item: *mut LXJsonNode) -> c_int {
    clear_last_error();
    if object.is_null() || key.is_null() || item.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    
    // Safety: All pointers are guaranteed to be valid and non-null at this point
    unsafe {
        let json_object = &mut *(object as *mut JsonNode);
        let json_item = Box::<JsonNode>::from_raw(item as *mut JsonNode);
        let c_key = match CStr::from_ptr(key).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                // Reclaim ownership to prevent memory leak
                let _ = Box::into_raw(json_item);
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8".to_string(),
                });
                return 0;
            }
        };
        let item_value = (*json_item).clone(); // Clone the value
        match json_object.add_item_to_object(c_key, item_value) {
            Ok(()) => 1,
            Err(e) => {
                // Reclaim ownership to prevent memory leak
                let _ = Box::into_raw(json_item);
                set_last_error(e);
                0
            }
        }
    }
}

/// Delete item from object
///
/// # Safety
///
/// The `object` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `key` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. If either is NULL, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_delete_item_from_object(object: *mut LXJsonNode, key: *const c_char) -> c_int {
    clear_last_error();
    if object.is_null() || key.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    
    // Safety: Both object and key are guaranteed to be valid and non-null at this point
    unsafe {
        let json_object = &mut *(object as *mut JsonNode);
        let c_key = match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8".to_string(),
                });
                return 0;
            }
        };
        match json_object.delete_item_from_object(c_key) {
            Ok(()) => 1,
            Err(e) => {
                set_last_error(e);
                0
            }
        }
    }
}

/// Detach item from object (caller must free the detached item)
///
/// # Safety
///
/// The `object` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `key` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. If either is NULL, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_detach_item_from_object(object: *mut LXJsonNode, key: *const c_char) -> *mut LXJsonNode {
    clear_last_error();
    if object.is_null() || key.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null".to_string(),
        });
        return ptr::null_mut();
    }
    
    // Safety: Both object and key are guaranteed to be valid and non-null at this point
    unsafe {
        let json_object = &mut *(object as *mut JsonNode);
        let c_key = match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8".to_string(),
                });
                return ptr::null_mut();
            }
        };
        match json_object.detach_item_from_object(c_key) {
            Ok(item) => Box::into_raw(Box::new(item)) as *mut LXJsonNode,
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }
}

/// Replace item in object
///
/// # Safety
///
/// The `object` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `key` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. The `new_item` pointer must be
/// either NULL or a valid pointer to a JsonNode. If any is NULL, the function returns 0.
///
/// On success, ownership of `new_item` is transferred to the object and the caller
/// should not free it. On failure, the caller is responsible for freeing `new_item`.
#[no_mangle]
pub unsafe extern "C" fn lx_json_replace_item_in_object(object: *mut LXJsonNode, key: *const c_char, new_item: *mut LXJsonNode) -> c_int {
    clear_last_error();
    if object.is_null() || key.is_null() || new_item.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null".to_string(),
        });
        return 0;
    }
    
    // Safety: All pointers are guaranteed to be valid and non-null at this point
    unsafe {
        let json_object = &mut *(object as *mut JsonNode);
        let json_item = Box::<JsonNode>::from_raw(new_item as *mut JsonNode);
        let c_key = match CStr::from_ptr(key).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                // Reclaim ownership to prevent memory leak
                let _ = Box::into_raw(json_item);
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8".to_string(),
                });
                return 0;
            }
        };
        let item_value = (*json_item).clone(); // Clone the value
        match json_object.replace_item_in_object(&c_key, item_value) {
            Ok(()) => 1,
            Err(e) => {
                // Reclaim ownership to prevent memory leak
                let _ = Box::into_raw(json_item);
                set_last_error(e);
                0
            }
        }
    }
}

/// Get value using JSON Pointer
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `pointer` pointer must be either NULL or
/// a valid pointer to a null-terminated C string. If either is NULL, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_pointer(node: *const LXJsonNode, pointer: *const c_char) -> *mut LXJsonNode {
    if node.is_null() || pointer.is_null() {
        return ptr::null_mut();
    }
    // Safety: Both node and pointer are guaranteed to be valid and non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        let c_pointer = match CStr::from_ptr(pointer).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        match crate::query::get_pointer(json_node, c_pointer) {
            Ok(item) => Box::into_raw(Box::new(item.clone())) as *mut LXJsonNode,
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Duplicate a node
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_duplicate(node: *const LXJsonNode) -> *mut LXJsonNode {
    if node.is_null() {
        return ptr::null_mut();
    }
    // Safety: node is guaranteed to be valid and non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        Box::into_raw(Box::new(crate::utils::duplicate(json_node, true))) as *mut LXJsonNode
    }
}

/// Compare two nodes
///
/// # Safety
///
/// The `a` and `b` pointers must be either NULL or valid pointers to JsonNodes
/// that were created by this library. If either is NULL, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_compare(a: *const LXJsonNode, b: *const LXJsonNode) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }
    // Safety: Both a and b are guaranteed to be valid and non-null at this point
    unsafe {
        let node_a = &*(a as *const JsonNode);
        let node_b = &*(b as *const JsonNode);
        if crate::utils::compare(node_a, node_b, true) { 1 } else { 0 }
    }
}

/// Add a single patch operation to a patch array
///
/// # Safety
///
/// The `patches` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. The `op` and `path` pointers must be either NULL or
/// valid pointers to null-terminated C strings. The `value` pointer can be NULL.
/// If `patches`, `op`, or `path` is NULL, the function returns 0.
#[no_mangle]
pub unsafe extern "C" fn lx_json_add_patch_to_array(
    patches: *mut LXJsonNode,
    op: *const c_char,
    path: *const c_char,
    value: *const LXJsonNode
) -> c_int {
    clear_last_error();
    if patches.is_null() || op.is_null() || path.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }
    
    // Safety: patches, op, and path are guaranteed to be valid and non-null at this point
    unsafe {
        let json_patches = &mut *(patches as *mut JsonNode);
        let op_str = match CStr::from_ptr(op).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8 in operation string".to_string(),
                });
                return 0;
            }
        };
        let path_str = match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => {
                set_last_error(JsonError::InvalidString {
                    position: 0,
                    reason: "Invalid UTF-8 in path string".to_string(),
                });
                return 0;
            }
        };
        
        let value_ref = if value.is_null() { None } else { Some(&*(value as *const JsonNode)) };
        
        match crate::patch::add_patch_to_array(json_patches, op_str, path_str, value_ref) {
            Ok(()) => 1,
            Err(e) => {
                set_last_error(e);
                0
            }
        }
    }
}

/// Generate JSON Patch patches
///
/// # Safety
///
/// The `source` and `target` pointers must be either NULL or valid pointers to JsonNodes
/// that were created by this library. If either is NULL, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_generate_patches(source: *const LXJsonNode, target: *const LXJsonNode) -> *mut LXJsonNode {
    if source.is_null() || target.is_null() {
        return ptr::null_mut();
    }
    // Safety: Both source and target are guaranteed to be valid and non-null at this point
    unsafe {
        let json_source = &*(source as *const JsonNode);
        let json_target = &*(target as *const JsonNode);
        match crate::patch::generate_patches(json_source, json_target, true) {
            Ok(patches) => Box::into_raw(Box::new(patches)) as *mut LXJsonNode,
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Apply JSON Patch patches
///
/// # Safety
///
/// The `node` and `patches` pointers must be either NULL or valid pointers to JsonNodes
/// that were created by this library. If either is NULL, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_apply_patches(node: *mut LXJsonNode, patches: *const LXJsonNode) -> *mut LXJsonNode {
    if node.is_null() || patches.is_null() {
        return ptr::null_mut();
    }
    // Safety: Both node and patches are guaranteed to be valid and non-null at this point
    unsafe {
        let json_node = &mut *(node as *mut JsonNode);
        let json_patches = &*(patches as *const JsonNode);
        match crate::patch::apply_patches(json_node, json_patches, true) {
            Ok(()) => Box::into_raw(Box::new(json_node.clone())) as *mut LXJsonNode,
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Apply JSON Merge Patch
///
/// # Safety
///
/// The `node` and `patch` pointers must be either NULL or valid pointers to JsonNodes
/// that were created by this library. If either is NULL, the function returns NULL.
///
/// The returned node must be freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_merge_patch(node: *mut LXJsonNode, patch: *const LXJsonNode) -> *mut LXJsonNode {
    if node.is_null() || patch.is_null() {
        return ptr::null_mut();
    }
    // Safety: Both node and patch are guaranteed to be valid and non-null at this point
    unsafe {
        let json_node = &mut *(node as *mut JsonNode);
        let json_patch = &*(patch as *const JsonNode);
        match crate::merge::merge_patch(json_node, json_patch, true) {
            Ok(()) => Box::into_raw(Box::new(json_node.clone())) as *mut LXJsonNode,
            Err(_) => ptr::null_mut(),
        }
    }
}
/// Get number value from a JSON node
///
/// # Safety
///
/// The `node` pointer must be a valid pointer to a JsonNode created by this library,
/// or NULL. If NULL is passed, 0.0 is returned.
///
/// The returned value is a simple c_double and does not need to be freed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_number_value(node: *const LXJsonNode) -> c_double {
    if node.is_null() {
        return 0.0;
    }
    // Safety: The pointer is guaranteed to be valid and non-null at this point
    unsafe {
        let json_node = &*(node as *const JsonNode);
        crate::query::get_number_value(json_node).unwrap_or(0.0)
    }
}

/// Get mutable reference using JSON Pointer
///
/// # Safety
///
/// The `node` pointer must be a valid pointer to a JsonNode created by this library,
/// or NULL. The `pointer` must be a valid C string or NULL.
///
/// # Returns
///
/// - If `pointer` is "/", returns the original `node` pointer (for the root)
/// - For any other pointer, returns NULL (because we cannot safely return internal pointers)
/// - If either `node` or `pointer` is NULL, returns NULL
///
/// This function is primarily useful for validating that a pointer is valid.
/// For actual mutation, use the specific mutation functions like `lx_json_add_item_to_object`.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_pointer_mut(node: *mut LXJsonNode, pointer: *const c_char) -> *mut LXJsonNode {
    if node.is_null() || pointer.is_null() {
        return ptr::null_mut();
    }
    // Safety: Both pointers are guaranteed to be valid and non-null at this point
    unsafe {
        let c_str = match CStr::from_ptr(pointer).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        
        // Only return the node pointer for root access ("/")
        // For any other pointer, we cannot safely return an internal pointer
        if c_str == "/" {
            node
        } else {
            ptr::null_mut()
        }
    }
}
/// Sort the keys of an object
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns 0.
///
/// # Arguments
///
/// * `node` - Object node to sort
/// * `case_sensitive` - If non-zero, sort with case sensitivity; otherwise, sort case-insensitively
///
/// # Returns
///
/// 1 on success, 0 on error. Check `lx_json_get_last_error()` for details.
///
/// # Errors
///
/// Returns 0 if the node is not an object.
#[no_mangle]
pub unsafe extern "C" fn lx_json_sort_object(node: *mut LXJsonNode, case_sensitive: c_int) -> c_int {
    clear_last_error();
    if node.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null".to_string(),
        });
        return 0_i32;
    }
    // Safety: node is guaranteed to be non-null at this point
    let json_node = &mut *(node as *mut JsonNode);
    match json_node.sort_object(case_sensitive != 0_i32) {
        Ok(_) => 1_i32,
        Err(e) => {
            set_last_error(e);
            0_i32
        }
    }
}

/// Create an array from i64 values
///
/// # Safety
///
/// The `values` pointer must be either NULL or a valid pointer to an array of i64 values.
/// If NULL is passed, the function returns NULL.
///
/// # Arguments
///
/// * `values` - Pointer to i64 array
/// * `len` - Number of elements in the array
///
/// # Returns
///
/// A pointer to a new array JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_create_int_array(values: *const i64, len: size_t) -> *mut LXJsonNode {
    if values.is_null() {
        return ptr::null_mut();
    }
    // Safety: values is guaranteed to be non-null at this point
    let slice = std::slice::from_raw_parts(values, len);
    Box::into_raw(Box::new(JsonNode::create_int_array(slice))) as *mut LXJsonNode
}

/// Create an array from f32 values
///
/// # Safety
///
/// The `values` pointer must be either NULL or a valid pointer to an array of f32 values.
/// If NULL is passed, the function returns NULL.
///
/// # Arguments
///
/// * `values` - Pointer to f32 array
/// * `len` - Number of elements in the array
///
/// # Returns
///
/// A pointer to a new array JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_create_float_array(values: *const f32, len: size_t) -> *mut LXJsonNode {
    if values.is_null() {
        return ptr::null_mut();
    }
    // Safety: values is guaranteed to be non-null at this point
    let slice = std::slice::from_raw_parts(values, len);
    Box::into_raw(Box::new(JsonNode::create_float_array(slice))) as *mut LXJsonNode
}

/// Create an array from f64 values
///
/// # Safety
///
/// The `values` pointer must be either NULL or a valid pointer to an array of f64 values.
/// If NULL is passed, the function returns NULL.
///
/// # Arguments
///
/// * `values` - Pointer to f64 array
/// * `len` - Number of elements in the array
///
/// # Returns
///
/// A pointer to a new array JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_create_double_array(values: *const f64, len: size_t) -> *mut LXJsonNode {
    if values.is_null() {
        return ptr::null_mut();
    }
    // Safety: values is guaranteed to be non-null at this point
    let slice = std::slice::from_raw_parts(values, len);
    Box::into_raw(Box::new(JsonNode::create_double_array(slice))) as *mut LXJsonNode
}

/// Create an array from string values
///
/// # Safety
///
/// The `values` pointer must be either NULL or a valid pointer to an array of C string pointers.
/// If NULL is passed, the function returns NULL.
///
/// # Arguments
///
/// * `values` - Pointer to array of C string pointers
/// * `len` - Number of elements in the array
///
/// # Returns
///
/// A pointer to a new array JsonNode. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
#[no_mangle]
pub unsafe extern "C" fn lx_json_create_string_array(values: *const *const c_char, len: size_t) -> *mut LXJsonNode {
    if values.is_null() {
        return ptr::null_mut();
    }
    // Safety: values is guaranteed to be non-null at this point
    let slice = std::slice::from_raw_parts(values, len);
    let mut rust_strings = Vec::with_capacity(len);
    for &ptr in slice {
        if ptr.is_null() {
            return ptr::null_mut();
        }
        let c_str = match CStr::from_ptr(ptr).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        rust_strings.push(c_str);
    }
    Box::into_raw(Box::new(JsonNode::create_string_array(&rust_strings))) as *mut LXJsonNode
}

/// Get the keys of an object as an array
///
/// # Safety
///
/// The `node` pointer must be either NULL or a valid pointer to a JsonNode
/// that was created by this library. If NULL is passed, the function returns NULL.
///
/// # Returns
///
/// A pointer to a new array JsonNode containing the keys. The returned pointer must be
/// freed with `lx_json_free()` when no longer needed.
///
/// # Errors
///
/// Returns NULL if the node is not an object.
#[no_mangle]
pub unsafe extern "C" fn lx_json_get_object_keys(node: *const LXJsonNode) -> *mut LXJsonNode {
    if node.is_null() {
        return ptr::null_mut();
    }
    // Safety: node is guaranteed to be non-null at this point
    let json_node = &*(node as *const JsonNode);
    match json_node.get_object_keys() {
        Ok(keys) => Box::into_raw(Box::new(keys)) as *mut LXJsonNode,
        Err(_) => ptr::null_mut(),
    }
}
/// Add a string to the object with the given key
///
/// # Safety
///
/// - `object` must be a valid pointer to an `LXJsonNode` or NULL
/// - `key` must be a valid pointer to a null-terminated C string or NULL
/// - `value` must be a valid pointer to a null-terminated C string or NULL
/// - The caller is responsible for managing the memory of `object`
/// - The returned string (if any) must be freed with `lx_json_free_string`
#[no_mangle]
pub unsafe extern "C" fn lx_json_add_string_to_object(
    object: *mut LXJsonNode,
    key: *const c_char,
    value: *const c_char,
) -> c_int {
    if object.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }
    if key.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }
    if value.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }

    let object = &mut *(object as *mut JsonNode);
    let key = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(JsonError::InvalidString {
                position: 0,
                reason: "Invalid UTF-8".to_string(),
            });
            return 0;
        }
    };
    let value = match CStr::from_ptr(value).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(JsonError::InvalidString {
                position: 0,
                reason: "Invalid UTF-8".to_string(),
            });
            return 0;
        }
    };

    match object.add_string_to_object(key, value) {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(e);
            0
        }
    }
}

/// Add a number to the object with the given key
///
/// # Safety
///
/// - `object` must be a valid pointer to an `LXJsonNode` or NULL
/// - `key` must be a valid pointer to a null-terminated C string or NULL
/// - The caller is responsible for managing the memory of `object`
#[no_mangle]
pub unsafe extern "C" fn lx_json_add_number_to_object(
    object: *mut LXJsonNode,
    key: *const c_char,
    value: c_double,
) -> c_int {
    if object.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }
    if key.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }

    let object = &mut *(object as *mut JsonNode);
    let key = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(JsonError::InvalidString {
                position: 0,
                reason: "Invalid UTF-8".to_string(),
            });
            return 0;
        }
    };

    match object.add_number_to_object(key, value) {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(e);
            0
        }
    }
}

/// Add a boolean to the object with the given key
///
/// # Safety
///
/// - `object` must be a valid pointer to an `LXJsonNode` or NULL
/// - `key` must be a valid pointer to a null-terminated C string or NULL
/// - The caller is responsible for managing the memory of `object`
#[no_mangle]
pub unsafe extern "C" fn lx_json_add_bool_to_object(
    object: *mut LXJsonNode,
    key: *const c_char,
    value: c_int,
) -> c_int {
    if object.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }
    if key.is_null() {
        set_last_error(JsonError::InvalidType {
            expected: "non-null pointer".to_string(),
            found: "null pointer".to_string(),
        });
        return 0;
    }

    let object = &mut *(object as *mut JsonNode);
    let key = match CStr::from_ptr(key).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(JsonError::InvalidString {
                position: 0,
                reason: "Invalid UTF-8".to_string(),
            });
            return 0;
        }
    };
    let value = value != 0_i32;

    match object.add_bool_to_object(key, value) {
        Ok(_) => 1,
        Err(e) => {
            set_last_error(e);
            0
        }
    }
}