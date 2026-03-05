/**
 * @file lx_json.h
 * @brief C ABI bindings for LX-json library
 * 
 * This header provides C-compatible FFI interfaces that allow any language
 * that can call C functions to use LX-json functionality.
 * 
 * @version 0.1.0
 * @author Lingxi
 */

#ifndef LX_JSON_H
#define LX_JSON_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Opaque pointer type for JSON node
 * 
 * This represents a JSON value (null, bool, number, string, array, or object)
 */
typedef struct LXJsonNode LXJsonNode;

/**
 * @brief Opaque pointer type for parse options
 */
typedef struct LXParseOptions LXParseOptions;

/* ==========================================================================
 * Memory Management Functions
 * ========================================================================== */

/**
 * @brief Free a JSON node
 * 
 * Must be called for any LXJsonNode* returned by the library
 * to avoid memory leaks.
 * 
 * @param node The node to free (can be NULL)
 */
void lx_json_free(LXJsonNode* node);

/**
 * @brief Free a string allocated by the library
 * 
 * Must be called for any char* returned by functions like lx_json_print()
 * to avoid memory leaks.
 * 
 * @param s The string to free (can be NULL)
 */
void lx_json_free_string(char* s);

/**
 * @brief Get the last error message
 * 
 * Returns a C string describing the last error that occurred.
 * The returned string must be freed with lx_json_free_string().
 * 
 * @return Error message string, or NULL if no error occurred
 */
const char* lx_json_get_last_error(void);

/* ==========================================================================
 * Parsing Functions
 * ========================================================================== */

/**
 * @brief Parse a JSON string
 * 
 * @param json Null-terminated JSON string
 * @return Parsed JSON node, or NULL on error
 */
LXJsonNode* lx_json_parse(const char* json);

/**
 * @brief Parse a JSON string with length
 * 
 * @param json JSON string
 * @param len Length of the string in bytes
 * @return Parsed JSON node, or NULL on error
 */
LXJsonNode* lx_json_parse_with_length(const char* json, size_t len);

/**
 * @brief Create default parse options
 * 
 * @return New parse options object
 */
LXParseOptions* lx_json_parse_options_new(void);

/**
 * @brief Free parse options
 * 
 * @param opts Options to free (can be NULL)
 */
void lx_json_parse_options_free(LXParseOptions* opts);

/**
 * @brief Set nesting limit in parse options
 * 
 * @param opts Parse options
 * @param limit Maximum nesting depth
 */
void lx_json_parse_options_set_nesting_limit(LXParseOptions* opts, size_t limit);

/**
 * @brief Set require null terminated flag
 * 
 * @param opts Parse options
 * @param value Non-zero to require null termination, 0 otherwise
 */
void lx_json_parse_options_set_require_null_terminated(LXParseOptions* opts, int value);

/**
 * @brief Parse JSON with options
 * 
 * @param json Null-terminated JSON string
 * @param opts Parse options (pointer to a valid LXParseOptions)
 * @return Parsed JSON node, or NULL on error
 */
LXJsonNode* lx_json_parse_with_opts(const char* json, LXParseOptions* opts);

/* ==========================================================================
 * Serialization Functions
 * ========================================================================== */

/**
 * @brief Print JSON node to formatted string
 * 
 * The returned string must be freed with lx_json_free_string().
 * 
 * @param node JSON node
 * @return Formatted JSON string, or NULL on error
 */
char* lx_json_print(LXJsonNode* node);

/**
 * @brief Print JSON node to unformatted string
 * 
 * The returned string must be freed with lx_json_free_string().
 * 
 * @param node JSON node
 * @return Unformatted JSON string, or NULL on error
 */
char* lx_json_print_unformatted(LXJsonNode* node);

/**
 * @brief Minify JSON
 * 
 * The returned string must be freed with lx_json_free_string().
 * 
 * @param node JSON node
 * @return Minified JSON string, or NULL on error
 */
char* lx_json_minify(LXJsonNode* node);

/**
 * @brief Print JSON with pre-allocated buffer capacity
 * 
 * Serializes a JSON node with a pre-allocated buffer capacity hint.
 * The returned string must be freed with lx_json_free_string().
 * 
 * @param node JSON node
 * @param prebuffer Initial buffer capacity to allocate
 * @param fmt Non-zero to format with indentation, 0 for compact
 * @return JSON string, or NULL on error
 */
char* lx_json_print_buffered(LXJsonNode* node, size_t prebuffer, int fmt);

