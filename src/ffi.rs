//! FFI layer – pure glue between LX-json (Rust) and the cJSON C ABI.
//!
//! Design principles:
//!   • JSON algorithms (parse, print, compare, duplicate, minify) delegate
//!     to LX-json modules – their results reflect LX-json's code quality.
//!   • C-struct operations (linked-list management, type predicates, value
//!     getters/setters, memory) are implemented here because they manage
//!     the cJSON data structure which has no LX-json equivalent.
//!   • Internal cJSON functions that operate on parse_buffer / printbuffer
//!     are exported as stubs – LX-json uses a different architecture.

// ── Original LX-json modules (unchanged) ────────────────────────────────────
pub mod error;
pub mod merge;
pub mod parser;
pub mod patch;
pub mod query;
pub mod serializer;
pub mod types;
pub mod utils;

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

// ═══════════════════════════════════════════════════════════════════════════════
//  §1  Constants & C-compatible types
// ═══════════════════════════════════════════════════════════════════════════════

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
    pub malloc_fn: Option<unsafe extern "C" fn(usize) -> *mut c_void>,
    pub free_fn: Option<unsafe extern "C" fn(*mut c_void)>,
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

// ═══════════════════════════════════════════════════════════════════════════════
//  §2  libc helpers & memory management
// ═══════════════════════════════════════════════════════════════════════════════

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
}

thread_local! {
    static HOOKS_MALLOC: RefCell<Option<unsafe extern "C" fn(usize) -> *mut c_void>> =
        RefCell::new(None);
    static HOOKS_FREE: RefCell<Option<unsafe extern "C" fn(*mut c_void)>> =
        RefCell::new(None);
    static ERROR_PTR: RefCell<*const c_char> = RefCell::new(ptr::null());
}

unsafe fn internal_malloc(size: usize) -> *mut c_void {
    HOOKS_MALLOC.with(|h| match *h.borrow() {
        Some(f) => f(size),
        None => malloc(size),
    })
}

unsafe fn internal_free(ptr: *mut c_void) {
    HOOKS_FREE.with(|h| match *h.borrow() {
        Some(f) => f(ptr),
        None => free(ptr),
    })
}

unsafe extern "C" fn default_malloc_fn(size: usize) -> *mut c_void { malloc(size) }
unsafe extern "C" fn default_free_fn(ptr: *mut c_void) { free(ptr) }
unsafe extern "C" fn default_realloc_fn(p: *mut c_void, s: usize) -> *mut c_void { realloc(p, s) }

static mut GLOBAL_HOOKS: internal_hooks = internal_hooks {
    allocate: Some(default_malloc_fn),
    deallocate: Some(default_free_fn),
    reallocate: Some(default_realloc_fn),
};

unsafe fn hooks_allocate(hooks: &internal_hooks, size: usize) -> *mut c_void {
    match hooks.allocate { Some(f) => f(size), None => malloc(size) }
}

unsafe fn hooks_deallocate(hooks: &internal_hooks, ptr: *mut c_void) {
    match hooks.deallocate { Some(f) => f(ptr), None => free(ptr) }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  §3  Internal helpers  (C-struct utilities, not JSON logic)
// ═══════════════════════════════════════════════════════════════════════════════

/// Duplicate a C string using the current allocator.
unsafe fn cstr_dup(s: *const c_char) -> *mut c_char {
    if s.is_null() { return ptr::null_mut(); }
    let len = strlen(s);
    let p = internal_malloc(len + 1) as *mut c_char;
    if !p.is_null() { memcpy(p as *mut c_void, s as *const c_void, len + 1); }
    p
}

/// Allocate a C string from a Rust `&str` using the current allocator.
unsafe fn cstr_from_rust(s: &str) -> *mut c_char {
    let bytes = s.as_bytes();
    let p = internal_malloc(bytes.len() + 1) as *mut c_char;
    if p.is_null() { return ptr::null_mut(); }
    ptr::copy_nonoverlapping(bytes.as_ptr(), p as *mut u8, bytes.len());
    *p.add(bytes.len()) = 0;
    p
}

/// Allocate and zero-initialise a `cJSON` node.
unsafe fn new_item() -> *mut cJSON {
    let p = internal_malloc(std::mem::size_of::<cJSON>()) as *mut cJSON;
    if !p.is_null() { ptr::write_bytes(p, 0, 1); }
    p
}

/// Link `item` after `prev` in a sibling chain.
unsafe fn suffix_object(prev: *mut cJSON, item: *mut cJSON) {
    (*prev).next = item;
    (*item).prev = prev;
}

fn saturating_double_to_int(n: f64) -> c_int {
    if n >= i32::MAX as f64 { i32::MAX }
    else if n <= i32::MIN as f64 { i32::MIN }
    else { n as c_int }
}

/// Strip UTF-8 BOM from a Rust &str before passing to LX-json parser.
fn strip_utf8_bom(s: &str) -> &str {
    if s.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) { &s[3..] } else { s }
}

