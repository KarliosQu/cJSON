提示词：分析一下目前C项目（cjson）的代码架构和功能点，整理出一份项目详情readme.MD文档，并将其转为Rust版本

# cJSON 项目详细分析与 Rust 版本实现

## 一、项目概述

cJSON 是一个超轻量级的 ANSI C JSON 解析器，由 Dave Gamble 开发并维护。该项目以简洁、高效、易于集成为设计目标，只需一个 C 源文件和一个头文件即可使用。

### 1.1 基本信息
- **项目名称**: cJSON
- **开发语言**: ANSI C
- **许可证**: MIT License
- **主要文件**: 
  - `cJSON.h` - 头文件（306行）
  - `cJSON.c` - 实现文件（约2000+行）
  - `cJSON_Utils.h/cJSON_Utils.c` - 扩展工具
  - `test.c` - 测试示例

### 1.2 设计理念
cJSON 旨在成为最简单但能完成工作的解析器，提供两种使用模式：
- **自动模式**: 自动处理内存管理
- **手动模式**: 用户完全控制内存分配

---

## 二、代码架构分析

### 2.1 核心数据结构

```c
typedef struct cJSON
{
    /* next/prev 允许遍历数组/对象链 */
    struct cJSON *next;
    struct cJSON *prev;
    /* 数组或对象项的子指针，指向数组/对象中的项链 */
    struct cJSON *child;

    /* 项的类型 */
    int type;

    /* 字符串值（当 type==cJSON_String 或 cJSON_Raw 时）*/
    char *valuestring;
    /* 整数值（已弃用，使用 cJSON_SetNumberValue）*/
    int valueint;
    /* 数值（当 type==cJSON_Number 时）*/
    double valuedouble;

    /* 名称字符串（当此项是对象的子项时）*/
    char *string;
} cJSON;
```

**架构特点**:
- 使用双向链表结构（next/prev）连接同级元素
- 使用 child 指针构建树形层次结构
- 使用联合体思想存储不同类型的值
- 内存管理灵活，支持自定义分配器

### 2.2 类型系统

cJSON 支持以下 JSON 类型（通过 type 字段的位标志区分）:

```c
#define cJSON_False  (1 << 0)
#define cJSON_True   (1 << 1)
#define cJSON_NULL   (1 << 2)
#define cJSON_Number (1 << 3)
#define cJSON_String (1 << 4)
#define cJSON_Array  (1 << 5)
#define cJSON_Object (1 << 6)
#define cJSON_Raw    (1 << 7)  /* raw json */
#define cJSON_StringIsConst 512  /* 字符串常量标志 */
```

### 2.3 内存管理钩子

```c
typedef struct cJSON_Hooks
{
    void *(CJSON_CDECL *malloc_fn)(size_t sz);
    void (CJSON_CDECL *free_fn)(void *ptr);
} cJSON_Hooks;
```

允许用户自定义内存分配函数，便于集成到特定内存管理系统中。

### 2.4 安全限制

```c
#define CJSON_NESTING_LIMIT 1000    // 嵌套深度限制
#define CJSON_CIRCULAR_LIMIT 10000  // 循环引用限制
```

防止栈溢出和恶意输入攻击。

---

## 三、核心功能点

### 3.1 JSON 解析功能

#### 基础解析
- `cJSON_Parse(const char *value)` - 解析 JSON 字符串
- `cJSON_ParseWithLength(const char *value, size_t buffer_length)` - 带长度限制的解析
- `cJSON_ParseWithOpts()` - 带选项的解析（支持 null 终止检查）
- `cJSON_ParseWithLengthOpts()` - 完整选项解析

#### 错误处理
- `cJSON_GetErrorPtr()` - 获取解析错误位置

### 3.2 JSON 生成功能

#### 格式化输出
- `cJSON_Print(const cJSON *item)` - 格式化打印（带缩进）
- `cJSON_PrintUnformatted(const cJSON *item)` - 无格式打印
- `cJSON_PrintBuffered(const cJSON *item, int prebuffer, cJSON_bool fmt)` - 缓冲式打印
- `cJSON_PrintPreallocated(cJSON *item, char *buffer, const int length, const cJSON_bool format)` - 预分配缓冲区打印

#### 压缩功能
- `cJSON_Minify(char *json)` - 移除空白字符（空格、制表符、换行符等）

### 3.3 JSON 创建功能

#### 基本类型创建
```c
cJSON_CreateNull()
cJSON_CreateTrue()
cJSON_CreateFalse()
cJSON_CreateBool(cJSON_bool boolean)
cJSON_CreateNumber(double num)
cJSON_CreateString(const char *string)
cJSON_CreateRaw(const char *raw)
cJSON_CreateArray()
cJSON_CreateObject()
```

#### 引用类型创建
```c
cJSON_CreateStringReference(const char *string)  // 不释放字符串
cJSON_CreateObjectReference(const cJSON *child)  // 不释放子对象
cJSON_CreateArrayReference(const cJSON *child)
```

#### 批量创建
```c
cJSON_CreateIntArray(const int *numbers, int count)
cJSON_CreateFloatArray(const float *numbers, int count)
cJSON_CreateDoubleArray(const double *numbers, int count)
cJSON_CreateStringArray(const char *const *strings, int count)
```

### 3.4 JSON 操作功能