/**
 * @brief Print JSON into a pre-allocated buffer
 * 
 * Serializes a JSON node into a pre-allocated buffer. The buffer must be
 * large enough to hold the entire JSON string plus a null terminator.
 * 
 * @param node JSON node
 * @param buffer Pre-allocated buffer to write to (can be NULL to query length)
 * @param buffer_len Length of the buffer in bytes
 * @param fmt Non-zero to format with indentation, 0 for compact
 * @param required_length Output parameter for required buffer size (including null terminator)
 * @return 1 on success, 0 on failure or if buffer is too small
 */
int lx_json_print_preallocated(
    LXJsonNode* node,
    char* buffer,
    size_t buffer_len,
    int fmt,
    size_t* required_length
);

/* ==========================================================================
 * Type Checking Functions
 * ========================================================================== */

/**
 * @brief Check if node is null
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_null(const LXJsonNode* node);

/**
 * @brief Check if node is boolean
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_bool(const LXJsonNode* node);

/**
 * @brief Check if node is number
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_number(const LXJsonNode* node);

/**
 * @brief Check if node is string
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_string(const LXJsonNode* node);

/**
 * @brief Check if node is array
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_array(const LXJsonNode* node);

/**
 * @brief Check if node is object
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_object(const LXJsonNode* node);

/**
 * @brief Check if node is true
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_true(const LXJsonNode* node);

/**
 * @brief Check if node is false
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_false(const LXJsonNode* node);

/**
 * @brief Check if node is raw
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 otherwise
 */
int lx_json_is_raw(const LXJsonNode* node);

/* ==========================================================================
 * Constructor Functions
 * ========================================================================== */

/**
 * @brief Create a new null node
 * 
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_null(void);

/**
 * @brief Create a new boolean node
 * 
 * @param value Non-zero for true, 0 for false
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_bool(int value);

/**
 * @brief Create a new true node
 * 
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_true(void);

/**
 * @brief Create a new false node
 * 
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_false(void);

/**
 * @brief Create a new number node
 * 
 * @param value Number value
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_number(double value);

/**
 * @brief Create a new string node
 * 
 * @param value Null-terminated string
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_string(const char* value);

/**
 * @brief Create a new empty array node
 * 
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_array(void);

/**
 * @brief Create a new empty object node
 * 
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_object(void);
/**
 * @brief Create an array from i64 values
 * 
 * @param values Pointer to i64 array
 * @param len Number of elements in the array
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_int_array(const int64_t* values, size_t len);

/**
 * @brief Create an array from f32 values
 * 
 * @param values Pointer to f32 array
 * @param len Number of elements in the array
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_float_array(const float* values, size_t len);

/**
 * @brief Create an array from f64 values
 * 
 * @param values Pointer to f64 array
 * @param len Number of elements in the array
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_double_array(const double* values, size_t len);

/**
 * @brief Create an array from string values
 * 
 * @param values Pointer to array of C string pointers
 * @param len Number of elements in the array
 * @return New JSON node, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_create_string_array(const char** values, size_t len);

/* ==========================================================================
 * Value Accessor Functions
 * ========================================================================== */

/**
 * @brief Get boolean value
 * 
 * @param node JSON node
 * @return Non-zero if true, 0 if false or not a boolean
 */
int lx_json_get_bool(const LXJsonNode* node);

/**
 * @brief Get number value
 * 
 * @param node JSON node
 * @return Number value, or 0.0 if not a number
 */
double lx_json_get_number(const LXJsonNode* node);

/**
 * @brief Get string value
 * 
 * The returned string must be freed with lx_json_free_string().
 * 
 * @param node JSON node
 * @return String value, or NULL if not a string
 */
char* lx_json_get_string_value(const LXJsonNode* node);

/**
 * @brief Get number value from a JSON node
 * 
 * @param node JSON node
 * @return Number value, or 0.0 if not a number
 */
double lx_json_get_number_value(const LXJsonNode* node);

/* ==========================================================================
 * Array Operations
 * ========================================================================== */

/**
 * @brief Get array size
 * 
 * @param node JSON node
 * @return Array size, or 0 if not an array
 */
size_t lx_json_get_array_size(const LXJsonNode* node);

/**
 * @brief Get array item at index
 * 
 * The returned node must be freed with lx_json_free().
 * 
 * @param node JSON node
 * @param index Array index
 * @return Array item, or NULL on error
 */
LXJsonNode* lx_json_get_array_item(const LXJsonNode* node, size_t index);

