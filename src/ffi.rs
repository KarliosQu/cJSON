//! FFI layer providing C-compatible cJSON API backed by LX-json internals.
//!
//! This module serves as the crate root when building the cdylib (.so),
//! re-declaring all original LX-json modules and exporting the full cJSON C API.

// ── Original LX-json modules (unchanged) ────────────────────────────────────
pub mod error;
pub mod merge;
pub mod parser;
pub mod patch;
pub mod query;
pub mod serializer;
pub mod types;
pub mod utils;

// Re-exports required by internal modules (they use `crate::JsonError` etc.)
pub use error::{JsonError, Result};
pub use merge::merge_patch;
pub use parser::{parse, parse_with_length, parse_with_opts, ParseOptions};
pub use patch::{add_patch_to_array, apply_patches, generate_patches};
pub use query::{
    get_array_item, get_array_size, get_number_value, get_object_item,
    get_object_item_case_sensitive, get_pointer, get_pointer_mut, get_string_value,
    has_object_item,
};
pub use serializer::{minify, print, print_buffered, print_preallocated, print_unformatted};
pub use types::JsonNode;
pub use utils::{compare, duplicate};

// ── FFI glue ─────────────────────────────────────────────────────────────────
use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_int, c_void};
use std::ptr;

// ---------------------------------------------------------------------------
// cJSON type flag constants (must match cJSON.h exactly)
// ---------------------------------------------------------------------------
const CJSON_INVALID: c_int = 0;
const CJSON_FALSE: c_int = 1 << 0;
const CJSON_TRUE: c_int = 1 << 1;
const CJSON_NULL: c_int = 1 << 2;
const CJSON_NUMBER: c_int = 1 << 3;
const CJSON_STRING: c_int = 1 << 4;
const CJSON_ARRAY: c_int = 1 << 5;
const CJSON_OBJECT: c_int = 1 << 6;
const CJSON_RAW: c_int = 1 << 7;
const CJSON_IS_REFERENCE: c_int = 256;
const CJSON_STRING_IS_CONST: c_int = 512;

// ---------------------------------------------------------------------------
// cJSON struct – binary-compatible with the C definition
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct cJSON {
    pub next: *mut cJSON,
    pub prev: *mut cJSON,
    pub child: *mut cJSON,
    pub type_: c_int,
    pub valuestring: *mut c_char,
    pub valueint: c_int,
    pub valuedouble: c_double,
    pub string: *mut c_char,
}

#[repr(C)]
pub struct cJSON_Hooks {
    pub malloc_fn: Option<unsafe extern "C" fn(sz: usize) -> *mut c_void>,
    pub free_fn: Option<unsafe extern "C" fn(ptr: *mut c_void)>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct internal_hooks {
    pub allocate: Option<unsafe extern "C" fn(usize) -> *mut c_void>,
    pub deallocate: Option<unsafe extern "C" fn(*mut c_void)>,
    pub reallocate: Option<unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void>,
}

#[repr(C)]
pub struct cjson_error {
    pub json: *const u8,
    pub position: usize,
}

#[repr(C)]
pub struct parse_buffer {
    pub content: *const u8,
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
    pub hooks: internal_hooks,
}

#[repr(C)]
pub struct printbuffer {
    pub buffer: *mut u8,
    pub length: usize,
    pub offset: usize,
    pub depth: usize,
    pub noalloc: c_int,
    pub format: c_int,
    pub hooks: internal_hooks,
}

const CJSON_NESTING_LIMIT: usize = 1000;

// ---------------------------------------------------------------------------
// libc helpers (declared here to avoid an external crate dependency)
// ---------------------------------------------------------------------------
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strtod(nptr: *const c_char, endptr: *mut *mut c_char) -> c_double;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// ---------------------------------------------------------------------------
// Custom allocator hooks (thread-local)
// ---------------------------------------------------------------------------
thread_local! {
    static HOOKS_MALLOC: RefCell<Option<unsafe extern "C" fn(usize) -> *mut c_void>> =
        RefCell::new(None);
    static HOOKS_FREE: RefCell<Option<unsafe extern "C" fn(*mut c_void)>> =
        RefCell::new(None);
    static ERROR_PTR: RefCell<*const c_char> = RefCell::new(ptr::null());
}

unsafe fn internal_malloc(size: usize) -> *mut c_void {
    HOOKS_MALLOC.with(|h| {
        if let Some(f) = *h.borrow() {
            f(size)
        } else {
            malloc(size)
        }
    })
}

unsafe fn internal_free(ptr: *mut c_void) {
    HOOKS_FREE.with(|h| {
        if let Some(f) = *h.borrow() {
            f(ptr)
        } else {
            free(ptr)
        }
    })
}

// ---------------------------------------------------------------------------
// Global hooks (matches C's static global_hooks)
// ---------------------------------------------------------------------------
unsafe extern "C" fn default_malloc_fn(size: usize) -> *mut c_void { malloc(size) }
unsafe extern "C" fn default_free_fn(ptr: *mut c_void) { free(ptr) }
unsafe extern "C" fn default_realloc_fn(ptr: *mut c_void, size: usize) -> *mut c_void { realloc(ptr, size) }

static mut GLOBAL_HOOKS: internal_hooks = internal_hooks {
    allocate: Some(default_malloc_fn),
    deallocate: Some(default_free_fn),
    reallocate: Some(default_realloc_fn),
};

static mut GLOBAL_ERROR: cjson_error = cjson_error {
    json: 0 as *const u8,
    position: 0,
};

unsafe fn hooks_allocate(hooks: &internal_hooks, size: usize) -> *mut c_void {
    if let Some(f) = hooks.allocate { f(size) } else { malloc(size) }
}

unsafe fn hooks_deallocate(hooks: &internal_hooks, ptr: *mut c_void) {
    if let Some(f) = hooks.deallocate { f(ptr) } else { free(ptr) }
}

unsafe fn hooks_reallocate(hooks: &internal_hooks, ptr: *mut c_void, size: usize) -> *mut c_void {
    if let Some(f) = hooks.reallocate { f(ptr, size) } else { realloc(ptr, size) }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Duplicate a C string using the current allocator.
unsafe fn cstr_dup(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    let len = strlen(s);
    let p = internal_malloc(len + 1) as *mut c_char;
    if !p.is_null() {
        memcpy(p as *mut c_void, s as *const c_void, len + 1);
    }
    p
}

/// Allocate a C string from a Rust `&str` using the current allocator.
unsafe fn cstr_from_rust(s: &str) -> *mut c_char {
    let bytes = s.as_bytes();
    let p = internal_malloc(bytes.len() + 1) as *mut c_char;
    if p.is_null() {
        return ptr::null_mut();
    }
    ptr::copy_nonoverlapping(bytes.as_ptr(), p as *mut u8, bytes.len());
    *p.add(bytes.len()) = 0;
    p
}

/// Allocate and zero-initialise a `cJSON` node.
unsafe fn new_item() -> *mut cJSON {
    let p = internal_malloc(std::mem::size_of::<cJSON>()) as *mut cJSON;
    if !p.is_null() {
        ptr::write_bytes(p, 0, 1);
    }
    p
}

/// Link `item` after `prev` in a sibling chain.
unsafe fn suffix_object(prev: *mut cJSON, item: *mut cJSON) {
    (*prev).next = item;
    (*item).prev = prev;
}

// ---------------------------------------------------------------------------
// JsonNode ↔ cJSON conversion
// ---------------------------------------------------------------------------

/// Convert a Rust `JsonNode` tree into a heap-allocated `cJSON` linked-list tree.
unsafe fn jsonnode_to_cjson(node: &types::JsonNode) -> *mut cJSON {
    match node {
        JsonNode::Null => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_NULL;
            item
        }
        JsonNode::Bool(true) => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_TRUE;
            item
        }
        JsonNode::Bool(false) => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_FALSE;
            item
        }
        JsonNode::Number(n) => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_NUMBER;
            (*item).valuedouble = *n;
            (*item).valueint = saturating_double_to_int(*n);
            item
        }
        JsonNode::String(s) => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_STRING;
            (*item).valuestring = cstr_from_rust(s);
            item
        }
        JsonNode::Raw(s) => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_RAW;
            (*item).valuestring = cstr_from_rust(s);
            item
        }
        JsonNode::Array(arr) => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_ARRAY;
            let mut prev_child: *mut cJSON = ptr::null_mut();
            for (i, child_node) in arr.iter().enumerate() {
                let child = jsonnode_to_cjson(child_node);
                if child.is_null() { continue; }
                if i == 0 {
                    (*item).child = child;
                } else {
                    suffix_object(prev_child, child);
                }
                prev_child = child;
            }
            // tail pointer: head->prev = last child
            if !(*item).child.is_null() && !prev_child.is_null() {
                (*(*item).child).prev = prev_child;
            }
            item
        }
        JsonNode::Object(pairs) => {
            let item = new_item();
            if item.is_null() { return ptr::null_mut(); }
            (*item).type_ = CJSON_OBJECT;
            let mut prev_child: *mut cJSON = ptr::null_mut();
            for (i, (key, val)) in pairs.iter().enumerate() {
                let child = jsonnode_to_cjson(val);
                if child.is_null() { continue; }
                (*child).string = cstr_from_rust(key);
                if i == 0 {
                    (*item).child = child;
                } else {
                    suffix_object(prev_child, child);
                }
                prev_child = child;
            }
            // tail pointer: head->prev = last child
            if !(*item).child.is_null() && !prev_child.is_null() {
                (*(*item).child).prev = prev_child;
            }
            item
        }
    }
}