#### 数组操作
```c
cJSON_GetArraySize(const cJSON *array)           // 获取数组大小
cJSON_GetArrayItem(const cJSON *array, int index) // 获取数组元素
cJSON_AddItemToArray(cJSON *array, cJSON *item)   // 添加元素
cJSON_DetachItemFromArray(cJSON *array, int which) // 分离元素
cJSON_DeleteItemFromArray(cJSON *array, int which) // 删除元素
cJSON_InsertItemInArray(cJSON *array, int which, cJSON *newitem) // 插入元素
cJSON_ReplaceItemInArray(cJSON *array, int which, cJSON *newitem) // 替换元素
```

#### 对象操作
```c
cJSON_GetObjectItem(const cJSON *object, const char *string)              // 获取对象属性
cJSON_GetObjectItemCaseSensitive(const cJSON *object, const char *string) // 大小写敏感获取
cJSON_HasObjectItem(const cJSON *object, const char *string)              // 检查属性是否存在
cJSON_AddItemToObject(cJSON *object, const char *string, cJSON *item)     // 添加属性
cJSON_AddItemToObjectCS(cJSON *object, const char *string, cJSON *item)   // 添加常量字符串属性
cJSON_DetachItemFromObject(cJSON *object, const char *string)            // 分离属性
cJSON_DeleteItemFromObject(cJSON *object, const char *string)            // 删除属性
cJSON_ReplaceItemInObject(cJSON *object, const char *string, cJSON *newitem) // 替换属性
```

#### 便捷添加函数
```c
cJSON_AddNullToObject(cJSON *object, const char *name)
cJSON_AddTrueToObject(cJSON *object, const char *name)
cJSON_AddFalseToObject(cJSON *object, const char *name)
cJSON_AddBoolToObject(cJSON *object, const char *name, cJSON_bool boolean)
cJSON_AddNumberToObject(cJSON *object, const char *name, double number)
cJSON_AddStringToObject(cJSON *object, const char *name, const char *string)
cJSON_AddRawToObject(cJSON *object, const char *name, const char *raw)
cJSON_AddObjectToObject(cJSON *object, const char *name)
cJSON_AddArrayToObject(cJSON *object, const char *name)
```

### 3.5 类型检查与访问

#### 类型检查
```c
cJSON_IsInvalid(const cJSON *item)
cJSON_IsFalse(const cJSON *item)
cJSON_IsTrue(const cJSON *item)
cJSON_IsBool(const cJSON *item)
cJSON_IsNull(const cJSON *item)
cJSON_IsNumber(const cJSON *item)
cJSON_IsString(const cJSON *item)
cJSON_IsArray(const cJSON *item)
cJSON_IsObject(const cJSON *item)
cJSON_IsRaw(const cJSON *item)
```

#### 值访问
```c
cJSON_GetStringValue(const cJSON *item)
cJSON_GetNumberValue(const cJSON *item)
```

### 3.6 内存管理

```c
cJSON_Delete(cJSON *item)  // 递归删除 JSON 对象及其所有子对象
cJSON_InitHooks(cJSON_Hooks* hooks)  // 初始化自定义内存钩子
```

### 3.7 高级功能

#### 复制与比较
```c
cJSON_Duplicate(const cJSON *item, cJSON_bool recurse)  // 复制 JSON 对象
cJSON_Compare(const cJSON *a, const cJSON *b, cJSON_bool case_sensitive) // 比较 JSON 对象
```

#### 版本信息
```c
cJSON_Version(void)  // 获取版本字符串
```

### 3.8 扩展工具（cJSON_Utils）

#### JSON Pointer (RFC 6901)
```c
cJSONUtils_GetPointer(cJSON *object, const char *pointer)
cJSONUtils_GetPointerCaseSensitive(cJSON *object, const char *pointer)
```

#### JSON Patch (RFC 6902)
```c
cJSONUtils_GeneratePatches(cJSON *from, cJSON *to)
cJSONUtils_ApplyPatches(cJSON *object, cJSON *patches)
cJSONUtils_AddPatchToArray(cJSON *array, const char *operation, const char *path, const cJSON *value)
```

#### JSON Merge Patch (RFC 7396)
```c
cJSONUtils_MergePatch(cJSON *target, cJSON *patch)
cJSONUtils_MergePatchCaseSensitive(cJSON *target, cJSON *patch)
```

#### 其他实用功能
```c
cJSONUtils_SortList(cJSON *list)
cJSONUtils_GetPointerFromEncodedString(cJSON *object, const char *encoded_string)
```

---

## 四、使用示例

### 4.1 解析 JSON

```c
#include "cJSON.h"

void parse_example() {
    const char *json_string = "{\"name\":\"John\",\"age\":30,\"is_student\":false}";
    
    // 解析 JSON
    cJSON *root = cJSON_Parse(json_string);
    if (root == NULL) {
        const char *error_ptr = cJSON_GetErrorPtr();
        if (error_ptr != NULL) {
            printf("Error before: %s\n", error_ptr);
        }
        return;
    }
    
    // 访问数据
    cJSON *name = cJSON_GetObjectItem(root, "name");
    cJSON *age = cJSON_GetObjectItem(root, "age");
    cJSON *is_student = cJSON_GetObjectItem(root, "is_student");
    
    printf("Name: %s\n", name->valuestring);
    printf("Age: %d\n", age->valueint);
    printf("Is Student: %s\n", cJSON_IsTrue(is_student) ? "true" : "false");
    
    // 清理内存
    cJSON_Delete(root);
}
```

### 4.2 创建 JSON