/**
 * @brief Add item to end of array
 * 
 * @param array Array node (must be an array)
 * @param item Item to add
 * @return Non-zero on success, 0 on failure
 */
int lx_json_add_item_to_array(LXJsonNode* array, LXJsonNode* item);

/**
 * @brief Delete item from array at index
 * 
 * @param array Array node (must be an array)
 * @param index Array index
 * @return Non-zero on success, 0 on failure
 */
int lx_json_delete_item_from_array(LXJsonNode* array, size_t index);

/**
 * @brief Detach item from array (removes and returns it)
 * 
 * The returned node must be freed with lx_json_free().
 * 
 * @param array Array node (must be an array)
 * @param index Array index
 * @return Detached item, or NULL on error
 */
LXJsonNode* lx_json_detach_item_from_array(LXJsonNode* array, size_t index);

/**
 * @brief Replace item in array at index
 * 
 * @param array Array node (must be an array)
 * @param index Array index
 * @param new_item New item
 * @return Non-zero on success, 0 on failure
 */
int lx_json_replace_item_in_array(LXJsonNode* array, size_t index, LXJsonNode* new_item);

/**
 * @brief Insert item into array at index
 * 
 * @param array Array node (must be an array)
 * @param index Array index
 * @param new_item Item to insert
 * @return Non-zero on success, 0 on failure
 */
int lx_json_insert_item_in_array(LXJsonNode* array, size_t index, LXJsonNode* new_item);

/* ==========================================================================
 * Object Operations
 * ========================================================================== */

/**
 * @brief Get object item by key (case-sensitive)
 * 
 * The returned node must be freed with lx_json_free().
 * 
 * @param node JSON node (must be an object)
 * @param key Object key (null-terminated)
 * @return Object value, or NULL on error
 */
LXJsonNode* lx_json_get_object_item(const LXJsonNode* node, const char* key);

/**
 * @brief Get object item by key (case-insensitive)
 * 
 * The returned node must be freed with lx_json_free().
 * 
 * @param node JSON node (must be an object)
 * @param key Object key (null-terminated)
 * @return Object value, or NULL on error
 */
LXJsonNode* lx_json_get_object_item_case_insensitive(const LXJsonNode* node, const char* key);

/**
 * @brief Check if object has key
 * 
 * @param node JSON node (must be an object)
 * @param key Object key (null-terminated)
 * @return Non-zero if key exists, 0 otherwise
 */
int lx_json_has_object_item(const LXJsonNode* node, const char* key);

/**
 * @brief Add item to object
 * 
 * If key already exists, its value will be replaced.
 * 
 * @param object Object node (must be an object)
 * @param key Object key (null-terminated)
 * @param item Value to add
 * @return Non-zero on success, 0 on failure
 */
int lx_json_add_item_to_object(LXJsonNode* object, const char* key, LXJsonNode* item);

/**
 * @brief Delete item from object by key
 * 
 * @param object Object node (must be an object)
 * @param key Object key (null-terminated)
 * @return Non-zero on success, 0 on failure
 */
int lx_json_delete_item_from_object(LXJsonNode* object, const char* key);

/**
 * @brief Detach item from object by key (removes and returns it)
 * 
 * The returned node must be freed with lx_json_free().
 * 
 * @param object Object node (must be an object)
 * @param key Object key (null-terminated)
 * @return Detached value, or NULL on error
 */
LXJsonNode* lx_json_detach_item_from_object(LXJsonNode* object, const char* key);
/**
 * @brief Sort the keys of an object
 * 
 * @param node Object node to sort
 * @param case_sensitive If non-zero, sort with case sensitivity; otherwise, sort case-insensitively
 * @return 1 on success, 0 on error
 */
int lx_json_sort_object(LXJsonNode* node, int case_sensitive);

/**
 * @brief Get the keys of an object as an array
 * 
 * @param node Object node (must be an object)
 * @return New JSON array containing the keys, must be freed with lx_json_free()
 */
LXJsonNode* lx_json_get_object_keys(LXJsonNode* node);

/**
 * @brief Replace item in object
 * 
 * @param object Object node (must be an object)
 * @param key Object key (null-terminated)
 * @param new_item New value
 * @return Non-zero on success, 0 on failure
 */
int lx_json_replace_item_in_object(LXJsonNode* object, const char* key, LXJsonNode* new_item);

/* ==========================================================================
 * JSON Pointer (RFC 6901)
 * ========================================================================== */