/// Extract byte position from a `JsonError` for C error pointer reporting.
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

// ═══════════════════════════════════════════════════════════════════════════════
//  §4  JsonNode ↔ cJSON conversion
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert a Rust `JsonNode` tree into a heap-allocated `cJSON` linked-list.
unsafe fn jsonnode_to_cjson(node: &JsonNode) -> *mut cJSON {
    match node {
        JsonNode::Null => {
            let i = new_item();
            if !i.is_null() { (*i).type_ = CJSON_NULL; }
            i
        }
        JsonNode::Bool(true) => {
            let i = new_item();
            if !i.is_null() { (*i).type_ = CJSON_TRUE; }
            i
        }
        JsonNode::Bool(false) => {
            let i = new_item();
            if !i.is_null() { (*i).type_ = CJSON_FALSE; }
            i
        }
        JsonNode::Number(n) => {
            let i = new_item();
            if !i.is_null() {
                (*i).type_ = CJSON_NUMBER;
                (*i).valuedouble = *n;
                (*i).valueint = saturating_double_to_int(*n);
            }
            i
        }
        JsonNode::String(s) => {
            let i = new_item();
            if !i.is_null() {
                (*i).type_ = CJSON_STRING;
                (*i).valuestring = cstr_from_rust(s);
            }
            i
        }
        JsonNode::Raw(s) => {
            let i = new_item();
            if !i.is_null() {
                (*i).type_ = CJSON_RAW;
                (*i).valuestring = cstr_from_rust(s);
            }
            i
        }
        JsonNode::Array(arr) => {
            let i = new_item();
            if i.is_null() { return ptr::null_mut(); }
            (*i).type_ = CJSON_ARRAY;
            let mut prev_child: *mut cJSON = ptr::null_mut();
            for (idx, child_node) in arr.iter().enumerate() {
                let child = jsonnode_to_cjson(child_node);
                if child.is_null() { continue; }
                if idx == 0 { (*i).child = child; } else { suffix_object(prev_child, child); }
                prev_child = child;
            }
            // cJSON tail-pointer convention: head->prev = last child
            if !(*i).child.is_null() && !prev_child.is_null() {
                (*(*i).child).prev = prev_child;
            }
            i
        }
        JsonNode::Object(pairs) => {
            let i = new_item();
            if i.is_null() { return ptr::null_mut(); }
            (*i).type_ = CJSON_OBJECT;
            let mut prev_child: *mut cJSON = ptr::null_mut();
            for (idx, (key, val)) in pairs.iter().enumerate() {
                let child = jsonnode_to_cjson(val);
                if child.is_null() { continue; }
                (*child).string = cstr_from_rust(key);
                if idx == 0 { (*i).child = child; } else { suffix_object(prev_child, child); }
                prev_child = child;
            }
            if !(*i).child.is_null() && !prev_child.is_null() {
                (*(*i).child).prev = prev_child;
            }
            i
        }
    }
}

/// Convert a `cJSON` linked-list back into a Rust `JsonNode`.
unsafe fn cjson_to_jsonnode(item: *const cJSON) -> JsonNode {
    let mut visited = std::collections::HashSet::new();
    cjson_to_jsonnode_impl(item, &mut visited).unwrap_or(JsonNode::Null)
}

/// Same but returns `None` on circular reference (used by cJSON_Duplicate).
unsafe fn cjson_to_jsonnode_checked(item: *const cJSON) -> Option<JsonNode> {
    let mut visited = std::collections::HashSet::new();
    cjson_to_jsonnode_impl(item, &mut visited)
}