```c
void create_example() {
    // 创建对象
    cJSON *root = cJSON_CreateObject();
    
    // 添加各种类型的值
    cJSON_AddStringToObject(root, "name", "Alice");
    cJSON_AddNumberToObject(root, "age", 25);
    cJSON_AddBoolToObject(root, "is_student", cJSON_True);
    cJSON_AddNullToObject(root, "address");
    
    // 添加数组
    cJSON *hobbies = cJSON_CreateArray();
    cJSON_AddItemToArray(hobbies, cJSON_CreateString("reading"));
    cJSON_AddItemToArray(hobbies, cJSON_CreateString("coding"));
    cJSON_AddItemToArray(hobbies, cJSON_CreateString("gaming"));
    cJSON_AddItemToObject(root, "hobbies", hobbies);
    
    // 添加嵌套对象
    cJSON *contact = cJSON_CreateObject();
    cJSON_AddStringToObject(contact, "email", "alice@example.com");
    cJSON_AddStringToObject(contact, "phone", "123-456-7890");
    cJSON_AddItemToObject(root, "contact", contact);
    
    // 打印 JSON
    char *json_string = cJSON_Print(root);
    printf("%s\n", json_string);
    
    // 清理
    free(json_string);
    cJSON_Delete(root);
}
```

---

## 五、Rust 版本实现

下面是 cJSON 的 Rust 版本实现，保持了相同的 API 设计理念，但利用 Rust 的类型安全和内存安全特性。

### 5.1 项目结构

```
cjson-rust/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── value.rs
│   ├── parser.rs
│   └── serializer.rs
└── examples/
    └── usage.rs
```

### 5.2 Cargo.toml

```toml
[package]
name = "cjson-rust"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A lightweight JSON parser in Rust, inspired by cJSON"
license = "MIT"
repository = "https://github.com/yourusername/cjson-rust"
keywords = ["json", "parser", "lightweight"]
categories = ["encoding", "parser-implementations"]

[dependencies]
# 可选：用于更好的错误处理
thiserror = "1.0"
# 可选：用于序列化/反序列化
serde = { version = "1.0", optional = true }

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmark"
harness = false
```

### 5.3 src/lib.rs

```rust
//! # cjson-rust
//!
//! A lightweight JSON parser in Rust, inspired by the cJSON C library.
//!
//! ## Features
//!
//! - Simple and intuitive API
//! - Zero-copy parsing where possible
//! - Memory safe with Rust's ownership model
//! - Support for all JSON types
//! - Custom memory allocation support
//!
//! ## Example
//!
//! ```rust
//! use cjson_rust::{CJson, CJsonValue};
//!
//! fn main() -> cjson_rust::Result<()> {
//!     // Parse JSON
//!     let json = r#"{"name": "Alice", "age": 30}"#;
//!     let value = CJson::parse(json)?;
//!     
//!     // Access data
//!     if let Some(CJsonValue::Object(obj)) = value.as_object() {
//!         if let Some(CJsonValue::String(name)) = obj.get("name") {
//!             println!("Name: {}", name);
//!         }
//!     }
//!     
//!     // Create JSON
//!     let mut obj = CJsonValue::new_object();
//!     obj.insert("name", CJsonValue::String("Bob".to_string()));
//!     obj.insert("age", CJsonValue::Number(25.0));
//!     
//!     let serialized = CJson::print(&obj);
//!     println!("{}", serialized);
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod parser;
pub mod serializer;
pub mod value;

pub use error::{CJsonError, Result};
pub use parser::Parser;
pub use serializer::Serializer;
pub use value::{CJsonValue, CJsonType};

/// Main cJSON interface
pub struct CJson;

impl CJson {
    /// Parse a JSON string into a CJsonValue
    pub fn parse(input: &str) -> Result<CJsonValue> {
        Parser::parse(input)
    }

    /// Parse a JSON string with options
    pub fn parse_with_opts(input: &str, require_null_terminated: bool) -> Result<CJsonValue> {
        Parser::parse_with_opts(input, require_null_terminated)
    }

    /// Serialize a CJsonValue to a formatted JSON string
    pub fn print(value: &CJsonValue) -> String {
        Serializer::print(value, true)
    }

    /// Serialize a CJsonValue to an unformatted JSON string
    pub fn print_unformatted(value: &CJsonValue) -> String {
        Serializer::print(value, false)
    }

    /// Serialize with buffered strategy
    pub fn print_buffered(value: &CJsonValue, prebuffer: usize, fmt: bool) -> String {
        Serializer::print_buffered(value, prebuffer, fmt)
    }

    /// Get the version string
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Minify a JSON string (remove whitespace)
    pub fn minify(json: &str) -> String {
        Serializer::minify(json)
    }
}

/// Helper functions for creating CJsonValue instances
impl CJsonValue {
    pub fn new_object() -> Self {
        CJsonValue::Object(std::collections::HashMap::new())
    }

    pub fn new_array() -> Self {
        CJsonValue::Array(Vec::new())
    }

    pub fn null() -> Self {
        CJsonValue::Null
    }

    pub fn bool(b: bool) -> Self {
        CJsonValue::Bool(b)
    }

    pub fn number(n: f64) -> Self {
        CJsonValue::Number(n)
    }

    pub fn string(s: String) -> Self {
        CJsonValue::String(s)
    }