/**
 * @brief Get value using JSON Pointer
 * 
 * The returned node must be freed with lx_json_free().
 * 
 * @param node JSON node
 * @param pointer JSON Pointer string (e.g., "/foo/bar/0")
 * @return Value at path, or NULL on error
 */
LXJsonNode* lx_json_get_pointer(const LXJsonNode* node, const char* pointer);

/**
 * @brief Get mutable reference using JSON Pointer
 * 
 * Note: This function is primarily useful for validating that a pointer is valid.
 * For actual mutation, use the specific mutation functions like lx_json_add_item_to_object.
 * 
 * @param node JSON node
 * @param pointer JSON Pointer string (e.g., "/foo/bar/0")
 * @return The original node pointer if pointer is "/", NULL otherwise
 */
LXJsonNode* lx_json_get_pointer_mut(LXJsonNode* node, const char* pointer);

/* ==========================================================================
 * Utility Functions
 * ========================================================================== */

/**
 * @brief Duplicate a JSON node
 * 
 * Creates a deep copy of the node.
 * The returned node must be freed with lx_json_free().
 * 
 * @param node JSON node
 * @return Duplicated node, or NULL on error
 */
LXJsonNode* lx_json_duplicate(const LXJsonNode* node);

/**
 * @brief Compare two JSON nodes
 * 
 * @param a First node
 * @param b Second node
 * @return Non-zero if equal, 0 otherwise
 */
int lx_json_compare(const LXJsonNode* a, const LXJsonNode* b);

/* ==========================================================================
 * JSON Patch (RFC 6902)
 * ========================================================================== */

/**
 * @brief Generate JSON Patch patches
 * 
 * Creates a patch array that transforms `source` into `target`.
 * The returned node must be freed with lx_json_free().
 * 
 * @param source Source JSON node
 * @param target Target JSON node
 * @return Patch array node, or NULL on error
 */
LXJsonNode* lx_json_generate_patches(const LXJsonNode* source, const LXJsonNode* target);

/**
 * @brief Add a single patch operation to a patch array
 * 
 * Adds a single patch operation to an existing patch array. The patch array
 * must be a valid JSON array (created with lx_json_create_array).
 * 
 * @param patches Patch array node (must be an array)
 * @param op Operation type: "add", "remove", "replace", "move", "copy", or "test"
 * @param path JSON Pointer path (RFC 6901)
 * @param value Value for the operation (can be NULL for operations that don't require a value)
 * @return 1 on success, 0 on error
 */
int lx_json_add_patch_to_array(
    LXJsonNode* patches,
    const char* op,
    const char* path,
    const LXJsonNode* value
);

/**
 * @brief Apply JSON Patch patches to a node
 * 
 * Creates a new node with patches applied.
 * The returned node must be freed with lx_json_free().
 * 
 * @param node JSON node to patch
 * @param patches Patch array node
 * @return Patched node, or NULL on error
 */
LXJsonNode* lx_json_apply_patches(LXJsonNode* node, const LXJsonNode* patches);

/* ==========================================================================
 * JSON Merge Patch (RFC 7396)
 * ========================================================================== */

/**
 * @brief Apply JSON Merge Patch
 * 
 * Creates a new node with merge patch applied.
 * The returned node must be freed with lx_json_free().
 * 
 * @param node JSON node to patch
 * @param patch Merge patch object node
 * @return Merged node, or NULL on error
 */
LXJsonNode* lx_json_merge_patch(LXJsonNode* node, const LXJsonNode* patch);
/**
 * Add a string to the object with the given key
 *
 * @param object The object to add the string to
 * @param key The key to add the string under
 * @param value The string value to add
 * @return 1 on success, 0 on failure
 */
int lx_json_add_string_to_object(LXJsonNode *object, const char *key, const char *value);

/**
 * Add a number to the object with the given key
 *
 * @param object The object to add the number to
 * @param key The key to add the number under
 * @param value The number value to add
 * @return 1 on success, 0 on failure
 */
int lx_json_add_number_to_object(LXJsonNode *object, const char *key, double value);

/**
 * Add a boolean to the object with the given key
 *
 * @param object The object to add the boolean to
 * @param key The key to add the boolean under
 * @param value The boolean value to add (0 = false, non-zero = true)
 * @return 1 on success, 0 on failure
 */
int lx_json_add_bool_to_object(LXJsonNode *object, const char *key, int value);

#ifdef __cplusplus
}
#endif

#endif /* LX_JSON_H */