/// Convert a `cJSON` linked-list tree back into a Rust `JsonNode`.
/// Returns `None` if a circular reference is detected.
unsafe fn cjson_to_jsonnode(item: *const cJSON) -> JsonNode {
    let mut visited = std::collections::HashSet::new();
    cjson_to_jsonnode_impl(item, &mut visited)
        .unwrap_or(JsonNode::Null)
}

/// Try converting, returning `None` on circular reference.
unsafe fn cjson_to_jsonnode_checked(item: *const cJSON) -> Option<JsonNode> {
    let mut visited = std::collections::HashSet::new();
    cjson_to_jsonnode_impl(item, &mut visited)
}

unsafe fn cjson_to_jsonnode_impl(
    item: *const cJSON,
    visited: &mut std::collections::HashSet<usize>,
) -> Option<JsonNode> {
    if item.is_null() {
        return Some(JsonNode::Null);
    }
    let addr = item as usize;
    if !visited.insert(addr) {
        // Circular reference detected
        return None;
    }
    let t = (*item).type_ & 0xFF;
    let result = match t {
        x if x == CJSON_FALSE => Some(JsonNode::Bool(false)),
        x if x == CJSON_TRUE => Some(JsonNode::Bool(true)),
        x if x == CJSON_NULL => Some(JsonNode::Null),
        x if x == CJSON_NUMBER => Some(JsonNode::Number((*item).valuedouble)),
        x if x == CJSON_STRING => {
            if (*item).valuestring.is_null() {
                Some(JsonNode::String(String::new()))
            } else {
                Some(JsonNode::String(
                    CStr::from_ptr((*item).valuestring)
                        .to_string_lossy()
                        .into_owned(),
                ))
            }
        }
        x if x == CJSON_RAW => {
            if (*item).valuestring.is_null() {
                Some(JsonNode::Raw(String::new()))
            } else {
                Some(JsonNode::Raw(
                    CStr::from_ptr((*item).valuestring)
                        .to_string_lossy()
                        .into_owned(),
                ))
            }
        }
        x if x == CJSON_ARRAY => {
            let mut v = Vec::new();
            let mut c = (*item).child;
            while !c.is_null() {
                match cjson_to_jsonnode_impl(c, visited) {
                    Some(node) => v.push(node),
                    None => return None,
                }
                c = (*c).next;
            }
            Some(JsonNode::Array(v))
        }
        x if x == CJSON_OBJECT => {
            let mut pairs = Vec::new();
            let mut c = (*item).child;
            while !c.is_null() {
                let key = if (*c).string.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr((*c).string)
                        .to_string_lossy()
                        .into_owned()
                };
                match cjson_to_jsonnode_impl(c, visited) {
                    Some(node) => pairs.push((key, node)),
                    None => return None,
                }
                c = (*c).next;
            }
            Some(JsonNode::Object(pairs))
        }
        _ => Some(JsonNode::Null),
    };
    visited.remove(&addr);
    result
}

fn saturating_double_to_int(n: f64) -> c_int {
    if n >= i32::MAX as f64 {
        i32::MAX
    } else if n <= i32::MIN as f64 {
        i32::MIN
    } else {
        n as c_int
    }
}

// ===========================================================================
// Exported C API  (matches cJSON.h declarations)
// ===========================================================================

static VERSION_CSTR: &[u8] = b"1.7.19\0";

#[no_mangle]
pub unsafe extern "C" fn cJSON_Version() -> *const c_char {
    VERSION_CSTR.as_ptr() as *const c_char
}

// ── Hooks ─────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_InitHooks(hooks: *mut cJSON_Hooks) {
    if hooks.is_null() {
        HOOKS_MALLOC.with(|h| *h.borrow_mut() = None);
        HOOKS_FREE.with(|h| *h.borrow_mut() = None);
        GLOBAL_HOOKS.allocate = Some(default_malloc_fn);
        GLOBAL_HOOKS.deallocate = Some(default_free_fn);
        GLOBAL_HOOKS.reallocate = Some(default_realloc_fn);
        return;
    }
    HOOKS_MALLOC.with(|h| *h.borrow_mut() = (*hooks).malloc_fn);
    HOOKS_FREE.with(|h| *h.borrow_mut() = (*hooks).free_fn);

    let mut custom_malloc = false;
    let mut custom_free = false;
    GLOBAL_HOOKS.allocate = Some(default_malloc_fn);
    if let Some(f) = (*hooks).malloc_fn {
        GLOBAL_HOOKS.allocate = Some(f);
        custom_malloc = true;
    }
    GLOBAL_HOOKS.deallocate = Some(default_free_fn);
    if let Some(f) = (*hooks).free_fn {
        GLOBAL_HOOKS.deallocate = Some(f);
        custom_free = true;
    }
    GLOBAL_HOOKS.reallocate = None;
    if !custom_malloc && !custom_free {
        GLOBAL_HOOKS.reallocate = Some(default_realloc_fn);
    }
}

// ── Parsing ───────────────────────────────────────────────────────────────────

/// Strip UTF-8 BOM (0xEF 0xBB 0xBF) from the beginning of a string.
fn strip_utf8_bom(s: &str) -> &str {
    if s.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) {
        &s[3..]
    } else {
        s
    }
}