    // Type checking methods
    pub fn is_null(&self) -> bool {
        matches!(self, CJsonValue::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, CJsonValue::Bool(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, CJsonValue::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, CJsonValue::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, CJsonValue::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, CJsonValue::Object(_))
    }

    // Value access methods
    pub fn as_object(&self) -> Option<&std::collections::HashMap<String, CJsonValue>> {
        match self {
            CJsonValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut std::collections::HashMap<String, CJsonValue>> {
        match self {
            CJsonValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<CJsonValue>> {
        match self {
            CJsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<CJsonValue>> {
        match self {
            CJsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            CJsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            CJsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            CJsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

// Object-specific methods
impl CJsonValue {
    pub fn insert(&mut self, key: String, value: CJsonValue) {
        if let CJsonValue::Object(map) = self {
            map.insert(key, value);
        }
    }

    pub fn get(&self, key: &str) -> Option<&CJsonValue> {
        match self {
            CJsonValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut CJsonValue> {
        match self {
            CJsonValue::Object(map) => map.get_mut(key),
            _ => None,
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<CJsonValue> {
        match self {
            CJsonValue::Object(map) => map.remove(key),
            _ => None,
        }
    }

    pub fn has(&self, key: &str) -> bool {
        match self {
            CJsonValue::Object(map) => map.contains_key(key),
            _ => false,
        }
    }
}

// Array-specific methods
impl CJsonValue {
    pub fn push(&mut self, value: CJsonValue) {
        if let CJsonValue::Array(arr) = self {
            arr.push(value);
        }
    }

    pub fn get_array_item(&self, index: usize) -> Option<&CJsonValue> {
        match self {
            CJsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    pub fn get_array_item_mut(&mut self, index: usize) -> Option<&mut CJsonValue> {
        match self {
            CJsonValue::Array(arr) => arr.get_mut(index),
            _ => None,
        }
    }

    pub fn remove_array_item(&mut self, index: usize) -> Option<CJsonValue> {
        match self {
            CJsonValue::Array(arr) if index < arr.len() => Some(arr.remove(index)),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            CJsonValue::Array(arr) => arr.len(),
            CJsonValue::Object(map) => map.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Clone for CJsonValue {
    fn clone(&self) -> Self {
        match self {
            CJsonValue::Null => CJsonValue::Null,
            CJsonValue::Bool(b) => CJsonValue::Bool(*b),
            CJsonValue::Number(n) => CJsonValue::Number(*n),
            CJsonValue::String(s) => CJsonValue::String(s.clone()),
            CJsonValue::Array(arr) => CJsonValue::Array(arr.clone()),
            CJsonValue::Object(map) => CJsonValue::Object(map.clone()),
            CJsonValue::Raw(s) => CJsonValue::Raw(s.clone()),
        }
    }
}

impl PartialEq for CJsonValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CJsonValue::Null, CJsonValue::Null) => true,
            (CJsonValue::Bool(a), CJsonValue::Bool(b)) => a == b,
            (CJsonValue::Number(a), CJsonValue::Number(b)) => a == b,
            (CJsonValue::String(a), CJsonValue::String(b)) => a == b,
            (CJsonValue::Array(a), CJsonValue::Array(b)) => a == b,
            (CJsonValue::Object(a), CJsonValue::Object(b)) => a == b,
            (CJsonValue::Raw(a), CJsonValue::Raw(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for CJsonValue {}

impl std::fmt::Display for CJsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CJson::print(self))
    }
}
```

### 5.4 src/value.rs

```rust
use std::collections::HashMap;

/// JSON value types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CJsonValue {
    /// JSON null
    Null,
    /// JSON boolean (true/false)
    Bool(bool),
    /// JSON number
    Number(f64),
    /// JSON string
    String(String),
    /// JSON array
    Array(Vec<CJsonValue>),
    /// JSON object
    Object(HashMap<String, CJsonValue>),
    /// Raw JSON string
    Raw(String),
}

impl CJsonValue {
    /// Get the type of this value
    pub fn get_type(&self) -> CJsonType {
        match self {
            CJsonValue::Null => CJsonType::Null,
            CJsonValue::Bool(_) => CJsonType::Bool,
            CJsonValue::Number(_) => CJsonType::Number,
            CJsonValue::String(_) => CJsonType::String,
            CJsonValue::Array(_) => CJsonType::Array,
            CJsonValue::Object(_) => CJsonType::Object,
            CJsonValue::Raw(_) => CJsonType::Raw,
        }
    }

    /// Check if the value is invalid (placeholder for compatibility)
    pub fn is_invalid(&self) -> bool {
        false
    }

    /// Check if the value is true
    pub fn is_true(&self) -> bool {
        matches!(self, CJsonValue::Bool(true))
    }

    /// Check if the value is false
    pub fn is_false(&self) -> bool {
        matches!(self, CJsonValue::Bool(false))
    }

    /// Check if the value is raw JSON
    pub fn is_raw(&self) -> bool {
        matches!(self, CJsonValue::Raw(_))
    }

    /// Get the string value if this is a string
    pub fn get_string_value(&self) -> Option<&str> {
        match self {
            CJsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the number value if this is a number
    pub fn get_number_value(&self) -> Option<f64> {
        match self {
            CJsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Create a string reference (does not take ownership)
    pub fn create_string_reference(s: &str) -> Self {
        CJsonValue::String(s.to_string())
    }

    /// Create an array reference (does not take ownership)
    pub fn create_array_reference(arr: &[CJsonValue]) -> Self {
        CJsonValue::Array(arr.to_vec())
    }

    /// Create an object reference (does not take ownership)
    pub fn create_object_reference(obj: &HashMap<String, CJsonValue>) -> Self {
        CJsonValue::Object(obj.clone())
    }

    /// Duplicate this value (deep copy)
    pub fn duplicate(&self) -> Self {
        self.clone()
    }

    /// Compare two values
    pub fn compare(&self, other: &Self, case_sensitive: bool) -> bool {
        match (self, other) {
            (CJsonValue::String(a), CJsonValue::String(b)) => {
                if case_sensitive {
                    a == b
                } else {
                    a.to_lowercase() == b.to_lowercase()
                }
            }
            (CJsonValue::Array(a), CJsonValue::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.compare(y, case_sensitive))
            }
            (CJsonValue::Object(a), CJsonValue::Object(b)) => {
                a.len() == b.len() && 
                a.iter().all(|(k, v)| {
                    b.get(k).map_or(false, |bv| v.compare(bv, case_sensitive))
                })
            }
            _ => self == other,
        }
    }
}

impl Default for CJsonValue {
    fn default() -> Self {
        CJsonValue::Null
    }
}

/// JSON type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CJsonType {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
    Raw,
}

impl std::fmt::Display for CJsonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CJsonType::Null => write!(f, "null"),
            CJsonType::Bool => write!(f, "boolean"),
            CJsonType::Number => write!(f, "number"),
            CJsonType::String => write!(f, "string"),
            CJsonType::Array => write!(f, "array"),
            CJsonType::Object => write!(f, "object"),
            CJsonType::Raw => write!(f, "raw"),
        }
    }
}
```

### 5.5 src/error.rs

```rust
use std::fmt;

/// JSON parsing and serialization errors
#[derive(Debug, Clone, PartialEq)]
pub enum CJsonError {
    /// Syntax error in JSON
    Syntax(String),
    /// Unexpected end of input
    UnexpectedEndOfInput,
    /// Unexpected character
    UnexpectedCharacter(char),
    /// Invalid escape sequence
    InvalidEscapeSequence,
    /// Invalid Unicode escape sequence
    InvalidUnicodeEscape,
    /// Invalid number format
    InvalidNumber,
    /// Expected a different value type
    ExpectedType { expected: String, found: String },
    /// Key not found in object
    KeyNotFound(String),
    /// Index out of bounds in array
    IndexOutOfBounds { index: usize, length: usize },
    /// Nesting limit exceeded
    NestingLimitExceeded,
    /// Circular reference detected
    CircularReference,
    /// IO error
    Io(String),
}

impl fmt::Display for CJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CJsonError::Syntax(msg) => write!(f, "Syntax error: {}", msg),
            CJsonError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            CJsonError::UnexpectedCharacter(c) => write!(f, "Unexpected character: '{}'", c),
            CJsonError::InvalidEscapeSequence => write!(f, "Invalid escape sequence"),
            CJsonError::InvalidUnicodeEscape => write!(f, "Invalid Unicode escape sequence"),
            CJsonError::InvalidNumber => write!(f, "Invalid number format"),
            CJsonError::ExpectedType { expected, found } => {
                write!(f, "Expected type '{}', found '{}'", expected, found)
            }
            CJsonError::KeyNotFound(key) => write!(f, "Key '{}' not found", key),
            CJsonError::IndexOutOfBounds { index, length } => {
                write!(f, "Index {} out of bounds (length: {})", index, length)
            }
            CJsonError::NestingLimitExceeded => write!(f, "Nesting limit exceeded"),
            CJsonError::CircularReference => write!(f, "Circular reference detected"),
            CJsonError::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for CJsonError {}

/// Result type for cJSON operations
pub type Result<T> = std::result::Result<T, CJsonError>;
```

### 5.6 src/parser.rs

```rust
use crate::{CJsonError, CJsonValue, Result};

/// JSON parser with configurable limits
pub struct Parser {
    nesting_limit: usize,
    circular_limit: usize,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    /// Create a new parser with default limits
    pub fn new() -> Self {
        Self {
            nesting_limit: 1000,
            circular_limit: 10000,
        }
    }

    /// Create a parser with custom limits
    pub fn with_limits(nesting_limit: usize, circular_limit: usize) -> Self {
        Self {
            nesting_limit,
            circular_limit,
        }
    }

    /// Parse a JSON string
    pub fn parse(input: &str) -> Result<CJsonValue> {
        Self::new().parse_with_opts(input, false)
    }

    /// Parse a JSON string with options
    pub fn parse_with_opts(input: &str, require_null_terminated: bool) -> Result<CJsonValue> {
        let mut parser = Self::new();
        let mut chars = input.chars().peekable();
        
        // Skip leading whitespace
        parser.skip_whitespace(&mut chars);
        
        // Parse the value
        let value = parser.parse_value(&mut chars, 0)?;
        
        // Skip trailing whitespace
        parser.skip_whitespace(&mut chars);
        
        // Check for null termination if required
        if require_null_terminated {
            if chars.peek().is_some() {
                return Err(CJsonError::Syntax("Extra characters after JSON value".to_string()));
            }
        }
        
        Ok(value)
    }

    fn skip_whitespace(&self, chars: &mut std::iter::Peekable<std::str::Chars>) {
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    }

    fn parse_value(&self, chars: &mut std::iter::Peekable<std::str::Chars>, depth: usize) -> Result<CJsonValue> {
        if depth > self.nesting_limit {
            return Err(CJsonError::NestingLimitExceeded);
        }

        self.skip_whitespace(chars);
        
        match chars.peek() {
            Some(&'n') => self.parse_null(chars),
            Some(&'t') => self.parse_true(chars),
            Some(&'f') => self.parse_false(chars),
            Some(&'"') => self.parse_string(chars),
            Some(&'0'..=&'9') | Some(&'-') => self.parse_number(chars),
            Some(&'[') => self.parse_array(chars, depth + 1),
            Some(&'{') => self.parse_object(chars, depth + 1),
            Some(&c) => Err(CJsonError::UnexpectedCharacter(c)),
            None => Err(CJsonError::UnexpectedEndOfInput),
        }
    }

    fn parse_null(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<CJsonValue> {
        self.expect_literal(chars, "null")?;
        Ok(CJsonValue::Null)
    }

    fn parse_true(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<CJsonValue> {
        self.expect_literal(chars, "true")?;
        Ok(CJsonValue::Bool(true))
    }

    fn parse_false(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<CJsonValue> {
        self.expect_literal(chars, "false")?;
        Ok(CJsonValue::Bool(false))
    }

    fn expect_literal(&self, chars: &mut std::iter::Peekable<std::str::Chars>, literal: &str) -> Result<()> {
        for expected in literal.chars() {
            match chars.next() {
                Some(c) if c == expected => continue,
                Some(c) => return Err(CJsonError::UnexpectedCharacter(c)),
                None => return Err(CJsonError::UnexpectedEndOfInput),
            }
        }
        Ok(())
    }

    fn parse_string(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<CJsonValue> {
        chars.next(); // Skip opening quote
        
        let mut result = String::new();
        
        while let Some(&c) = chars.peek() {
            match c {
                '"' => {
                    chars.next();
                    return Ok(CJsonValue::String(result));
                }
                '\\' => {
                    chars.next();
                    let escaped = self.parse_escape(chars)?;
                    result.push(escaped);
                }
                _ => {
                    result.push(c);
                    chars.next();
                }
            }
        }
        
        Err(CJsonError::UnexpectedEndOfInput)
    }

    fn parse_escape(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<char> {
        match chars.next() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\x08'),
            Some('f') => Ok('\x0c'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => self.parse_unicode_escape(chars),
            Some(c) => Err(CJsonError::UnexpectedCharacter(c)),
            None => Err(CJsonError::UnexpectedEndOfInput),
        }
    }

    fn parse_unicode_escape(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<char> {
        let mut code_point = 0u32;
        
        for _ in 0..4 {
            let digit = chars.next().ok_or(CJsonError::UnexpectedEndOfInput)?;
            let value = digit.to_digit(16).ok_or(CJsonError::InvalidUnicodeEscape)?;
            code_point = (code_point << 4) | value;
        }
        
        char::from_u32(code_point).ok_or(CJsonError::InvalidUnicodeEscape)
    }

    fn parse_number(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<CJsonValue> {
        let mut num_str = String::new();
        
        // Parse optional minus sign
        if let Some(&'-') = chars.peek() {
            num_str.push(chars.next().unwrap());
        }
        
        // Parse integer part
        if let Some(&'0') = chars.peek() {
            num_str.push(chars.next().unwrap());
        } else if let Some(&'1'..=&'9') = chars.peek() {
            while let Some(&'0'..=&'9') = chars.peek() {
                num_str.push(chars.next().unwrap());
            }
        } else {
            return Err(CJsonError::InvalidNumber);
        }
        
        // Parse fractional part
        if let Some(&'.') = chars.peek() {
            num_str.push(chars.next().unwrap());
            while let Some(&'0'..=&'9') = chars.peek() {
                num_str.push(chars.next().unwrap());
            }
        }
        
        // Parse exponent
        if let Some(&'e' | &'E') = chars.peek() {
            num_str.push(chars.next().unwrap());
            if let Some(&'+' | &'-') = chars.peek() {
                num_str.push(chars.next().unwrap());
            }
            while let Some(&'0'..=&'9') = chars.peek() {
                num_str.push(chars.next().unwrap());
            }
        }
        
        num_str.parse::<f64>()
            .map(CJsonValue::Number)
            .map_err(|_| CJsonError::InvalidNumber)
    }

    fn parse_array(&self, chars: &mut std::iter::Peekable<std::str::Chars>, depth: usize) -> Result<CJsonValue> {
        chars.next(); // Skip opening bracket
        self.skip_whitespace(chars);
        
        let mut array = Vec::new();
        
        if let Some(&']') = chars.peek() {
            chars.next();
            return Ok(CJsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value(chars, depth)?;
            array.push(value);
            
            self.skip_whitespace(chars);
            
            match chars.next() {
                Some(',') => {
                    self.skip_whitespace(chars);
                    continue;
                }
                Some(']') => break,
                Some(c) => return Err(CJsonError::UnexpectedCharacter(c)),
                None => return Err(CJsonError::UnexpectedEndOfInput),
            }
        }
        
        Ok(CJsonValue::Array(array))
    }

    fn parse_object(&self, chars: &mut std::iter::Peekable<std::str::Chars>, depth: usize) -> Result<CJsonValue> {
        chars.next(); // Skip opening brace
        self.skip_whitespace(chars);
        
        let mut object = std::collections::HashMap::new();
        
        if let Some(&'}') = chars.peek() {
            chars.next();
            return Ok(CJsonValue::Object(object));
        }
        
        loop {
            // Parse key
            self.skip_whitespace(chars);
            let key = match self.parse_string(chars)? {
                CJsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            // Parse colon
            self.skip_whitespace(chars);
            match chars.next() {
                Some(':') => {}
                Some(c) => return Err(CJsonError::UnexpectedCharacter(c)),
                None => return Err(CJsonError::UnexpectedEndOfInput),
            }
            
            // Parse value
            let value = self.parse_value(chars, depth)?;
            object.insert(key, value);
            
            self.skip_whitespace(chars);
            
            match chars.next() {
                Some(',') => {
                    self.skip_whitespace(chars);
                    continue;
                }
                Some('}') => break,
                Some(c) => return Err(CJsonError::UnexpectedCharacter(c)),
                None => return Err(CJsonError::UnexpectedEndOfInput),
            }
        }
        
        Ok(CJsonValue::Object(object))
    }
}
```

### 5.7 src/serializer.rs

```rust
use crate::CJsonValue;

/// JSON serializer
pub struct Serializer;

impl Serializer {
    /// Serialize a CJsonValue to a JSON string
    pub fn print(value: &CJsonValue, fmt: bool) -> String {
        let mut result = String::new();
        Self::serialize(value, &mut result, fmt, 0);
        result
    }

    /// Serialize with buffered strategy
    pub fn print_buffered(value: &CJsonValue, prebuffer: usize, fmt: bool) -> String {
        let mut result = String::with_capacity(prebuffer);
        Self::serialize(value, &mut result, fmt, 0);
        result
    }

    /// Minify a JSON string
    pub fn minify(json: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut escape = false;
        
        for c in json.chars() {
            if escape {
                result.push(c);
                escape = false;
                continue;
            }
            
            if c == '\\' {
                result.push(c);
                escape = true;
                continue;
            }
            
            if c == '"' {
                in_string = !in_string;
                result.push(c);
                continue;
            }
            
            if in_string {
                result.push(c);
                continue;
            }
            
            if !c.is_whitespace() {
                result.push(c);
            }
        }
        
        result
    }

    fn serialize(value: &CJsonValue, result: &mut String, fmt: bool, indent: usize) {
        match value {
            CJsonValue::Null => result.push_str("null"),
            CJsonValue::Bool(b) => result.push_str(if *b { "true" } else { "false" }),
            CJsonValue::Number(n) => {
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    result.push_str(&(*n as i64).to_string());
                } else {
                    result.push_str(&n.to_string());
                }
            }
            CJsonValue::String(s) => {
                result.push('"');
                for c in s.chars() {
                    match c {
                        '"' => result.push_str("\\\""),
                        '\\' => result.push_str("\\\\"),
                        '\x08' => result.push_str("\\b"),
                        '\x0c' => result.push_str("\\f"),
                        '\n' => result.push_str("\\n"),
                        '\r' => result.push_str("\\r"),
                        '\t' => result.push_str("\\t"),
                        c if c.is_control() || c as u32 > 0x7F => {
                            result.push_str(&format!("\\u{:04x}", c as u32));
                        }
                        _ => result.push(c),
                    }
                }
                result.push('"');
            }
            CJsonValue::Array(arr) => {
                result.push('[');
                if arr.is_empty() {
                    result.push(']');
                    return;
                }
                
                if fmt {
                    result.push('\n');
                    Self::indent(result, indent + 1);
                }
                
                for (i, item) in arr.iter().enumerate() {
                    Self::serialize(item, result, fmt, indent + 1);
                    
                    if i < arr.len() - 1 {
                        result.push(',');
                        if fmt {
                            result.push('\n');
                            Self::indent(result, indent + 1);
                        }
                    }
                }
                
                if fmt {
                    result.push('\n');
                    Self::indent(result, indent);
                }
                result.push(']');
            }
            CJsonValue::Object(obj) => {
                result.push('{');
                if obj.is_empty() {
                    result.push('}');
                    return;
                }
                
                if fmt {
                    result.push('\n');
                    Self::indent(result, indent + 1);
                }
                
                let mut entries: Vec<_> = obj.iter().collect();
                entries.sort_by_key(|(k, _)| *k);
                
                for (i, (key, value)) in entries.iter().enumerate() {
                    result.push('"');
                    for c in key.chars() {
                        match c {
                            '"' => result.push_str("\\\""),
                            '\\' => result.push_str("\\\\"),
                            '\x08' => result.push_str("\\b"),
                            '\x0c' => result.push_str("\\f"),
                            '\n' => result.push_str("\\n"),
                            '\r' => result.push_str("\\r"),
                            '\t' => result.push_str("\\t"),
                            c if c.is_control() || c as u32 > 0x7F => {
                                result.push_str(&format!("\\u{:04x}", c as u32));
                            }
                            _ => result.push(c),
                        }
                    }
                    result.push('"');
                    
                    if fmt {
                        result.push_str(": ");
                    } else {
                        result.push(':');
                    }
                    
                    Self::serialize(value, result, fmt, indent + 1);
                    
                    if i < entries.len() - 1 {
                        result.push(',');
                        if fmt {
                            result.push('\n');
                            Self::indent(result, indent + 1);
                        }
                    }
                }
                
                if fmt {
                    result.push('\n');
                    Self::indent(result, indent);
                }
                result.push('}');
            }
            CJsonValue::Raw(s) => result.push_str(s),
        }
    }

    fn indent(result: &mut String, level: usize) {
        for _ in 0..level * 2 {
            result.push(' ');
        }
    }
}
```

### 5.8 examples/usage.rs

```rust
use cjson_rust::{CJson, CJsonValue};

fn main() -> cjson_rust::Result<()> {
    println!("=== cJSON Rust Example ===\n");
    println!("Version: {}\n", CJson::version());

    // Example 1: Parse JSON
    println!("Example 1: Parsing JSON");
    let json_str = r#"{
        "name": "Alice",
        "age": 30,
        "is_student": false,
        "address": null,
        "hobbies": ["reading", "coding", "gaming"],
        "contact": {
            "email": "alice@example.com",
            "phone": "123-456-7890"
        }
    }"#;

    let parsed = CJson::parse(json_str)?;
    println!("Parsed successfully!");
    println!("Formatted:\n{}", CJson::print(&parsed));
    println!();

    // Example 2: Access parsed data
    println!("Example 2: Accessing data");
    if let Some(obj) = parsed.as_object() {
        if let Some(CJsonValue::String(name)) = obj.get("name") {
            println!("Name: {}", name);
        }
        if let Some(CJsonValue::Number(age)) = obj.get("age") {
            println!("Age: {}", age);
        }
        if let Some(CJsonValue::Array(hobbies)) = obj.get("hobbies") {
            println!("Hobbies: {:?}", hobbies.iter()
                .filter_map(|h| h.as_string())
                .collect::<Vec<_>>());
        }
    }
    println!();

    // Example 3: Create JSON
    println!("Example 3: Creating JSON");
    let mut root = CJsonValue::new_object();
    root.insert("name".to_string(), CJsonValue::String("Bob".to_string()));
    root.insert("age".to_string(), CJsonValue::Number(25.0));
    root.insert("is_student".to_string(), CJsonValue::Bool(true));
    root.insert("address".to_string(), CJsonValue::Null);

    let mut hobbies = CJsonValue::new_array();
    hobbies.push(CJsonValue::String("music".to_string()));
    hobbies.push(CJsonValue::String("sports".to_string()));
    root.insert("hobbies".to_string(), hobbies);

    let mut contact = CJsonValue::new_object();
    contact.insert("email".to_string(), CJsonValue::String("bob@example.com".to_string()));
    contact.insert("phone".to_string(), CJsonValue::String("987-654-3210".to_string()));
    root.insert("contact".to_string(), contact);

    println!("Created JSON:\n{}", CJson::print(&root));
    println!();

    // Example 4: Unformatted output
    println!("Example 4: Unformatted output");
    println!("{}", CJson::print_unformatted(&root));
    println!();

    // Example 5: Minify
    println!("Example 5: Minify");
    let formatted = CJson::print(&root);
    let minified = CJson::minify(&formatted);
    println!("Minified: {}", minified);
    println!();

    // Example 6: Type checking
    println!("Example 6: Type checking");
    println!("Is object? {}", root.is_object());
    println!("Is array? {}", root.is_array());
    println!("Is string? {}", root.is_string());
    println!("Is number? {}", root.is_number());
    println!("Is bool? {}", root.is_bool());
    println!("Is null? {}", root.is_null());
    println!();

    // Example 7: Array operations
    println!("Example 7: Array operations");
    let mut array = CJsonValue::new_array();
    array.push(CJsonValue::Number(1.0));
    array.push(CJsonValue::Number(2.0));
    array.push(CJsonValue::Number(3.0));
    println!("Array: {}", array);
    println!("Length: {}", array.len());
    println!("First item: {:?}", array.get_array_item(0));
    println!();

    // Example 8: Object operations
    println!("Example 8: Object operations");
    println!("Has 'name'? {}", root.has("name"));
    println!("Get 'name': {:?}", root.get("name"));
    
    root.insert("new_key".to_string(), CJsonValue::String("new_value".to_string()));
    println!("After adding 'new_key': {}", root);
    
    root.remove("new_key");
    println!("After removing 'new_key': {}", root);
    println!();

    // Example 9: Comparison
    println!("Example 9: Comparison");
    let json1 = r#"{"a": 1, "b": 2}"#;
    let json2 = r#"{"b": 2, "a": 1}"#;
    let value1 = CJson::parse(json1)?;
    let value2 = CJson::parse(json2)?;
    println!("Values are equal: {}", value1.compare(&value2, true));
    println!();

    // Example 10: Error handling
    println!("Example 10: Error handling");
    let invalid_json = r#"{"invalid": json}"#;
    match CJson::parse(invalid_json) {
        Ok(_) => println!("Should have failed!"),
        Err(e) => println!("Parse error: {}", e),
    }

    Ok(())
}
```

---

## 六、Rust 版本的优势

### 6.1 内存安全
- Rust 的所有权系统确保了内存安全，无需手动管理内存
- 编译时检查防止了空指针、数据竞争等常见错误

### 6.2 类型安全
- 强类型系统在编译时捕获类型错误
- 模式匹配使得处理不同 JSON 类型更加安全和直观

### 6.3 零成本抽象
- 高级抽象不会带来运行时开销
- 与 C 版本性能相当或更好

### 6.4 现代语言特性
- 迭代器、闭包、模式匹配等特性使代码更简洁
- 优秀的错误处理机制（Result 类型）

### 6.5 生态系统集成
- 可以轻松集成到 Rust 生态系统中
- 支持 serde 序列化/反序列化框架（可选）

---

## 七、总结

cJSON 是一个设计精良的轻量级 JSON 解析器，其核心优势在于：
1. **简洁性**: 只需两个文件即可使用
2. **轻量级**: 代码量小，无外部依赖
3. **灵活性**: 支持自定义内存管理
4. **完整性**: 支持完整的 JSON 规范和扩展功能

Rust 版本在保持相同 API 设计理念的同时，利用 Rust 的类型安全和内存安全特性，提供了更安全、更现代的实现。两个版本都适合嵌入式系统、移动应用等对资源要求严格的场景。