unsafe fn cjson_to_jsonnode_impl(
    item: *const cJSON,
    visited: &mut std::collections::HashSet<usize>,
) -> Option<JsonNode> {
    if item.is_null() { return Some(JsonNode::Null); }
    let addr = item as usize;
    if !visited.insert(addr) { return None; } // circular reference
    let t = (*item).type_ & 0xFF;
    let result = match t {
        x if x == CJSON_FALSE => Some(JsonNode::Bool(false)),
        x if x == CJSON_TRUE => Some(JsonNode::Bool(true)),
        x if x == CJSON_NULL => Some(JsonNode::Null),
        x if x == CJSON_NUMBER => Some(JsonNode::Number((*item).valuedouble)),
        x if x == CJSON_STRING => Some(JsonNode::String(
            if (*item).valuestring.is_null() { String::new() }
            else { CStr::from_ptr((*item).valuestring).to_string_lossy().into_owned() }
        )),
        x if x == CJSON_RAW => Some(JsonNode::Raw(
            if (*item).valuestring.is_null() { String::new() }
            else { CStr::from_ptr((*item).valuestring).to_string_lossy().into_owned() }
        )),
        x if x == CJSON_ARRAY => {
            let mut v = Vec::new();
            let mut c = (*item).child;
            while !c.is_null() {
                match cjson_to_jsonnode_impl(c, visited) {
                    Some(n) => v.push(n),
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
                let key = if (*c).string.is_null() { String::new() }
                          else { CStr::from_ptr((*c).string).to_string_lossy().into_owned() };
                match cjson_to_jsonnode_impl(c, visited) {
                    Some(n) => pairs.push((key, n)),
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

// ═══════════════════════════════════════════════════════════════════════════════
//  §5  Exported C API
// ═══════════════════════════════════════════════════════════════════════════════

// ── 5.1  Version ─────────────────────────────────────────────────────────────

static VERSION_CSTR: &[u8] = b"1.7.19\0";

#[no_mangle]
pub unsafe extern "C" fn cJSON_Version() -> *const c_char {
    VERSION_CSTR.as_ptr() as *const c_char
}

// ── 5.2  Hooks ───────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_InitHooks(hooks: *mut cJSON_Hooks) {
    if hooks.is_null() {
        HOOKS_MALLOC.with(|h| *h.borrow_mut() = None);
        HOOKS_FREE.with(|h| *h.borrow_mut() = None);
        GLOBAL_HOOKS = internal_hooks {
            allocate: Some(default_malloc_fn),
            deallocate: Some(default_free_fn),
            reallocate: Some(default_realloc_fn),
        };
        return;
    }
    HOOKS_MALLOC.with(|h| *h.borrow_mut() = (*hooks).malloc_fn);
    HOOKS_FREE.with(|h| *h.borrow_mut() = (*hooks).free_fn);
    GLOBAL_HOOKS.allocate = (*hooks).malloc_fn.or(Some(default_malloc_fn));
    GLOBAL_HOOKS.deallocate = (*hooks).free_fn.or(Some(default_free_fn));
    GLOBAL_HOOKS.reallocate = if (*hooks).malloc_fn.is_none() && (*hooks).free_fn.is_none() {
        Some(default_realloc_fn)
    } else {
        None
    };
}

// ── 5.3  Parsing  (→ LX-json parser) ────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Parse(value: *const c_char) -> *mut cJSON {
    if value.is_null() { return ptr::null_mut(); }
    let json_str = match CStr::from_ptr(value).to_str() {
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
    if value.is_null() { return ptr::null_mut(); }
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
    if value.is_null() { return ptr::null_mut(); }
    let raw_str = match CStr::from_ptr(value).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let bom_offset = if raw_str.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) { 3usize } else { 0 };
    let json_str = &raw_str[bom_offset..];
    let opts = parser::ParseOptions {
        nesting_limit: 1000,
        require_null_terminated: require_null_terminated != 0,
    };
    match parser::parse_with_opts(json_str, opts) {
        Ok(node) => {
            if !return_parse_end.is_null() {
                *return_parse_end = value.add(bom_offset + json_str.len());
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
    if value.is_null() || buffer_length == 0 { return ptr::null_mut(); }
    let slice = std::slice::from_raw_parts(value as *const u8, buffer_length);
    let json_str = match std::str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let bom_offset = if json_str.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) { 3usize } else { 0 };
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

// ── 5.4  Printing  (→ LX-json serializer) ───────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Print(item: *const cJSON) -> *mut c_char {
    if item.is_null() { return ptr::null_mut(); }
    let node = cjson_to_jsonnode(item);
    cstr_from_rust(&serializer::print(&node))
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_PrintUnformatted(item: *const cJSON) -> *mut c_char {
    if item.is_null() { return ptr::null_mut(); }
    let node = cjson_to_jsonnode(item);
    cstr_from_rust(&serializer::print_unformatted(&node))
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_PrintBuffered(
    item: *const cJSON,
    prebuffer: c_int,
    fmt: c_int,
) -> *mut c_char {
    if item.is_null() { return ptr::null_mut(); }
    let node = cjson_to_jsonnode(item);
    cstr_from_rust(&serializer::print_buffered(&node, prebuffer.max(0) as usize, fmt != 0))
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_PrintPreallocated(
    item: *mut cJSON,
    buffer: *mut c_char,
    length: c_int,
    format: c_int,
) -> c_int {
    if item.is_null() || buffer.is_null() || length <= 0 { return 0; }
    let node = cjson_to_jsonnode(item);
    let output = if format != 0 {
        serializer::print(&node)
    } else {
        serializer::print_unformatted(&node)
    };
    let bytes = output.as_bytes();
    if bytes.len() >= length as usize { return 0; }
    ptr::copy_nonoverlapping(bytes.as_ptr(), buffer as *mut u8, bytes.len());
    *buffer.add(bytes.len()) = 0;
    1
}

// ── 5.5  Delete  (C-struct cleanup) ──────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Delete(item: *mut cJSON) {
    let mut current = item;
    while !current.is_null() {
        let next = (*current).next;
        if (*current).type_ & CJSON_IS_REFERENCE == 0 && !(*current).child.is_null() {
            cJSON_Delete((*current).child);
        }
        if (*current).type_ & CJSON_IS_REFERENCE == 0 && !(*current).valuestring.is_null() {
            internal_free((*current).valuestring as *mut c_void);
        }
        if (*current).type_ & CJSON_STRING_IS_CONST == 0 && !(*current).string.is_null() {
            internal_free((*current).string as *mut c_void);
        }
        internal_free(current as *mut c_void);
        current = next;
    }
}

// ── 5.6  Accessors  (C-struct operations) ────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetArraySize(array: *const cJSON) -> c_int {
    if array.is_null() { return 0; }
    let mut count: c_int = 0;
    let mut c = (*array).child;
    while !c.is_null() { count += 1; c = (*c).next; }
    count
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetArrayItem(array: *const cJSON, index: c_int) -> *mut cJSON {
    if array.is_null() || index < 0 { return ptr::null_mut(); }
    let mut c = (*array).child;
    let mut i: c_int = 0;
    while !c.is_null() {
        if i == index { return c; }
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
    if object.is_null() || string.is_null() { return ptr::null_mut(); }
    let key = match CStr::from_ptr(string).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut c = (*object).child;
    while !c.is_null() {
        if !(*c).string.is_null() {
            if let Ok(k) = CStr::from_ptr((*c).string).to_str() {
                if k.eq_ignore_ascii_case(key) { return c; }
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
    if object.is_null() || string.is_null() { return ptr::null_mut(); }
    let key = match CStr::from_ptr(string).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut c = (*object).child;
    while !c.is_null() {
        if !(*c).string.is_null() {
            if let Ok(k) = CStr::from_ptr((*c).string).to_str() {
                if k == key { return c; }
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
    if cJSON_GetObjectItem(object, string).is_null() { 0 } else { 1 }
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetErrorPtr() -> *const c_char {
    ERROR_PTR.with(|p| *p.borrow())
}

// ── 5.7  Value getters ───────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetStringValue(item: *const cJSON) -> *mut c_char {
    if item.is_null() || ((*item).type_ & 0xFF) != CJSON_STRING { return ptr::null_mut(); }
    (*item).valuestring
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_GetNumberValue(item: *const cJSON) -> c_double {
    if item.is_null() || ((*item).type_ & 0xFF) != CJSON_NUMBER { return f64::NAN; }
    (*item).valuedouble
}

// ── 5.8  Type predicates ─────────────────────────────────────────────────────

macro_rules! type_pred {
    ($name:ident, $($pat:expr),+) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(item: *const cJSON) -> c_int {
            if item.is_null() { return 0; }
            let t = (*item).type_ & 0xFF;
            if $( t == $pat )||+ { 1 } else { 0 }
        }
    };
}

type_pred!(cJSON_IsInvalid, CJSON_INVALID);
type_pred!(cJSON_IsFalse,   CJSON_FALSE);
type_pred!(cJSON_IsTrue,    CJSON_TRUE);
type_pred!(cJSON_IsBool,    CJSON_FALSE, CJSON_TRUE);
type_pred!(cJSON_IsNull,    CJSON_NULL);
type_pred!(cJSON_IsNumber,  CJSON_NUMBER);
type_pred!(cJSON_IsString,  CJSON_STRING);
type_pred!(cJSON_IsArray,   CJSON_ARRAY);
type_pred!(cJSON_IsObject,  CJSON_OBJECT);
type_pred!(cJSON_IsRaw,     CJSON_RAW);

// ── 5.9  Creation  (C-struct init) ───────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateNull() -> *mut cJSON {
    let i = new_item(); if !i.is_null() { (*i).type_ = CJSON_NULL; } i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateTrue() -> *mut cJSON {
    let i = new_item(); if !i.is_null() { (*i).type_ = CJSON_TRUE; } i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateFalse() -> *mut cJSON {
    let i = new_item(); if !i.is_null() { (*i).type_ = CJSON_FALSE; } i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateBool(b: c_int) -> *mut cJSON {
    let i = new_item();
    if !i.is_null() { (*i).type_ = if b != 0 { CJSON_TRUE } else { CJSON_FALSE }; }
    i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateNumber(num: c_double) -> *mut cJSON {
    let i = new_item();
    if !i.is_null() {
        (*i).type_ = CJSON_NUMBER;
        (*i).valuedouble = num;
        (*i).valueint = saturating_double_to_int(num);
    }
    i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateString(string: *const c_char) -> *mut cJSON {
    if string.is_null() { return ptr::null_mut(); }
    let i = new_item();
    if i.is_null() { return ptr::null_mut(); }
    (*i).type_ = CJSON_STRING;
    (*i).valuestring = cstr_dup(string);
    if (*i).valuestring.is_null() { internal_free(i as *mut c_void); return ptr::null_mut(); }
    i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateRaw(raw: *const c_char) -> *mut cJSON {
    if raw.is_null() { return ptr::null_mut(); }
    let i = new_item();
    if i.is_null() { return ptr::null_mut(); }
    (*i).type_ = CJSON_RAW;
    (*i).valuestring = cstr_dup(raw);
    if (*i).valuestring.is_null() { internal_free(i as *mut c_void); return ptr::null_mut(); }
    i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateArray() -> *mut cJSON {
    let i = new_item(); if !i.is_null() { (*i).type_ = CJSON_ARRAY; } i
}
#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateObject() -> *mut cJSON {
    let i = new_item(); if !i.is_null() { (*i).type_ = CJSON_OBJECT; } i
}

// ── 5.10  Reference creation ─────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateStringReference(string: *const c_char) -> *mut cJSON {
    let i = new_item();
    if i.is_null() { return ptr::null_mut(); }
    (*i).type_ = CJSON_STRING | CJSON_IS_REFERENCE;
    (*i).valuestring = string as *mut c_char;
    i
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateObjectReference(child: *const cJSON) -> *mut cJSON {
    if child.is_null() { return ptr::null_mut(); }
    let i = new_item();
    if i.is_null() { return ptr::null_mut(); }
    (*i).type_ = CJSON_OBJECT | CJSON_IS_REFERENCE;
    (*i).child = child as *mut cJSON;
    i
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateArrayReference(child: *const cJSON) -> *mut cJSON {
    if child.is_null() { return ptr::null_mut(); }
    let i = new_item();
    if i.is_null() { return ptr::null_mut(); }
    (*i).type_ = CJSON_ARRAY | CJSON_IS_REFERENCE;
    (*i).child = child as *mut cJSON;
    i
}

// ── 5.11  Array creation utilities ───────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateIntArray(numbers: *const c_int, count: c_int) -> *mut cJSON {
    if numbers.is_null() || count <= 0 { return ptr::null_mut(); }
    let arr = cJSON_CreateArray();
    if arr.is_null() { return ptr::null_mut(); }
    for idx in 0..count as usize {
        let n = cJSON_CreateNumber(*numbers.add(idx) as c_double);
        if n.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if idx == 0 { (*arr).child = n; (*n).prev = n; }
        else {
            let tail = (*(*arr).child).prev;
            suffix_object(tail, n);
            (*(*arr).child).prev = n;
        }
    }
    arr
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateFloatArray(numbers: *const f32, count: c_int) -> *mut cJSON {
    if numbers.is_null() || count <= 0 { return ptr::null_mut(); }
    let arr = cJSON_CreateArray();
    if arr.is_null() { return ptr::null_mut(); }
    for idx in 0..count as usize {
        let n = cJSON_CreateNumber(*numbers.add(idx) as c_double);
        if n.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if idx == 0 { (*arr).child = n; (*n).prev = n; }
        else {
            let tail = (*(*arr).child).prev;
            suffix_object(tail, n);
            (*(*arr).child).prev = n;
        }
    }
    arr
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_CreateDoubleArray(numbers: *const c_double, count: c_int) -> *mut cJSON {
    if numbers.is_null() || count <= 0 { return ptr::null_mut(); }
    let arr = cJSON_CreateArray();
    if arr.is_null() { return ptr::null_mut(); }
    for idx in 0..count as usize {
        let n = cJSON_CreateNumber(*numbers.add(idx));
        if n.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if idx == 0 { (*arr).child = n; (*n).prev = n; }
        else {
            let tail = (*(*arr).child).prev;
            suffix_object(tail, n);
            (*(*arr).child).prev = n;
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
    for idx in 0..count as usize {
        let s = cJSON_CreateString(*strings.add(idx));
        if s.is_null() { cJSON_Delete(arr); return ptr::null_mut(); }
        if idx == 0 { (*arr).child = s; (*s).prev = s; }
        else {
            let tail = (*(*arr).child).prev;
            suffix_object(tail, s);
            (*(*arr).child).prev = s;
        }
    }
    arr
}

// ── 5.12  Add / Detach / Delete / Insert / Replace  (linked-list ops) ───────

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
    if object.is_null() || string.is_null() || item.is_null() { return 0; }
    if (*item).type_ & CJSON_STRING_IS_CONST == 0 && !(*item).string.is_null() {
        internal_free((*item).string as *mut c_void);
    }
    (*item).string = cstr_dup(string);
    (*item).type_ &= !CJSON_STRING_IS_CONST;
    add_item_to_array(object, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemToObjectCS(
    object: *mut cJSON,
    string: *const c_char,
    item: *mut cJSON,
) -> c_int {
    if object.is_null() || string.is_null() || item.is_null() { return 0; }
    if (*item).type_ & CJSON_STRING_IS_CONST == 0 && !(*item).string.is_null() {
        internal_free((*item).string as *mut c_void);
    }
    (*item).string = string as *mut c_char;
    (*item).type_ |= CJSON_STRING_IS_CONST;
    add_item_to_array(object, item)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemReferenceToArray(
    array: *mut cJSON,
    item: *mut cJSON,
) -> c_int {
    if array.is_null() || item.is_null() { return 0; }
    let r = new_item();
    if r.is_null() { return 0; }
    ptr::copy_nonoverlapping(item, r, 1);
    (*r).type_ |= CJSON_IS_REFERENCE;
    (*r).prev = ptr::null_mut();
    (*r).next = ptr::null_mut();
    (*r).string = ptr::null_mut();
    add_item_to_array(array, r)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_AddItemReferenceToObject(
    object: *mut cJSON,
    string: *const c_char,
    item: *mut cJSON,
) -> c_int {
    if object.is_null() || string.is_null() || item.is_null() { return 0; }
    let r = new_item();
    if r.is_null() { return 0; }
    ptr::copy_nonoverlapping(item, r, 1);
    (*r).type_ |= CJSON_IS_REFERENCE;
    (*r).prev = ptr::null_mut();
    (*r).next = ptr::null_mut();
    (*r).string = cstr_dup(string);
    add_item_to_array(object, r)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_DetachItemViaPointer(
    parent: *mut cJSON,
    item: *mut cJSON,
) -> *mut cJSON {
    if parent.is_null() || item.is_null() { return ptr::null_mut(); }
    if item != (*parent).child && (*item).prev.is_null() { return ptr::null_mut(); }
    if item != (*parent).child {
        (*(*item).prev).next = (*item).next;
    }
    if !(*item).next.is_null() {
        (*(*item).next).prev = (*item).prev;
    }
    if item == (*parent).child {
        (*parent).child = (*item).next;
    } else if (*item).next.is_null() {
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

#[no_mangle]
pub unsafe extern "C" fn cJSON_InsertItemInArray(
    array: *mut cJSON,
    which: c_int,
    newitem: *mut cJSON,
) -> c_int {
    if array.is_null() || newitem.is_null() || which < 0 { return 0; }
    let after = cJSON_GetArrayItem(array, which);
    if after.is_null() { return add_item_to_array(array, newitem); }
    if after != (*array).child && (*after).prev.is_null() { return 0; }
    (*newitem).next = after;
    (*newitem).prev = (*after).prev;
    (*after).prev = newitem;
    if after == (*array).child {
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

// ── 5.13  Duplicate  (→ LX-json utils::duplicate) ───────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Duplicate(item: *const cJSON, recurse: c_int) -> *mut cJSON {
    if item.is_null() { return ptr::null_mut(); }
    let node = match cjson_to_jsonnode_checked(item) {
        Some(n) => n,
        None => return ptr::null_mut(), // circular reference
    };
    let dup_node = utils::duplicate(&node, recurse != 0);
    let dup = jsonnode_to_cjson(&dup_node);
    if !dup.is_null() && !(*item).string.is_null() {
        (*dup).string = cstr_dup((*item).string);
    }
    dup
}

// ── 5.14  Compare  (→ LX-json utils::compare) ───────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Compare(
    a: *const cJSON,
    b: *const cJSON,
    case_sensitive: c_int,
) -> c_int {
    if a.is_null() || b.is_null() { return 0; }
    // Reject INVALID (0) or composite types (multiple bits set) – these have
    // no JsonNode equivalent and cJSON considers them always not-equal.
    let ta = (*a).type_ & 0xFF;
    let tb = (*b).type_ & 0xFF;
    if ta == 0 || (ta & (ta - 1)) != 0 || ta > CJSON_RAW { return 0; }
    if tb == 0 || (tb & (tb - 1)) != 0 || tb > CJSON_RAW { return 0; }
    let node_a = cjson_to_jsonnode(a);
    let node_b = cjson_to_jsonnode(b);
    if utils::compare(&node_a, &node_b, case_sensitive != 0) { 1 } else { 0 }
}

// ── 5.15  Minify  (→ LX-json serializer::minify) ────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_Minify(json: *mut c_char) {
    if json.is_null() { return; }
    let json_str = match CStr::from_ptr(json).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    if let Ok(minified) = serializer::minify(json_str) {
        let bytes = minified.as_bytes();
        ptr::copy_nonoverlapping(bytes.as_ptr(), json as *mut u8, bytes.len());
        *json.add(bytes.len()) = 0;
    }
}

// ── 5.16  Convenience add-to-object helpers ──────────────────────────────────

macro_rules! add_to_object {
    ($name:ident, $create:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            object: *mut cJSON,
            name: *const c_char,
        ) -> *mut cJSON {
            if object.is_null() || name.is_null() { return ptr::null_mut(); }
            let item = $create;
            if item.is_null() { return ptr::null_mut(); }
            if cJSON_AddItemToObject(object, name, item) == 0 {
                cJSON_Delete(item);
                return ptr::null_mut();
            }
            item
        }
    };
}

add_to_object!(cJSON_AddNullToObject,   cJSON_CreateNull());
add_to_object!(cJSON_AddTrueToObject,   cJSON_CreateTrue());
add_to_object!(cJSON_AddFalseToObject,  cJSON_CreateFalse());
add_to_object!(cJSON_AddObjectToObject, cJSON_CreateObject());
add_to_object!(cJSON_AddArrayToObject,  cJSON_CreateArray());

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
        cJSON_Delete(item); return ptr::null_mut();
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
        cJSON_Delete(item); return ptr::null_mut();
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
        cJSON_Delete(item); return ptr::null_mut();
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
        cJSON_Delete(item); return ptr::null_mut();
    }
    item
}

// ── 5.17  Setters ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_SetNumberHelper(
    object: *mut cJSON,
    number: c_double,
) -> c_double {
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
    // Corrupted: original valuestring was freed/nulled
    if (*object).valuestring.is_null() { return ptr::null_mut(); }
    let old_len = strlen((*object).valuestring);
    let new_len = strlen(valuestring);
    // Overlap detection: new string points inside old buffer
    let old_start = (*object).valuestring as usize;
    let old_end = old_start + old_len + 1;
    let new_start = valuestring as usize;
    if new_start >= old_start && new_start < old_end {
        return ptr::null_mut();
    }
    // Reuse buffer if new string fits
    if new_len <= old_len {
        ptr::copy_nonoverlapping(valuestring as *const u8, (*object).valuestring as *mut u8, new_len);
        *((*object).valuestring.add(new_len)) = 0;
        return (*object).valuestring;
    }
    // Allocate new buffer for longer string
    let new_str = cstr_dup(valuestring);
    if new_str.is_null() { return ptr::null_mut(); }
    internal_free((*object).valuestring as *mut c_void);
    (*object).valuestring = new_str;
    new_str
}

// ── 5.18  Memory pass-through ────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn cJSON_malloc(size: usize) -> *mut c_void {
    internal_malloc(size)
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_free(object: *mut c_void) {
    internal_free(object)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  §6  Internal API  (symbols added by user to cJSON.h)
//
//  Category A – Memory / linked-list utilities: implemented normally.
//  Category B – parse_buffer / printbuffer pipeline: LX-json has no
//               equivalent architecture, so these are exported as stubs
//               that always return failure (0 / null).
// ═══════════════════════════════════════════════════════════════════════════════

// ── 6A  Working utilities ────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn global_hooks_get() -> *mut internal_hooks {
    &raw mut GLOBAL_HOOKS
}

#[no_mangle]
pub unsafe extern "C" fn cJSON_strdup(
    string: *const u8,
    hooks: *const internal_hooks,
) -> *mut u8 {
    if string.is_null() { return ptr::null_mut(); }
    let length = strlen(string as *const c_char) + 1;
    let copy = hooks_allocate(&*hooks, length) as *mut u8;
    if copy.is_null() { return ptr::null_mut(); }
    memcpy(copy as *mut c_void, string as *const c_void, length);
    copy
}

#[no_mangle]
pub unsafe extern "C" fn add_item_to_array(
    array: *mut cJSON,
    item: *mut cJSON,
) -> c_int {
    if item.is_null() || array.is_null() || array == item { return 0; }
    let child = (*array).child;
    if child.is_null() {
        (*array).child = item;
        (*item).prev = item; // tail pointer to self
        (*item).next = ptr::null_mut();
    } else if !(*child).prev.is_null() {
        suffix_object((*child).prev, item);
        (*(*array).child).prev = item;
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn compare_double(a: c_double, b: c_double) -> c_int {
    let max_val = if a.abs() > b.abs() { a.abs() } else { b.abs() };
    if (a - b).abs() <= max_val * f64::EPSILON { 1 } else { 0 }
}

// ── 6B  Stubs  (parse_buffer / printbuffer – no LX-json equivalent) ─────────

#[no_mangle]
pub unsafe extern "C" fn parse_hex4(_input: *const u8) -> u32 { 0 }

#[no_mangle]
pub unsafe extern "C" fn parse_number(_item: *mut cJSON, _buf: *mut parse_buffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn parse_string(_item: *mut cJSON, _buf: *mut parse_buffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn parse_value(_item: *mut cJSON, _buf: *mut parse_buffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn parse_array(_item: *mut cJSON, _buf: *mut parse_buffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn parse_object(_item: *mut cJSON, _buf: *mut parse_buffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn skip_utf8_bom(_buf: *mut parse_buffer) -> *mut parse_buffer {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn ensure(_p: *mut printbuffer, _needed: usize) -> *mut u8 {
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn print_number(_item: *const cJSON, _buf: *mut printbuffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn print_string_ptr(_input: *const u8, _buf: *mut printbuffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn print_value(_item: *const cJSON, _buf: *mut printbuffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn print_array(_item: *const cJSON, _buf: *mut printbuffer) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn print_object(_item: *const cJSON, _buf: *mut printbuffer) -> c_int { 0 }