/// Extract the byte position from any `JsonError` variant.
fn error_position(e: &JsonError) -> usize {
    match e {
        JsonError::UnexpectedEndOfInput { position, .. }
        | JsonError::UnexpectedCharacter { position, .. }
        | JsonError::InvalidNumber { position, .. }
        | JsonError::InvalidString { position, .. }
        | JsonError::SyntaxError { position, .. }
        | JsonError::NestingLimitExceeded { position, .. }
        | JsonError::InvalidEscapeSequence { position, .. }
        | JsonError::DuplicateKey { position, .. }
        | JsonError::TrailingComma { position, .. }
        | JsonError::ExpectedNullTerminator { position, .. }
        | JsonError::TrailingCharacters { position, .. } => *position,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_Parse(value: *const c_char) -> *mut cJSON {
    if value.is_null() {
        return ptr::null_mut();
    }
    let c_str = CStr::from_ptr(value);
    let json_str = match c_str.to_str() {
        Ok(s) => strip_utf8_bom(s),
        Err(_) => return ptr::null_mut(),
    };
    match parser::parse(json_str) {
        Ok(node) => jsonnode_to_cjson(&node),
        Err(_) => {
            ERROR_PTR.with(|p| *p.borrow_mut() = value);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_ParseWithLength(
    value: *const c_char,
    buffer_length: usize,
) -> *mut cJSON {
    if value.is_null() {
        return ptr::null_mut();
    }
    let slice = std::slice::from_raw_parts(value as *const u8, buffer_length);
    let json_str = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match parser::parse_with_length(json_str, buffer_length) {
        Ok(node) => jsonnode_to_cjson(&node),
        Err(_) => {
            ERROR_PTR.with(|p| *p.borrow_mut() = value);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_ParseWithOpts(
    value: *const c_char,
    return_parse_end: *mut *const c_char,
    require_null_terminated: c_int,
) -> *mut cJSON {
    if value.is_null() {
        return ptr::null_mut();
    }
    let c_str = CStr::from_ptr(value);
    let raw_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let bom_offset = if raw_str.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) { 3usize } else { 0usize };
    let json_str = &raw_str[bom_offset..];
    let opts = parser::ParseOptions {
        nesting_limit: 1000,
        require_null_terminated: require_null_terminated != 0,
    };
    match parser::parse_with_opts(json_str, opts) {
        Ok(node) => {
            if !return_parse_end.is_null() {
                let end = json_str.len();
                *return_parse_end = value.add(bom_offset + end);
            }
            jsonnode_to_cjson(&node)
        }
        Err(ref e) => {
            let err_pos = error_position(e) + bom_offset;
            let err_ptr = value.add(err_pos);
            ERROR_PTR.with(|p| *p.borrow_mut() = err_ptr);
            if !return_parse_end.is_null() {
                *return_parse_end = err_ptr;
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_ParseWithLengthOpts(
    value: *const c_char,
    buffer_length: usize,
    return_parse_end: *mut *const c_char,
    require_null_terminated: c_int,
) -> *mut cJSON {
    if value.is_null() || buffer_length == 0 {
        return ptr::null_mut();
    }
    let slice = std::slice::from_raw_parts(value as *const u8, buffer_length);
    let json_str = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let bom_offset = if json_str.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) { 3usize } else { 0usize };
    let parsed_str = &json_str[bom_offset..];
    let opts = parser::ParseOptions {
        nesting_limit: 1000,
        require_null_terminated: require_null_terminated != 0,
    };
    match parser::parse_with_opts(parsed_str, opts) {
        Ok(node) => {
            if !return_parse_end.is_null() {
                *return_parse_end = value.add(bom_offset + parsed_str.len());
            }
            jsonnode_to_cjson(&node)
        }
        Err(_) => {
            ERROR_PTR.with(|p| *p.borrow_mut() = value);
            if !return_parse_end.is_null() {
                *return_parse_end = value;
            }
            ptr::null_mut()
        }
    }
}

// ── Printing ──────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Print(item: *const cJSON) -> *mut c_char {
    if item.is_null() {
        return ptr::null_mut();
    }
    let node = cjson_to_jsonnode(item);
    let output = serializer::print(&node);
    cstr_from_rust(&output)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_PrintUnformatted(item: *const cJSON) -> *mut c_char {
    if item.is_null() {
        return ptr::null_mut();
    }
    let node = cjson_to_jsonnode(item);
    let output = serializer::print_unformatted(&node);
    cstr_from_rust(&output)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_PrintBuffered(
    item: *const cJSON,
    prebuffer: c_int,
    fmt: c_int,
) -> *mut c_char {
    if item.is_null() {
        return ptr::null_mut();
    }
    let node = cjson_to_jsonnode(item);
    let output = serializer::print_buffered(&node, prebuffer.max(0) as usize, fmt != 0);
    cstr_from_rust(&output)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_PrintPreallocated(
    item: *mut cJSON,
    buffer: *mut c_char,
    length: c_int,
    format: c_int,
) -> c_int {
    if item.is_null() || buffer.is_null() || length <= 0 {
        return 0;
    }
    let node = cjson_to_jsonnode(item);
    let output = if format != 0 {
        serializer::print(&node)
    } else {
        serializer::print_unformatted(&node)
    };
    let bytes = output.as_bytes();
    if bytes.len() >= length as usize {
        return 0;
    }
    ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
    *buffer.add(bytes.len()) = 0;
    1
}

// ── Delete ────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Delete(item: *mut cJSON) {
    let mut current = item;
    while !current.is_null() {
        let next = (*current).next;
        // Only recurse into children if this item is NOT a reference
        if (*current).type_ & CJSON_IS_REFERENCE == 0 && !(*current).child.is_null() {
            cJSON_Delete((*current).child);
        }
        // Free valuestring only if not a reference
        if (*current).type_ & CJSON_IS_REFERENCE == 0 && !(*current).valuestring.is_null() {
            internal_free((*current).valuestring as *mut c_void);
        }
        // Free key name unless const
        if (*current).type_ & CJSON_STRING_IS_CONST == 0 && !(*current).string.is_null() {
            internal_free((*current).string as *mut c_void);
        }
        internal_free(current as *mut c_void);
        current = next;
    }
}

// ── Array / Object accessors ──────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetArraySize(array: *const cJSON) -> c_int {
    if array.is_null() {
        return 0;
    }
    let mut count: c_int = 0;
    let mut c = (*array).child;
    while !c.is_null() {
        count += 1;
        c = (*c).next;
    }
    count
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetArrayItem(array: *const cJSON, index: c_int) -> *mut cJSON {
    if array.is_null() || index < 0 {
        return ptr::null_mut();
    }
    let mut c = (*array).child;
    let mut i: c_int = 0;
    while !c.is_null() {
        if i == index {
            return c;
        }
        i += 1;
        c = (*c).next;
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetObjectItem(
    object: *const cJSON,
    string: *const c_char,
) -> *mut cJSON {
    if object.is_null() || string.is_null() {
        return ptr::null_mut();
    }
    let key = match CStr::from_ptr(string).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut c = (*object).child;
    while !c.is_null() {
        if !(*c).string.is_null() {
            if let Ok(k) = CStr::from_ptr((*c).string).to_str() {
                if k.eq_ignore_ascii_case(key) {
                    return c;
                }
            }
        }
        c = (*c).next;
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetObjectItemCaseSensitive(
    object: *const cJSON,
    string: *const c_char,
) -> *mut cJSON {
    if object.is_null() || string.is_null() {
        return ptr::null_mut();
    }
    let key = match CStr::from_ptr(string).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut c = (*object).child;
    while !c.is_null() {
        if !(*c).string.is_null() {
            if let Ok(k) = CStr::from_ptr((*c).string).to_str() {
                if k == key {
                    return c;
                }
            }
        }
        c = (*c).next;
    }
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_HasObjectItem(
    object: *const cJSON,
    string: *const c_char,
) -> c_int {
    if cJSON_GetObjectItem(object, string).is_null() {
        0
    } else {
        1
    }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetErrorPtr() -> *const c_char {
    ERROR_PTR.with(|p| *p.borrow())
}

// ── Value getters ─────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetStringValue(item: *const cJSON) -> *mut c_char {
    if item.is_null() || ((*item).type_ & 0xFF) != CJSON_STRING {
        return ptr::null_mut();
    }
    (*item).valuestring
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetNumberValue(item: *const cJSON) -> c_double {
    if item.is_null() || ((*item).type_ & 0xFF) != CJSON_NUMBER {
        return f64::NAN;
    }
    (*item).valuedouble
}

// ── Type predicates ───────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsInvalid(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_INVALID { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsFalse(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_FALSE { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsTrue(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_TRUE { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsBool(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    let t = (*item).type_ & 0xFF;
    if t == CJSON_FALSE || t == CJSON_TRUE { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsNull(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_NULL { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsNumber(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_NUMBER { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsString(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_STRING { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsArray(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_ARRAY { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsObject(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_OBJECT { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_IsRaw(item: *const cJSON) -> c_int {
    if item.is_null() { return 0; }
    if ((*item).type_ & 0xFF) == CJSON_RAW { 1 } else { 0 }
}

// ── Creation ──────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateNull() -> *mut cJSON {
    let item = new_item();
    if !item.is_null() { (*item).type_ = CJSON_NULL; }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateTrue() -> *mut cJSON {
    let item = new_item();
    if !item.is_null() { (*item).type_ = CJSON_TRUE; }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateFalse() -> *mut cJSON {
    let item = new_item();
    if !item.is_null() { (*item).type_ = CJSON_FALSE; }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateBool(boolean: c_int) -> *mut cJSON {
    let item = new_item();
    if !item.is_null() {
        (*item).type_ = if boolean != 0 { CJSON_TRUE } else { CJSON_FALSE };
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateNumber(num: c_double) -> *mut cJSON {
    let item = new_item();
    if !item.is_null() {
        (*item).type_ = CJSON_NUMBER;
        (*item).valuedouble = num;
        (*item).valueint = saturating_double_to_int(num);
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateString(string: *const c_char) -> *mut cJSON {
    if string.is_null() { return ptr::null_mut(); }
    let item = new_item();
    if item.is_null() { return ptr::null_mut(); }
    (*item).type_ = CJSON_STRING;
    (*item).valuestring = cstr_dup(string);
    if (*item).valuestring.is_null() {
        internal_free(item as *mut c_void);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateRaw(raw: *const c_char) -> *mut cJSON {
    if raw.is_null() { return ptr::null_mut(); }
    let item = new_item();
    if item.is_null() { return ptr::null_mut(); }
    (*item).type_ = CJSON_RAW;
    (*item).valuestring = cstr_dup(raw);
    if (*item).valuestring.is_null() {
        internal_free(item as *mut c_void);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateArray() -> *mut cJSON {
    let item = new_item();
    if !item.is_null() { (*item).type_ = CJSON_ARRAY; }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateObject() -> *mut cJSON {
    let item = new_item();
    if !item.is_null() { (*item).type_ = CJSON_OBJECT; }
    item
}

// ── Reference creation ───────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateStringReference(string: *const c_char) -> *mut cJSON {
    let item = new_item();
    if item.is_null() { return ptr::null_mut(); }
    (*item).type_ = CJSON_STRING | CJSON_IS_REFERENCE;
    (*item).valuestring = string as *mut c_char;
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateObjectReference(child: *const cJSON) -> *mut cJSON {
    if child.is_null() { return ptr::null_mut(); }
    let item = new_item();
    if item.is_null() { return ptr::null_mut(); }
    (*item).type_ = CJSON_OBJECT | CJSON_IS_REFERENCE;
    (*item).child = child as *mut cJSON;
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateArrayReference(child: *const cJSON) -> *mut cJSON {
    if child.is_null() { return ptr::null_mut(); }
    let item = new_item();
    if item.is_null() { return ptr::null_mut(); }
    (*item).type_ = CJSON_ARRAY | CJSON_IS_REFERENCE;
    (*item).child = child as *mut cJSON;
    item
}

// ── Array creation utilities ──────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateIntArray(
    numbers: *const c_int,
    count: c_int,
) -> *mut cJSON {
    if numbers.is_null() || count <= 0 { return ptr::null_mut(); }
    let arr = cJSON_CreateArray();
    if arr.is_null() { return ptr::null_mut(); }
    for i in 0..count {
        let n = cJSON_CreateNumber(*numbers.add(i as usize) as c_double);
        if n.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if i == 0 {
            (*arr).child = n;
        } else {
            suffix_object(cJSON_GetArrayItem(arr, i - 1), n);
        }
    }
    arr
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateFloatArray(
    numbers: *const f32,
    count: c_int,
) -> *mut cJSON {
    if numbers.is_null() || count <= 0 { return ptr::null_mut(); }
    let arr = cJSON_CreateArray();
    if arr.is_null() { return ptr::null_mut(); }
    for i in 0..count {
        let n = cJSON_CreateNumber(*numbers.add(i as usize) as c_double);
        if n.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if i == 0 {
            (*arr).child = n;
        } else {
            suffix_object(cJSON_GetArrayItem(arr, i - 1), n);
        }
    }
    arr
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateDoubleArray(
    numbers: *const c_double,
    count: c_int,
) -> *mut cJSON {
    if numbers.is_null() || count <= 0 { return ptr::null_mut(); }
    let arr = cJSON_CreateArray();
    if arr.is_null() { return ptr::null_mut(); }
    for i in 0..count {
        let n = cJSON_CreateNumber(*numbers.add(i as usize));
        if n.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if i == 0 {
            (*arr).child = n;
        } else {
            suffix_object(cJSON_GetArrayItem(arr, i - 1), n);
        }
    }
    arr
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateStringArray(
    strings: *const *const c_char,
    count: c_int,
) -> *mut cJSON {
    if strings.is_null() || count <= 0 { return ptr::null_mut(); }
    let arr = cJSON_CreateArray();
    if arr.is_null() { return ptr::null_mut(); }
    for i in 0..count {
        let s = cJSON_CreateString(*strings.add(i as usize));
        if s.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if i == 0 {
            (*arr).child = s;
        } else {
            suffix_object(cJSON_GetArrayItem(arr, i - 1), s);
        }
    }
    arr
}

// ── Add item funcations ──────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemToArray(
    array: *mut cJSON,
    item: *mut cJSON,
) -> c_int {
    add_item_to_array(array, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemToObject(
    object: *mut cJSON,
    string: *const c_char,
    item: *mut cJSON,
) -> c_int {
    if object.is_null() || string.is_null() || item.is_null() {
        return 0;
    }
    if (*item).type_ & CJSON_STRING_IS_CONST == 0 && !(*item).string.is_null() {
        internal_free((*item).string as *mut c_void);
    }
    (*item).string = cstr_dup(string);
    (*item).type_ &= !CJSON_STRING_IS_CONST;
    cJSON_AddItemToArray(object, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemToObjectCS(
    object: *mut cJSON,
    string: *const c_char,
    item: *mut cJSON,
) -> c_int {
    if object.is_null() || string.is_null() || item.is_null() {
        return 0;
    }
    if (*item).type_ & CJSON_STRING_IS_CONST == 0 && !(*item).string.is_null() {
        internal_free((*item).string as *mut c_void);
    }
    (*item).string = string as *mut c_char;
    (*item).type_ |= CJSON_STRING_IS_CONST;
    cJSON_AddItemToArray(object, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemReferenceToArray(
    array: *mut cJSON,
    item: *mut cJSON,
) -> c_int {
    if array.is_null() || item.is_null() { return 0; }
    let ref_item = new_item();
    if ref_item.is_null() { return 0; }
    ptr::copy_nonoverlapping(item, ref_item, 1);
    (*ref_item).type_ |= CJSON_IS_REFERENCE;
    (*ref_item).prev = ptr::null_mut();
    (*ref_item).next = ptr::null_mut();
    (*ref_item).string = ptr::null_mut();
    cJSON_AddItemToArray(array, ref_item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemReferenceToObject(
    object: *mut cJSON,
    string: *const c_char,
    item: *mut cJSON,
) -> c_int {
    if object.is_null() || string.is_null() || item.is_null() { return 0; }
    let ref_item = new_item();
    if ref_item.is_null() { return 0; }
    ptr::copy_nonoverlapping(item, ref_item, 1);
    (*ref_item).type_ |= CJSON_IS_REFERENCE;
    (*ref_item).prev = ptr::null_mut();
    (*ref_item).next = ptr::null_mut();
    (*ref_item).string = cstr_dup(string);
    cJSON_AddItemToArray(object, ref_item)
}

// ── Detach / Delete ───────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_DetachItemViaPointer(
    parent: *mut cJSON,
    item: *mut cJSON,
) -> *mut cJSON {
    if parent.is_null() || item.is_null() { return ptr::null_mut(); }
    if item != (*parent).child && (*item).prev.is_null() { return ptr::null_mut(); }
    if item != (*parent).child {
        // not the first element
        (*(*item).prev).next = (*item).next;
    }
    if !(*item).next.is_null() {
        // not the last element
        (*(*item).next).prev = (*item).prev;
    }
    if item == (*parent).child {
        // first element
        (*parent).child = (*item).next;
    } else if (*item).next.is_null() {
        // last element - update tail pointer
        (*(*parent).child).prev = (*item).prev;
    }
    (*item).prev = ptr::null_mut();
    (*item).next = ptr::null_mut();
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_DetachItemFromArray(
    array: *mut cJSON,
    which: c_int,
) -> *mut cJSON {
    if which < 0 { return ptr::null_mut(); }
    let item = cJSON_GetArrayItem(array, which);
    if item.is_null() { return ptr::null_mut(); }
    cJSON_DetachItemViaPointer(array, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_DeleteItemFromArray(array: *mut cJSON, which: c_int) {
    let item = cJSON_DetachItemFromArray(array, which);
    if !item.is_null() { cJSON_Delete(item); }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_DetachItemFromObject(
    object: *mut cJSON,
    string: *const c_char,
) -> *mut cJSON {
    let item = cJSON_GetObjectItem(object, string);
    if item.is_null() { return ptr::null_mut(); }
    cJSON_DetachItemViaPointer(object, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_DetachItemFromObjectCaseSensitive(
    object: *mut cJSON,
    string: *const c_char,
) -> *mut cJSON {
    let item = cJSON_GetObjectItemCaseSensitive(object, string);
    if item.is_null() { return ptr::null_mut(); }
    cJSON_DetachItemViaPointer(object, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_DeleteItemFromObject(
    object: *mut cJSON,
    string: *const c_char,
) {
    let item = cJSON_DetachItemFromObject(object, string);
    if !item.is_null() { cJSON_Delete(item); }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_DeleteItemFromObjectCaseSensitive(
    object: *mut cJSON,
    string: *const c_char,
) {
    let item = cJSON_DetachItemFromObjectCaseSensitive(object, string);
    if !item.is_null() { cJSON_Delete(item); }
}

// ── Insert / Replace ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_InsertItemInArray(
    array: *mut cJSON,
    which: c_int,
    newitem: *mut cJSON,
) -> c_int {
    if array.is_null() || newitem.is_null() || which < 0 { return 0; }
    let after_inserted = cJSON_GetArrayItem(array, which);
    if after_inserted.is_null() {
        return add_item_to_array(array, newitem);
    }
    if after_inserted != (*array).child && (*after_inserted).prev.is_null() {
        return 0;
    }
    (*newitem).next = after_inserted;
    (*newitem).prev = (*after_inserted).prev;
    (*after_inserted).prev = newitem;
    if after_inserted == (*array).child {
        (*array).child = newitem;
    } else {
        (*(*newitem).prev).next = newitem;
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_ReplaceItemViaPointer(
    parent: *mut cJSON,
    item: *mut cJSON,
    replacement: *mut cJSON,
) -> c_int {
    if parent.is_null() || (*parent).child.is_null() || replacement.is_null() || item.is_null() {
        return 0;
    }
    if replacement == item { return 1; }
    (*replacement).next = (*item).next;
    (*replacement).prev = (*item).prev;
    if !(*replacement).next.is_null() {
        (*(*replacement).next).prev = replacement;
    }
    if (*parent).child == item {
        if (*(*parent).child).prev == (*parent).child {
            (*replacement).prev = replacement;
        }
        (*parent).child = replacement;
    } else {
        if !(*replacement).prev.is_null() {
            (*(*replacement).prev).next = replacement;
        }
        if (*replacement).next.is_null() {
            (*(*parent).child).prev = replacement;
        }
    }
    (*item).next = ptr::null_mut();
    (*item).prev = ptr::null_mut();
    // Preserve the key
    if !(*item).string.is_null() && (*replacement).string.is_null() {
        (*replacement).string = (*item).string;
        (*item).string = ptr::null_mut();
    }
    cJSON_Delete(item);
    1
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_ReplaceItemInArray(
    array: *mut cJSON,
    which: c_int,
    newitem: *mut cJSON,
) -> c_int {
    let existing = cJSON_GetArrayItem(array, which);
    if existing.is_null() { return 0; }
    cJSON_ReplaceItemViaPointer(array, existing, newitem)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_ReplaceItemInObject(
    object: *mut cJSON,
    string: *const c_char,
    newitem: *mut cJSON,
) -> c_int {
    if object.is_null() || string.is_null() || newitem.is_null() { return 0; }
    if (*newitem).type_ & CJSON_STRING_IS_CONST == 0 && !(*newitem).string.is_null() {
        internal_free((*newitem).string as *mut c_void);
    }
    (*newitem).string = cstr_dup(string);
    (*newitem).type_ &= !CJSON_STRING_IS_CONST;
    let existing = cJSON_GetObjectItem(object, string);
    if existing.is_null() { return 0; }
    cJSON_ReplaceItemViaPointer(object, existing, newitem)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_ReplaceItemInObjectCaseSensitive(
    object: *mut cJSON,
    string: *const c_char,
    newitem: *mut cJSON,
) -> c_int {
    if object.is_null() || string.is_null() || newitem.is_null() { return 0; }
    if (*newitem).type_ & CJSON_STRING_IS_CONST == 0 && !(*newitem).string.is_null() {
        internal_free((*newitem).string as *mut c_void);
    }
    (*newitem).string = cstr_dup(string);
    (*newitem).type_ &= !CJSON_STRING_IS_CONST;
    let existing = cJSON_GetObjectItemCaseSensitive(object, string);
    if existing.is_null() { return 0; }
    cJSON_ReplaceItemViaPointer(object, existing, newitem)
}

// ── Duplicate ─────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Duplicate(item: *const cJSON, recurse: c_int) -> *mut cJSON {
    if item.is_null() { return ptr::null_mut(); }
    let node = match cjson_to_jsonnode_checked(item) {
        Some(n) => n,
        None => return ptr::null_mut(), // circular reference detected
    };
    let dup_node = utils::duplicate(&node, recurse != 0);
    let dup = jsonnode_to_cjson(&dup_node);
    // Preserve the key name from the original item
    if !dup.is_null() && !(*item).string.is_null() {
        (*dup).string = cstr_dup((*item).string);
    }
    dup
}

// ── Compare ───────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Compare(
    a: *const cJSON,
    b: *const cJSON,
    case_sensitive: c_int,
) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }
    let node_a = cjson_to_jsonnode(a);
    let node_b = cjson_to_jsonnode(b);
    if utils::compare(&node_a, &node_b, case_sensitive != 0) { 1 } else { 0 }
}

// ── Minify ────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Minify(json: *mut c_char) {
    if json.is_null() { return; }
    let c_str = CStr::from_ptr(json);
    let json_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    if let Ok(minified) = serializer::minify(json_str) {
        let bytes = minified.as_bytes();
        ptr::copy_nonoverlapping(bytes.as_ptr(), json as *mut u8, bytes.len());
        *json.add(bytes.len()) = 0;
    }
}

// ── Convenience add-to-object helpers ─────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddNullToObject(
    object: *mut cJSON,
    name: *const c_char,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateNull();
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddTrueToObject(
    object: *mut cJSON,
    name: *const c_char,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateTrue();
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddFalseToObject(
    object: *mut cJSON,
    name: *const c_char,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateFalse();
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddBoolToObject(
    object: *mut cJSON,
    name: *const c_char,
    boolean: c_int,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateBool(boolean);
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddNumberToObject(
    object: *mut cJSON,
    name: *const c_char,
    number: c_double,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateNumber(number);
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddStringToObject(
    object: *mut cJSON,
    name: *const c_char,
    string: *const c_char,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateString(string);
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddRawToObject(
    object: *mut cJSON,
    name: *const c_char,
    raw: *const c_char,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateRaw(raw);
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddObjectToObject(
    object: *mut cJSON,
    name: *const c_char,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateObject();
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddArrayToObject(
    object: *mut cJSON,
    name: *const c_char,
) -> *mut cJSON {
    if object.is_null() || name.is_null() { return ptr::null_mut(); }
    let item = cJSON_CreateArray();
    if item.is_null() { return ptr::null_mut(); }
    if cJSON_AddItemToObject(object, name, item) == 0 {
        cJSON_Delete(item);
        return ptr::null_mut();
    }
    item
}

// ── Number / string setters ───────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_SetNumberHelper(object: *mut cJSON, number: c_double) -> c_double {
    if object.is_null() { return number; }
    (*object).valuedouble = number;
    (*object).valueint = saturating_double_to_int(number);
    number
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_SetValuestring(
    object: *mut cJSON,
    valuestring: *const c_char,
) -> *mut c_char {
    if object.is_null() || valuestring.is_null() { return ptr::null_mut(); }
    if ((*object).type_ & 0xFF) != CJSON_STRING { return ptr::null_mut(); }
    if (*object).type_ & CJSON_IS_REFERENCE != 0 { return ptr::null_mut(); }
    if (*object).valuestring.is_null() { return ptr::null_mut(); }

    let v1_len = strlen(valuestring);
    let v2_len = strlen((*object).valuestring);

    if v1_len <= v2_len {
        // Check for overlap: [valuestring, valuestring+v1_len] must not overlap [valuestring_old, valuestring_old+v2_len]
        let new_start = valuestring as usize;
        let new_end = new_start + v1_len;
        let old_start = (*object).valuestring as usize;
        let old_end = old_start + v2_len;
        if !(new_end < old_start || old_end < new_start) {
            // strings overlap
            return ptr::null_mut();
        }
        strcpy((*object).valuestring, valuestring);
        return (*object).valuestring;
    }

    // New string is longer, need to reallocate
    let new_str = cstr_dup(valuestring);
    if new_str.is_null() { return ptr::null_mut(); }
    internal_free((*object).valuestring as *mut c_void);
    (*object).valuestring = new_str;
    new_str
}

// ── Allocator pass-through ────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_malloc(size: usize) -> *mut c_void {
    internal_malloc(size)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_free(object: *mut c_void) {
    internal_free(object)
}

// ===========================================================================
// Newly exposed internal API (matches modified cJSON.h)
// ===========================================================================

// ── global_hooks_get ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn global_hooks_get() -> *mut internal_hooks {
    &raw mut GLOBAL_HOOKS
}

// ── cJSON_strdup ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_strdup(
    string: *const u8,
    hooks: *const internal_hooks,
) -> *mut u8 {
    if string.is_null() {
        return ptr::null_mut();
    }
    let length = strlen(string as *const c_char) + 1;
    let copy = hooks_allocate(&*hooks, length) as *mut u8;
    if copy.is_null() {
        return ptr::null_mut();
    }
    memcpy(copy as *mut c_void, string as *const c_void, length);
    copy
}

// ── compare_double ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn compare_double(a: c_double, b: c_double) -> c_int {
    let max_val = if a.abs() > b.abs() { a.abs() } else { b.abs() };
    if (a - b).abs() <= max_val * f64::EPSILON { 1 } else { 0 }
}

// ── add_item_to_array ─────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn add_item_to_array(
    array: *mut cJSON,
    item: *mut cJSON,
) -> c_int {
    if item.is_null() || array.is_null() || array == item {
        return 0;
    }
    let child = (*array).child;
    if child.is_null() {
        (*array).child = item;
        (*item).prev = item; // tail pointer to self
        (*item).next = ptr::null_mut();
    } else {
        // append to the end using tail pointer
        if !(*child).prev.is_null() {
            suffix_object((*child).prev, item);
            (*(*array).child).prev = item;
        }
    }
    1
}

// ── ensure (printbuffer) ──────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn ensure(
    p: *mut printbuffer,
    needed: usize,
) -> *mut u8 {
    if p.is_null() || (*p).buffer.is_null() {
        return ptr::null_mut();
    }
    if (*p).length > 0 && (*p).offset >= (*p).length {
        return ptr::null_mut();
    }
    if needed > i32::MAX as usize {
        return ptr::null_mut();
    }
    let needed_total = needed + (*p).offset + 1;
    if needed_total <= (*p).length {
        return (*p).buffer.add((*p).offset);
    }
    if (*p).noalloc != 0 {
        return ptr::null_mut();
    }

    let newsize: usize;
    if needed_total > (i32::MAX as usize / 2) {
        if needed_total <= i32::MAX as usize {
            newsize = i32::MAX as usize;
        } else {
            return ptr::null_mut();
        }
    } else {
        newsize = needed_total * 2;
    }

    if (*p).hooks.reallocate.is_some() {
        let newbuffer = hooks_reallocate(&(*p).hooks, (*p).buffer as *mut c_void, newsize) as *mut u8;
        if newbuffer.is_null() {
            hooks_deallocate(&(*p).hooks, (*p).buffer as *mut c_void);
            (*p).length = 0;
            (*p).buffer = ptr::null_mut();
            return ptr::null_mut();
        }
        (*p).length = newsize;
        (*p).buffer = newbuffer;
    } else {
        let newbuffer = hooks_allocate(&(*p).hooks, newsize) as *mut u8;
        if newbuffer.is_null() {
            hooks_deallocate(&(*p).hooks, (*p).buffer as *mut c_void);
            (*p).length = 0;
            (*p).buffer = ptr::null_mut();
            return ptr::null_mut();
        }
        memcpy(newbuffer as *mut c_void, (*p).buffer as *const c_void, (*p).offset + 1);
        hooks_deallocate(&(*p).hooks, (*p).buffer as *mut c_void);
        (*p).length = newsize;
        (*p).buffer = newbuffer;
    }
    (*p).buffer.add((*p).offset)
}

// ── update_offset (internal helper) ──────────────────────────────────────────

unsafe fn update_offset(buffer: *mut printbuffer) {
    if buffer.is_null() || (*buffer).buffer.is_null() {
        return;
    }
    let bp = (*buffer).buffer.add((*buffer).offset);
    (*buffer).offset += strlen(bp as *const c_char);
}

// ── skip_utf8_bom ─────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn skip_utf8_bom(
    buffer: *mut parse_buffer,
) -> *mut parse_buffer {
    if buffer.is_null() || (*buffer).content.is_null() || (*buffer).offset != 0 {
        return ptr::null_mut();
    }
    if pb_can_access(buffer, 4)
        && strncmp(
            (*buffer).content.add((*buffer).offset) as *const c_char,
            b"\xEF\xBB\xBF\0".as_ptr() as *const c_char,
            3,
        ) == 0
    {
        (*buffer).offset += 3;
    }
    buffer
}

// ── parse_buffer helpers ──────────────────────────────────────────────────────

#[inline]
unsafe fn pb_can_read(buffer: *const parse_buffer, size: usize) -> bool {
    !buffer.is_null() && ((*buffer).offset + size) <= (*buffer).length
}

#[inline]
unsafe fn pb_can_access(buffer: *const parse_buffer, index: usize) -> bool {
    !buffer.is_null() && ((*buffer).offset + index) < (*buffer).length
}

#[inline]
unsafe fn pb_at_offset(buffer: *const parse_buffer) -> *const u8 {
    (*buffer).content.add((*buffer).offset)
}

unsafe fn buffer_skip_whitespace(buffer: *mut parse_buffer) -> *mut parse_buffer {
    if buffer.is_null() || (*buffer).content.is_null() {
        return ptr::null_mut();
    }
    if !pb_can_access(buffer, 0) {
        return buffer;
    }
    while pb_can_access(buffer, 0) && *pb_at_offset(buffer) <= 32 {
        (*buffer).offset += 1;
    }
    if (*buffer).offset == (*buffer).length {
        (*buffer).offset -= 1;
    }
    buffer
}

unsafe fn cjson_new_item(hooks: &internal_hooks) -> *mut cJSON {
    let node = hooks_allocate(hooks, std::mem::size_of::<cJSON>()) as *mut cJSON;
    if !node.is_null() {
        ptr::write_bytes(node, 0, 1);
    }
    node
}

// ── parse_hex4 ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn parse_hex4(input: *const u8) -> u32 {
    let mut h: u32 = 0;
    for i in 0..4usize {
        let c = *input.add(i);
        if c >= b'0' && c <= b'9' {
            h += (c - b'0') as u32;
        } else if c >= b'A' && c <= b'F' {
            h += 10 + (c - b'A') as u32;
        } else if c >= b'a' && c <= b'f' {
            h += 10 + (c - b'a') as u32;
        } else {
            return 0;
        }
        if i < 3 {
            h <<= 4;
        }
    }
    h
}

// ── utf16_literal_to_utf8 (internal helper) ──────────────────────────────────

unsafe fn utf16_literal_to_utf8(
    input_pointer: *const u8,
    input_end: *const u8,
    output_pointer: *mut *mut u8,
) -> u8 {
    let first_sequence = input_pointer;
    if (input_end as usize) - (first_sequence as usize) < 6 {
        return 0;
    }
    let first_code = parse_hex4(first_sequence.add(2));
    if first_code >= 0xDC00 && first_code <= 0xDFFF {
        return 0;
    }

    let codepoint: u32;
    let sequence_length: u8;

    if first_code >= 0xD800 && first_code <= 0xDBFF {
        let second_sequence = first_sequence.add(6);
        sequence_length = 12;
        if (input_end as usize) - (second_sequence as usize) < 6 {
            return 0;
        }
        if *second_sequence != b'\\' || *second_sequence.add(1) != b'u' {
            return 0;
        }
        let second_code = parse_hex4(second_sequence.add(2));
        if second_code < 0xDC00 || second_code > 0xDFFF {
            return 0;
        }
        codepoint = 0x10000 + (((first_code & 0x3FF) << 10) | (second_code & 0x3FF));
    } else {
        sequence_length = 6;
        codepoint = first_code;
    }

    let utf8_length: u8;
    let first_byte_mark: u8;
    if codepoint < 0x80 {
        utf8_length = 1;
        first_byte_mark = 0;
    } else if codepoint < 0x800 {
        utf8_length = 2;
        first_byte_mark = 0xC0;
    } else if codepoint < 0x10000 {
        utf8_length = 3;
        first_byte_mark = 0xE0;
    } else if codepoint <= 0x10FFFF {
        utf8_length = 4;
        first_byte_mark = 0xF0;
    } else {
        return 0;
    }

    let mut cp = codepoint;
    let out = *output_pointer;
    let mut pos = utf8_length as isize - 1;
    while pos > 0 {
        *out.offset(pos) = ((cp | 0x80) & 0xBF) as u8;
        cp >>= 6;
        pos -= 1;
    }
    if utf8_length > 1 {
        *out.offset(0) = ((cp | first_byte_mark as u32) & 0xFF) as u8;
    } else {
        *out.offset(0) = (cp & 0x7F) as u8;
    }
    *output_pointer = out.add(utf8_length as usize);
    sequence_length
}

// ── parse_number ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn parse_number(
    item: *mut cJSON,
    input_buffer: *mut parse_buffer,
) -> c_int {
    if input_buffer.is_null() || (*input_buffer).content.is_null() {
        return 0;
    }
    let mut number_string_length: usize = 0;
    let mut i: usize = 0;
    while pb_can_access(input_buffer, i) {
        let c = *pb_at_offset(input_buffer).add(i);
        match c {
            b'0'..=b'9' | b'+' | b'-' | b'e' | b'E' => { number_string_length += 1; }
            b'.' => { number_string_length += 1; }
            _ => break,
        }
        i += 1;
    }

    let number_c_string = hooks_allocate(&(*input_buffer).hooks, number_string_length + 1) as *mut u8;
    if number_c_string.is_null() {
        return 0;
    }
    memcpy(
        number_c_string as *mut c_void,
        pb_at_offset(input_buffer) as *const c_void,
        number_string_length,
    );
    *number_c_string.add(number_string_length) = 0;

    let mut after_end: *mut c_char = ptr::null_mut();
    let number = strtod(number_c_string as *const c_char, &mut after_end);
    if number_c_string as *mut c_char == after_end {
        hooks_deallocate(&(*input_buffer).hooks, number_c_string as *mut c_void);
        return 0;
    }

    (*item).valuedouble = number;
    if number >= i32::MAX as f64 {
        (*item).valueint = i32::MAX;
    } else if number <= i32::MIN as f64 {
        (*item).valueint = i32::MIN;
    } else {
        (*item).valueint = number as c_int;
    }
    (*item).type_ = CJSON_NUMBER;

    (*input_buffer).offset += (after_end as usize) - (number_c_string as usize);
    hooks_deallocate(&(*input_buffer).hooks, number_c_string as *mut c_void);
    1
}

// ── parse_string ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn parse_string(
    item: *mut cJSON,
    input_buffer: *mut parse_buffer,
) -> c_int {
    if !pb_can_access(input_buffer, 0) || *pb_at_offset(input_buffer) != b'\"' {
        return 0;
    }

    let input_start = pb_at_offset(input_buffer).add(1);
    let mut input_end = input_start;
    let mut skipped_bytes: usize = 0;

    while ((input_end as usize) - ((*input_buffer).content as usize)) < (*input_buffer).length
        && *input_end != b'\"'
    {
        if *input_end == b'\\' {
            if ((input_end.add(1) as usize) - ((*input_buffer).content as usize)) >= (*input_buffer).length {
                // buffer overflow protection
                return 0; // goto fail
            }
            skipped_bytes += 1;
            input_end = input_end.add(1);
        }
        input_end = input_end.add(1);
    }

    if ((input_end as usize) - ((*input_buffer).content as usize)) >= (*input_buffer).length
        || *input_end != b'\"'
    {
        return 0;
    }

    let allocation_length = (input_end as usize) - (pb_at_offset(input_buffer) as usize) - skipped_bytes;
    let output = hooks_allocate(&(*input_buffer).hooks, allocation_length + 1) as *mut u8;
    if output.is_null() {
        return 0;
    }

    let mut output_pointer = output;
    let mut input_pointer = input_start;

    while (input_pointer as usize) < (input_end as usize) {
        if *input_pointer != b'\\' {
            *output_pointer = *input_pointer;
            output_pointer = output_pointer.add(1);
            input_pointer = input_pointer.add(1);
        } else {
            if (input_end as usize) - (input_pointer as usize) < 1 {
                hooks_deallocate(&(*input_buffer).hooks, output as *mut c_void);
                return 0;
            }
            let mut sequence_length: u8 = 2;
            match *input_pointer.add(1) {
                b'b' => { *output_pointer = b'\x08'; output_pointer = output_pointer.add(1); }
                b'f' => { *output_pointer = b'\x0C'; output_pointer = output_pointer.add(1); }
                b'n' => { *output_pointer = b'\n'; output_pointer = output_pointer.add(1); }
                b'r' => { *output_pointer = b'\r'; output_pointer = output_pointer.add(1); }
                b't' => { *output_pointer = b'\t'; output_pointer = output_pointer.add(1); }
                b'\"' | b'\\' | b'/' => {
                    *output_pointer = *input_pointer.add(1);
                    output_pointer = output_pointer.add(1);
                }
                b'u' => {
                    sequence_length = utf16_literal_to_utf8(input_pointer, input_end, &mut output_pointer);
                    if sequence_length == 0 {
                        hooks_deallocate(&(*input_buffer).hooks, output as *mut c_void);
                        return 0;
                    }
                }
                _ => {
                    hooks_deallocate(&(*input_buffer).hooks, output as *mut c_void);
                    return 0;
                }
            }
            input_pointer = input_pointer.add(sequence_length as usize);
        }
    }

    *output_pointer = 0;
    (*item).type_ = CJSON_STRING;
    (*item).valuestring = output as *mut c_char;
    (*input_buffer).offset = (input_end as usize) - ((*input_buffer).content as usize);
    (*input_buffer).offset += 1;
    1
}

// ── parse_value ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn parse_value(
    item: *mut cJSON,
    input_buffer: *mut parse_buffer,
) -> c_int {
    if input_buffer.is_null() || (*input_buffer).content.is_null() {
        return 0;
    }
    // null
    if pb_can_read(input_buffer, 4)
        && strncmp(pb_at_offset(input_buffer) as *const c_char, b"null\0".as_ptr() as *const c_char, 4) == 0
    {
        (*item).type_ = CJSON_NULL;
        (*input_buffer).offset += 4;
        return 1;
    }
    // false
    if pb_can_read(input_buffer, 5)
        && strncmp(pb_at_offset(input_buffer) as *const c_char, b"false\0".as_ptr() as *const c_char, 5) == 0
    {
        (*item).type_ = CJSON_FALSE;
        (*input_buffer).offset += 5;
        return 1;
    }
    // true
    if pb_can_read(input_buffer, 4)
        && strncmp(pb_at_offset(input_buffer) as *const c_char, b"true\0".as_ptr() as *const c_char, 4) == 0
    {
        (*item).type_ = CJSON_TRUE;
        (*item).valueint = 1;
        (*input_buffer).offset += 4;
        return 1;
    }
    // string
    if pb_can_access(input_buffer, 0) && *pb_at_offset(input_buffer) == b'\"' {
        return parse_string(item, input_buffer);
    }
    // number
    if pb_can_access(input_buffer, 0) {
        let c = *pb_at_offset(input_buffer);
        if c == b'-' || (c >= b'0' && c <= b'9') {
            return parse_number(item, input_buffer);
        }
    }
    // array
    if pb_can_access(input_buffer, 0) && *pb_at_offset(input_buffer) == b'[' {
        return parse_array(item, input_buffer);
    }
    // object
    if pb_can_access(input_buffer, 0) && *pb_at_offset(input_buffer) == b'{' {
        return parse_object(item, input_buffer);
    }
    0
}

// ── parse_array ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn parse_array(
    item: *mut cJSON,
    input_buffer: *mut parse_buffer,
) -> c_int {
    let mut head: *mut cJSON = ptr::null_mut();
    let mut current_item: *mut cJSON = ptr::null_mut();

    if (*input_buffer).depth >= CJSON_NESTING_LIMIT {
        return 0;
    }
    (*input_buffer).depth += 1;

    if *pb_at_offset(input_buffer) != b'[' {
        return 0;
    }

    (*input_buffer).offset += 1;
    buffer_skip_whitespace(input_buffer);
    if pb_can_access(input_buffer, 0) && *pb_at_offset(input_buffer) == b']' {
        // empty array - success
        (*input_buffer).depth -= 1;
        (*item).type_ = CJSON_ARRAY;
        (*item).child = ptr::null_mut();
        (*input_buffer).offset += 1;
        return 1;
    }

    if !pb_can_access(input_buffer, 0) {
        (*input_buffer).offset -= 1;
        return 0;
    }

    (*input_buffer).offset -= 1;

    loop {
        let new_item = cjson_new_item(&(*input_buffer).hooks);
        if new_item.is_null() {
            if !head.is_null() { cJSON_Delete(head); }
            return 0;
        }
        if head.is_null() {
            head = new_item;
            current_item = new_item;
        } else {
            (*current_item).next = new_item;
            (*new_item).prev = current_item;
            current_item = new_item;
        }

        (*input_buffer).offset += 1;
        buffer_skip_whitespace(input_buffer);
        if parse_value(current_item, input_buffer) == 0 {
            if !head.is_null() { cJSON_Delete(head); }
            return 0;
        }
        buffer_skip_whitespace(input_buffer);

        if !pb_can_access(input_buffer, 0) || *pb_at_offset(input_buffer) != b',' {
            break;
        }
    }

    if !pb_can_access(input_buffer, 0) || *pb_at_offset(input_buffer) != b']' {
        if !head.is_null() { cJSON_Delete(head); }
        return 0;
    }

    (*input_buffer).depth -= 1;
    if !head.is_null() {
        (*head).prev = current_item;
    }
    (*item).type_ = CJSON_ARRAY;
    (*item).child = head;
    (*input_buffer).offset += 1;
    1
}

// ── parse_object ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn parse_object(
    item: *mut cJSON,
    input_buffer: *mut parse_buffer,
) -> c_int {
    let mut head: *mut cJSON = ptr::null_mut();
    let mut current_item: *mut cJSON = ptr::null_mut();

    if (*input_buffer).depth >= CJSON_NESTING_LIMIT {
        return 0;
    }
    (*input_buffer).depth += 1;

    if !pb_can_access(input_buffer, 0) || *pb_at_offset(input_buffer) != b'{' {
        return 0;
    }

    (*input_buffer).offset += 1;
    buffer_skip_whitespace(input_buffer);
    if pb_can_access(input_buffer, 0) && *pb_at_offset(input_buffer) == b'}' {
        // empty object
        (*input_buffer).depth -= 1;
        (*item).type_ = CJSON_OBJECT;
        (*item).child = ptr::null_mut();
        (*input_buffer).offset += 1;
        return 1;
    }

    if !pb_can_access(input_buffer, 0) {
        (*input_buffer).offset -= 1;
        return 0;
    }

    (*input_buffer).offset -= 1;

    loop {
        let new_item = cjson_new_item(&(*input_buffer).hooks);
        if new_item.is_null() {
            if !head.is_null() { cJSON_Delete(head); }
            return 0;
        }
        if head.is_null() {
            head = new_item;
            current_item = new_item;
        } else {
            (*current_item).next = new_item;
            (*new_item).prev = current_item;
            current_item = new_item;
        }

        if !pb_can_access(input_buffer, 1) {
            if !head.is_null() { cJSON_Delete(head); }
            return 0;
        }

        (*input_buffer).offset += 1;
        buffer_skip_whitespace(input_buffer);
        if parse_string(current_item, input_buffer) == 0 {
            if !head.is_null() { cJSON_Delete(head); }
            return 0;
        }
        buffer_skip_whitespace(input_buffer);

        // swap valuestring and string (we parsed the name)
        (*current_item).string = (*current_item).valuestring;
        (*current_item).valuestring = ptr::null_mut();

        if !pb_can_access(input_buffer, 0) || *pb_at_offset(input_buffer) != b':' {
            if !head.is_null() { cJSON_Delete(head); }
            return 0;
        }

        (*input_buffer).offset += 1;
        buffer_skip_whitespace(input_buffer);
        if parse_value(current_item, input_buffer) == 0 {
            if !head.is_null() { cJSON_Delete(head); }
            return 0;
        }
        buffer_skip_whitespace(input_buffer);

        if !pb_can_access(input_buffer, 0) || *pb_at_offset(input_buffer) != b',' {
            break;
        }
    }

    if !pb_can_access(input_buffer, 0) || *pb_at_offset(input_buffer) != b'}' {
        if !head.is_null() { cJSON_Delete(head); }
        return 0;
    }

    (*input_buffer).depth -= 1;
    if !head.is_null() {
        (*head).prev = current_item;
    }
    (*item).type_ = CJSON_OBJECT;
    (*item).child = head;
    (*input_buffer).offset += 1;
    1
}

// ── print_number ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn print_number(
    item: *const cJSON,
    output_buffer: *mut printbuffer,
) -> c_int {
    if output_buffer.is_null() {
        return 0;
    }
    let d = (*item).valuedouble;
    let mut number_buffer = [0u8; 26];
    let mut length: c_int;

    if d.is_nan() || d.is_infinite() {
        length = sprintf(
            number_buffer.as_mut_ptr() as *mut c_char,
            b"null\0".as_ptr() as *const c_char,
        );
    } else if d == (*item).valueint as f64 {
        length = sprintf(
            number_buffer.as_mut_ptr() as *mut c_char,
            b"%d\0".as_ptr() as *const c_char,
            (*item).valueint,
        );
    } else {
        length = sprintf(
            number_buffer.as_mut_ptr() as *mut c_char,
            b"%1.15g\0".as_ptr() as *const c_char,
            d,
        );
        let mut test: c_double = 0.0;
        if sscanf(
            number_buffer.as_ptr() as *const c_char,
            b"%lg\0".as_ptr() as *const c_char,
            &mut test as *mut c_double,
        ) != 1 || compare_double(test, d) == 0
        {
            length = sprintf(
                number_buffer.as_mut_ptr() as *mut c_char,
                b"%1.17g\0".as_ptr() as *const c_char,
                d,
            );
        }
    }

    if length < 0 || length > (number_buffer.len() as c_int - 1) {
        return 0;
    }

    let output_pointer = ensure(output_buffer, length as usize + 1);
    if output_pointer.is_null() {
        return 0;
    }

    for i in 0..(length as usize) {
        if number_buffer[i] == b'.' {
            *output_pointer.add(i) = b'.';
        } else {
            *output_pointer.add(i) = number_buffer[i];
        }
    }
    *output_pointer.add(length as usize) = 0;
    (*output_buffer).offset += length as usize;
    1
}

// ── print_string_ptr ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn print_string_ptr(
    input: *const u8,
    output_buffer: *mut printbuffer,
) -> c_int {
    if output_buffer.is_null() {
        return 0;
    }
    if input.is_null() {
        let output = ensure(output_buffer, 3);
        if output.is_null() { return 0; }
        strcpy(output as *mut c_char, b"\"\"\0".as_ptr() as *const c_char);
        return 1;
    }

    let mut escape_characters: usize = 0;
    let mut input_pointer = input;
    while *input_pointer != 0 {
        match *input_pointer {
            b'\"' | b'\\' | 0x08 | 0x0C | b'\n' | b'\r' | b'\t' => {
                escape_characters += 1;
            }
            c if c < 32 => {
                escape_characters += 5; // \uXXXX
            }
            _ => {}
        }
        input_pointer = input_pointer.add(1);
    }
    let output_length = (input_pointer as usize) - (input as usize) + escape_characters;

    let output = ensure(output_buffer, output_length + 3);
    if output.is_null() {
        return 0;
    }

    if escape_characters == 0 {
        *output = b'\"';
        memcpy(
            output.add(1) as *mut c_void,
            input as *const c_void,
            output_length,
        );
        *output.add(output_length + 1) = b'\"';
        *output.add(output_length + 2) = 0;
        return 1;
    }

    *output = b'\"';
    let mut op = output.add(1);
    input_pointer = input;
    while *input_pointer != 0 {
        if *input_pointer > 31 && *input_pointer != b'\"' && *input_pointer != b'\\' {
            *op = *input_pointer;
        } else {
            *op = b'\\';
            op = op.add(1);
            match *input_pointer {
                b'\\' => { *op = b'\\'; }
                b'\"' => { *op = b'\"'; }
                0x08 => { *op = b'b'; }
                0x0C => { *op = b'f'; }
                b'\n' => { *op = b'n'; }
                b'\r' => { *op = b'r'; }
                b'\t' => { *op = b't'; }
                _ => {
                    sprintf(
                        op as *mut c_char,
                        b"u%04x\0".as_ptr() as *const c_char,
                        *input_pointer as c_int,
                    );
                    op = op.add(4);
                }
            }
        }
        input_pointer = input_pointer.add(1);
        op = op.add(1);
    }
    *output.add(output_length + 1) = b'\"';
    *output.add(output_length + 2) = 0;
    1
}

// ── print_value ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn print_value(
    item: *const cJSON,
    output_buffer: *mut printbuffer,
) -> c_int {
    if item.is_null() || output_buffer.is_null() {
        return 0;
    }
    match (*item).type_ & 0xFF {
        x if x == CJSON_NULL => {
            let output = ensure(output_buffer, 5);
            if output.is_null() { return 0; }
            strcpy(output as *mut c_char, b"null\0".as_ptr() as *const c_char);
            1
        }
        x if x == CJSON_FALSE => {
            let output = ensure(output_buffer, 6);
            if output.is_null() { return 0; }
            strcpy(output as *mut c_char, b"false\0".as_ptr() as *const c_char);
            1
        }
        x if x == CJSON_TRUE => {
            let output = ensure(output_buffer, 5);
            if output.is_null() { return 0; }
            strcpy(output as *mut c_char, b"true\0".as_ptr() as *const c_char);
            1
        }
        x if x == CJSON_NUMBER => {
            print_number(item, output_buffer)
        }
        x if x == CJSON_RAW => {
            if (*item).valuestring.is_null() { return 0; }
            let raw_length = strlen((*item).valuestring) + 1;
            let output = ensure(output_buffer, raw_length);
            if output.is_null() { return 0; }
            memcpy(
                output as *mut c_void,
                (*item).valuestring as *const c_void,
                raw_length,
            );
            1
        }
        x if x == CJSON_STRING => {
            print_string_ptr((*item).valuestring as *const u8, output_buffer)
        }
        x if x == CJSON_ARRAY => {
            print_array(item, output_buffer)
        }
        x if x == CJSON_OBJECT => {
            print_object(item, output_buffer)
        }
        _ => 0,
    }
}

// ── print_array ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn print_array(
    item: *const cJSON,
    output_buffer: *mut printbuffer,
) -> c_int {
    if output_buffer.is_null() {
        return 0;
    }
    let mut output_pointer = ensure(output_buffer, 1);
    if output_pointer.is_null() { return 0; }
    *output_pointer = b'[';
    (*output_buffer).offset += 1;
    (*output_buffer).depth += 1;

    let mut current_element = (*item).child;
    while !current_element.is_null() {
        if print_value(current_element, output_buffer) == 0 {
            return 0;
        }
        update_offset(output_buffer);
        if !(*current_element).next.is_null() {
            let length = if (*output_buffer).format != 0 { 2usize } else { 1usize };
            output_pointer = ensure(output_buffer, length + 1);
            if output_pointer.is_null() { return 0; }
            *output_pointer = b',';
            if (*output_buffer).format != 0 {
                *output_pointer.add(1) = b' ';
            }
            *output_pointer.add(length) = 0;
            (*output_buffer).offset += length;
        }
        current_element = (*current_element).next;
    }

    output_pointer = ensure(output_buffer, 2);
    if output_pointer.is_null() { return 0; }
    *output_pointer = b']';
    *output_pointer.add(1) = 0;
    (*output_buffer).depth -= 1;
    1
}

// ── print_object ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn print_object(
    item: *const cJSON,
    output_buffer: *mut printbuffer,
) -> c_int {
    if output_buffer.is_null() {
        return 0;
    }

    let length = if (*output_buffer).format != 0 { 2usize } else { 1usize };
    let mut output_pointer = ensure(output_buffer, length + 1);
    if output_pointer.is_null() { return 0; }
    *output_pointer = b'{';
    (*output_buffer).depth += 1;
    if (*output_buffer).format != 0 {
        *output_pointer.add(1) = b'\n';
    }
    (*output_buffer).offset += length;

    let mut current_item = (*item).child;
    while !current_item.is_null() {
        if (*output_buffer).format != 0 {
            let indent = (*output_buffer).depth;
            output_pointer = ensure(output_buffer, indent);
            if output_pointer.is_null() { return 0; }
            for i in 0..indent {
                *output_pointer.add(i) = b'\t';
            }
            (*output_buffer).offset += indent;
        }

        // print key
        if print_string_ptr((*current_item).string as *const u8, output_buffer) == 0 {
            return 0;
        }
        update_offset(output_buffer);

        let sep_length = if (*output_buffer).format != 0 { 2usize } else { 1usize };
        output_pointer = ensure(output_buffer, sep_length);
        if output_pointer.is_null() { return 0; }
        *output_pointer = b':';
        if (*output_buffer).format != 0 {
            *output_pointer.add(1) = b'\t';
        }
        (*output_buffer).offset += sep_length;

        // print value
        if print_value(current_item, output_buffer) == 0 {
            return 0;
        }
        update_offset(output_buffer);

        // print comma if not last
        let comma_len = (if (*output_buffer).format != 0 { 1usize } else { 0 })
            + (if !(*current_item).next.is_null() { 1usize } else { 0 });
        output_pointer = ensure(output_buffer, comma_len + 1);
        if output_pointer.is_null() { return 0; }
        if !(*current_item).next.is_null() {
            *output_pointer = b',';
            output_pointer = output_pointer.add(1);
        }
        if (*output_buffer).format != 0 {
            *output_pointer = b'\n';
            output_pointer = output_pointer.add(1);
        }
        *output_pointer = 0;
        (*output_buffer).offset += comma_len;

        current_item = (*current_item).next;
    }

    let closing_len = if (*output_buffer).format != 0 {
        (*output_buffer).depth + 1
    } else {
        2
    };
    output_pointer = ensure(output_buffer, closing_len);
    if output_pointer.is_null() { return 0; }
    if (*output_buffer).format != 0 {
        for i in 0..((*output_buffer).depth - 1) {
            *output_pointer.add(i) = b'\t';
        }
        output_pointer = output_pointer.add((*output_buffer).depth - 1);
    }
    *output_pointer = b'}';
    *output_pointer.add(1) = 0;
    (*output_buffer).depth -= 1;
    1
}