# LX-json - 轻量级 Rust JSON 解析器使用手册

## 项目简介

LX-json 是一个使用 Rust 语言实现的轻量级 JSON 解析器库，灵感来源于著名的 cJSON C 库。它提供了简洁、高效、类型安全的 JSON 解析和序列化功能，完全支持 JSON 规范，并且无外部依赖。

### 当前版本
- **版本**: 0.1.0
- **许可证**: MIT License
- **编程语言**: Rust (Edition 2021)
- **作者**: 灵犀

### 核心特性

- **类型安全**: 使用 Rust 的枚举类型系统，确保编译时的类型安全
- **完整支持**: 完全支持 JSON 规范，包括所有标准 JSON 类型
- **无依赖**: 纯 Rust 实现，无外部依赖
- **跨平台**: 支持 Windows、Linux、macOS 等多种平台
- **高性能**: 高效的解析算法，优化的内存使用
- **安全特性**: 遵守 Rust 的所有权和借用规则，无 unsafe 代码
- **可配置**: 支持自定义解析选项（嵌套深度限制等）
- **完整测试**: 包含全面的单元测试和功能测试
- **高级功能**: 支持 JSON Pointer、JSON Patch (RFC 6902) 和 JSON Merge Patch (RFC 7396)

### 项目结构

```
lx-json/
├── Cargo.toml              # 项目配置
├── Cargo.lock              # 依赖锁定
├── README.md               # 项目说明文档
├── USER_MANUAL.md          # 本使用手册
├── ACCEPTANCE.md           # 验收文档
├── test.rs                 # 完整的测试用例（2500+行）
├── src/
│   ├── lib.rs              # 库入口，重新导出所有公开 API
│   ├── types.rs            # 核心数据结构 JsonNode 及其方法
│   ├── parser.rs           # JSON 解析器实现
│   ├── serializer.rs       # JSON 序列化器实现
│   ├── query.rs            # JSON 查询功能（包括 JSON Pointer）
│   ├── patch.rs            # JSON Patch (RFC 6902) 实现
│   ├── merge.rs            # JSON Merge Patch (RFC 7396) 实现
│   ├── utils.rs            # 工具函数（复制、比较、排序等）
│   └── error.rs            # 错误类型定义
└── build.sh / build.bat    # 构建脚本
```

## 安装和配置

### 使用 Cargo 添加依赖

在您的 Rust 项目的 `Cargo.toml` 文件中添加：

```toml
[dependencies]
lx-json = "0.1.0"
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/lingxi/lx-json.git
cd lx-json

# 构建库
cargo build --release

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

### 编译选项

在 `Cargo.toml` 中可以配置编译选项：

```toml
[profile.release]
opt-level = 3      # 优化级别
lto = true         # 启用链接时优化
```

## 核心数据结构

### JsonNode 枚举

`JsonNode` 是 LX-json 的核心数据结构，使用 Rust 的枚举类型表示所有 JSON 值类型：

```rust
pub enum JsonNode {
    Null,                                   // null 值
    Bool(bool),                             // 布尔值（true 或 false）
    Number(f64),                            // 数字（整数或浮点数）
    String(String),                         // 字符串
    Array(Vec<JsonNode>),                   // 数组
    Object(Vec<(String, JsonNode)>),        // 对象
    Raw(String),                            // 原始 JSON 字符串
}
```

### 支持的 JSON 类型

| 类型 | 枚举变体 | 说明 |
|------|---------|------|
| Null | `JsonNode::Null` | 表示 JSON 的 null 值 |
| Boolean | `JsonNode::Bool(bool)` | 表示 true 或 false |
| Number | `JsonNode::Number(f64)` | 表示整数或浮点数，使用 f64 存储 |
| String | `JsonNode::String(String)` | 表示 JSON 字符串 |
| Array | `JsonNode::Array(Vec<JsonNode>)` | 表示 JSON 数组，元素可以是任意类型 |
| Object | `JsonNode::Object(Vec<(String, JsonNode)>)` | 表示 JSON 对象，键值对集合 |
| Raw | `JsonNode::Raw(String)` | 表示原始 JSON 字符串（保留原始格式） |

## 快速开始

### 基本示例

```rust
use lx_json::{parse, JsonNode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析 JSON 字符串
    let json = r#"{"name": "John", "age": 30, "city": "New York"}"#;
    let node = parse(json)?;
    
    // 获取值
    if let Some(name) = node.get("name").and_then(|v| v.as_string()) {
        println!("Name: {}", name);
    }
    
    if let Some(age) = node.get("age").and_then(|v| v.as_number()) {
        println!("Age: {}", age);
    }
    
    Ok(())
}
```

## 核心功能模块

### 1. 解析功能 (Parsing)

解析功能将 JSON 字符串转换为 `JsonNode` 对象树。

#### parse

**函数签名**
```rust
pub fn parse(input: &str) -> Result<JsonNode>
```

**参数说明**
- `input`: 要解析的 JSON 字符串

**返回值**
- `Ok(JsonNode)`: 解析成功，返回 JSON 对象树
- `Err(JsonError)`: 解析失败，返回错误信息

**使用示例**
```rust
use lx_json::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"key": "value"}"#;
    let node = parse(json)?;
    println!("{:?}", node);
    Ok(())
}
```

#### parse_with_length

**函数签名**
```rust
pub fn parse_with_length(input: &str, length: usize) -> Result<JsonNode>
```

**参数说明**
- `input`: 要解析的 JSON 字符串
- `length`: 要解析的最大长度

**返回值**
- `Ok(JsonNode)`: 解析成功
- `Err(JsonError)`: 解析失败

**使用示例**
```rust
use lx_json::parse_with_length;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"key": "value"}"#;
    let node = parse_with_length(json, 100)?;
    println!("{:?}", node);
    Ok(())
}
```

#### parse_with_opts

**函数签名**
```rust
pub fn parse_with_opts(input: &str, options: ParseOptions) -> Result<JsonNode>
```

**参数说明**
- `input`: 要解析的 JSON 字符串
- `options`: 解析选项配置

**返回值**
- `Ok(JsonNode)`: 解析成功
- `Err(JsonError)`: 解析失败

**ParseOptions 配置**
```rust
pub struct ParseOptions {
    pub nesting_limit: usize,        // 最大嵌套深度（默认：1000）
    pub require_null_terminated: bool, // 是否要求 null 终止符（默认：false）
}
```

**使用示例**
```rust
use lx_json::{parse_with_opts, ParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"key": "value"}"#;
    let options = ParseOptions {
        nesting_limit: 500,
        require_null_terminated: false,
    };
    let node = parse_with_opts(json, options)?;
    println!("{:?}", node);
    Ok(())
}
```

### 2. 序列化功能 (Serialization)

序列化功能将 `JsonNode` 对象树转换为 JSON 字符串。

#### print

**函数签名**
```rust
pub fn print(node: &JsonNode) -> String
```

**参数说明**
- `node`: 要序列化的 JsonNode

**返回值**
- 格式化的 JSON 字符串（带缩进）

**使用示例**
```rust
use lx_json::{parse, print};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name":"John","age":30}"#;
    let node = parse(json)?;
    let formatted = print(&node);
    println!("{}", formatted);
    // 输出：
    // {
    //   "name": "John",
    //   "age": 30
    // }
    Ok(())
}
```

#### print_unformatted

**函数签名**
```rust
pub fn print_unformatted(node: &JsonNode) -> String
```

**参数说明**
- `node`: 要序列化的 JsonNode

**返回值**
- 紧凑的 JSON 字符串（无空格和换行）

**使用示例**
```rust
use lx_json::{parse, print_unformatted};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name":"John","age":30}"#;
    let node = parse(json)?;
    let compact = print_unformatted(&node);
    println!("{}", compact);
    // 输出：{"name":"John","age":30}
    Ok(())
}
```

#### print_buffered

**函数签名**
```rust
pub fn print_buffered(node: &JsonNode, prebuffer: i32, fmt: bool) -> String
```

**参数说明**
- `node`: 要序列化的 JsonNode
- `prebuffer`: 预分配缓冲区大小
- `fmt`: 是否格式化输出

**返回值**
- JSON 字符串

**使用示例**
```rust
use lx_json::{parse, print_buffered};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name":"John","age":30}"#;
    let node = parse(json)?;
    let output = print_buffered(&node, 1024, true);
    println!("{}", output);
    Ok(())
}
```

#### print_preallocated

**函数签名**
```rust
pub fn print_preallocated(node: &JsonNode, buffer: &mut String, format: bool) -> Result<()>
```

**参数说明**
- `node`: 要序列化的 JsonNode
- `buffer`: 预分配的字符串缓冲区
- `format`: 是否格式化输出

**返回值**
- `Ok(())`: 成功
- `Err(JsonError)`: 缓冲区不足等错误

**使用示例**
```rust
use lx_json::{parse, print_preallocated};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name":"John","age":30}"#;
    let node = parse(json)?;
    let mut buffer = String::with_capacity(100);
    print_preallocated(&node, &mut buffer, true)?;
    println!("{}", buffer);
    Ok(())
}
```

#### minify

**函数签名**
```rust
pub fn minify(json: &str) -> String
```

**参数说明**
- `json`: 要压缩的 JSON 字符串

**返回值**
- 压缩后的 JSON 字符串（移除所有空格和换行）

**使用示例**
```rust
use lx_json::minify;

fn main() {
    let json = r#"{
        "name": "John",
        "age": 30
    }"#;
    let minified = minify(json);
    println!("{}", minified);
    // 输出：{"name":"John","age":30}
}
```

### 3. 创建功能 (Creation)

创建各种类型的 `JsonNode`。

#### 创建基本类型

**方法签名**
```rust
// 创建 null 值
pub fn new_null() -> JsonNode

// 创建布尔值
pub fn new_bool(b: bool) -> JsonNode

// 创建 true 值
pub fn new_true() -> JsonNode

// 创建 false 值
pub fn new_false() -> JsonNode

// 创建数字值
pub fn new_number(n: f64) -> JsonNode

// 创建字符串值
pub fn new_string(s: impl Into<String>) -> JsonNode

// 创建原始 JSON 值
pub fn new_raw(s: impl Into<String>) -> JsonNode
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() {
    let null = JsonNode::new_null();
    let bool_val = JsonNode::new_bool(true);
    let true_val = JsonNode::new_true();
    let false_val = JsonNode::new_false();
    let number = JsonNode::new_number(42.0);
    let string = JsonNode::new_string("hello");
    let raw = JsonNode::new_raw(r#"{"key":"value"}"#);
    
    println!("{:?}", null);
    println!("{:?}", bool_val);
    println!("{:?}", number);
    println!("{:?}", string);
}
```

#### 创建容器类型

**方法签名**
```rust
// 创建空数组
pub fn new_array() -> JsonNode

// 创建带值的数组
pub fn create_array(values: Vec<JsonNode>) -> JsonNode

// 创建空对象
pub fn new_object() -> JsonNode

// 创建带键值对的对象
pub fn create_object(pairs: Vec<(String, JsonNode)>) -> JsonNode
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() {
    // 创建数组
    let array = JsonNode::create_array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    // 创建对象
    let object = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    println!("{:?}", array);
    println!("{:?}", object);
}
```

#### 创建类型化数组

**方法签名**
```rust
// 从 i64 切片创建数组
pub fn create_int_array(values: &[i64]) -> JsonNode

// 从 f32 切片创建数组
pub fn create_float_array(values: &[f32]) -> JsonNode

// 从 f64 切片创建数组
pub fn create_double_array(values: &[f64]) -> JsonNode

// 从字符串切片创建数组
pub fn create_string_array(values: &[&str]) -> JsonNode
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() {
    // 创建整数数组
    let int_array = JsonNode::create_int_array(&[1, 2, 3, 4, 5]);
    
    // 创建浮点数数组
    let float_array = JsonNode::create_double_array(&[1.1, 2.2, 3.3]);
    
    // 创建字符串数组
    let string_array = JsonNode::create_string_array(&["apple", "banana", "cherry"]);
    
    println!("{:?}", int_array);
    println!("{:?}", float_array);
    println!("{:?}", string_array);
}
```

### 4. 操作功能 (Manipulation)

对 `JsonNode` 对象树进行添加、删除、替换、插入等操作。

#### 添加元素

**方法签名**
```rust
// 添加元素到数组
pub fn add_item_to_array(&mut self, item: JsonNode) -> Result<()>

// 添加元素到对象
pub fn add_item_to_object(&mut self, key: impl Into<String>, item: JsonNode) -> Result<()>

// 添加字符串到对象
pub fn add_string_to_object(&mut self, key: impl Into<String>, value: &str) -> Result<()>

// 添加数字到对象
pub fn add_number_to_object(&mut self, key: impl Into<String>, value: f64) -> Result<()>

// 添加布尔值到对象
pub fn add_bool_to_object(&mut self, key: impl Into<String>, value: bool) -> Result<()>
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数组并添加元素
    let mut array = JsonNode::new_array();
    array.add_item_to_array(JsonNode::Number(1.0))?;
    array.add_item_to_array(JsonNode::String("hello"))?;
    
    // 创建对象并添加元素
    let mut object = JsonNode::new_object();
    object.add_string_to_object("name", "John")?;
    object.add_number_to_object("age", 30.0)?;
    object.add_bool_to_object("is_student", false)?;
    
    println!("{:?}", array);
    println!("{:?}", object);
    
    Ok(())
}
```

#### 删除元素

**方法签名**
```rust
// 从数组中删除元素
pub fn delete_item_from_array(&mut self, index: usize) -> Result<()>

// 从对象中删除元素
pub fn delete_item_from_object(&mut self, key: &str) -> Result<()>
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数组
    let mut array = JsonNode::create_array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    // 删除第二个元素
    array.delete_item_from_array(1)?;
    
    // 创建对象
    let mut object = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    // 删除 age 字段
    object.delete_item_from_object("age")?;
    
    println!("{:?}", array);
    println!("{:?}", object);
    
    Ok(())
}
```

#### 替换元素

**方法签名**
```rust
// 替换数组中的元素
pub fn replace_item_in_array(&mut self, index: usize, new_item: JsonNode) -> Result<()>

// 替换对象中的元素
pub fn replace_item_in_object(&mut self, key: &str, new_item: JsonNode) -> Result<()>
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数组
    let mut array = JsonNode::create_array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    // 替换第二个元素
    array.replace_item_in_array(1, JsonNode::Number(20.0))?;
    
    // 创建对象
    let mut object = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    // 替换 age 字段
    object.replace_item_in_object("age", JsonNode::Number(35.0))?;
    
    println!("{:?}", array);
    println!("{:?}", object);
    
    Ok(())
}
```

#### 插入元素

**方法签名**
```rust
// 在数组指定位置插入元素
pub fn insert_item_in_array(&mut self, index: usize, item: JsonNode) -> Result<()>
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut array = JsonNode::create_array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(3.0),
    ]);
    
    // 在索引 1 处插入元素
    array.insert_item_in_array(1, JsonNode::Number(2.0))?;
    
    println!("{:?}", array);
    
    Ok(())
}
```

#### 排序对象

**方法签名**
```rust
// 对对象的键进行排序
pub fn sort_object(&mut self, case_sensitive: bool) -> Result<()>
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut object = JsonNode::create_object(vec![
        ("zebra".to_string(), JsonNode::String("Z".to_string())),
        ("apple".to_string(), JsonNode::String("A".to_string())),
        ("banana".to_string(), JsonNode::String("B".to_string())),
    ]);
    
    // 排序（不区分大小写）
    object.sort_object(false)?;
    
    println!("{:?}", object);
    
    Ok(())
}
```

### 5. 查询功能 (Query)

从 `JsonNode` 对象树中获取数据。

#### 数组查询

**函数签名**
```rust
// 获取数组大小
pub fn get_array_size(node: &JsonNode) -> Result<usize>

// 获取数组中的元素
pub fn get_array_item<'a>(node: &'a JsonNode, index: usize) -> Result<&'a JsonNode>
```

**使用示例**
```rust
use lx_json::{JsonNode, get_array_size, get_array_item};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let array = JsonNode::create_array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    // 获取数组大小
    let size = get_array_size(&array)?;
    println!("Array size: {}", size);
    
    // 获取第一个元素
    let item = get_array_item(&array, 0)?;
    println!("First item: {:?}", item);
    
    Ok(())
}
```

#### 对象查询

**函数签名**
```rust
// 获取对象中的值（不区分大小写）
pub fn get_object_item<'a>(node: &'a JsonNode, key: &str) -> Result<&'a JsonNode>

// 获取对象中的值（区分大小写）
pub fn get_object_item_case_sensitive<'a>(node: &'a JsonNode, key: &str) -> Result<&'a JsonNode>

// 检查对象中是否存在某个键
pub fn has_object_item(node: &JsonNode, key: &str) -> Result<bool>
```

**使用示例**
```rust
use lx_json::{JsonNode, get_object_item, has_object_item};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let object = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    // 获取 name 字段
    if let Some(name) = get_object_item(&object, "name")?.as_string() {
        println!("Name: {}", name);
    }
    
    // 检查是否存在 age 字段
    let has_age = has_object_item(&object, "age")?;
    println!("Has age: {}", has_age);
    
    Ok(())
}
```

#### 值获取

**函数签名**
```rust
// 获取字符串值
pub fn get_string_value(node: &JsonNode) -> Result<&str>

// 获取数字值
pub fn get_number_value(node: &JsonNode) -> Result<f64>
```

**使用示例**
```rust
use lx_json::{get_string_value, get_number_value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let string_node = JsonNode::String("hello".to_string());
    let number_node = JsonNode::Number(42.0);
    
    let s = get_string_value(&string_node)?;
    println!("String: {}", s);
    
    let n = get_number_value(&number_node)?;
    println!("Number: {}", n);
    
    Ok(())
}
```

#### JSON Pointer (RFC 6901)

**函数签名**
```rust
// 使用 JSON Pointer 获取值（不可变）
pub fn get_pointer<'a>(node: &'a JsonNode, pointer: &str) -> Result<&'a JsonNode>

// 使用 JSON Pointer 获取值（可变）
pub fn get_pointer_mut<'a>(node: &'a mut JsonNode, pointer: &str) -> Result<&'a mut JsonNode>
```

**使用示例**
```rust
use lx_json::{parse, get_pointer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"user":{"name":"John","age":30}}"#;
    let node = parse(json)?;
    
    // 使用 JSON Pointer 获取嵌套值
    let name = get_pointer(&node, "/user/name")?;
    println!("{:?}", name);
    
    let age = get_pointer(&node, "/user/age")?;
    println!("{:?}", age);
    
    Ok(())
}
```

### 6. 类型检查 (Type Checking)

检查 `JsonNode` 的类型。

**方法签名**
```rust
pub fn is_null(&self) -> bool
pub fn is_bool(&self) -> bool
pub fn is_number(&self) -> bool
pub fn is_string(&self) -> bool
pub fn is_array(&self) -> bool
pub fn is_object(&self) -> bool
pub fn is_true(&self) -> bool
pub fn is_false(&self) -> bool
pub fn is_invalid(&self) -> bool
pub fn is_raw(&self) -> bool
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() {
    let null = JsonNode::new_null();
    let bool_val = JsonNode::new_bool(true);
    let number = JsonNode::new_number(42.0);
    let string = JsonNode::new_string("hello");
    let array = JsonNode::new_array();
    let object = JsonNode::new_object();
    
    println!("null.is_null() = {}", null.is_null());        // true
    println!("bool_val.is_bool() = {}", bool_val.is_bool()); // true
    println!("bool_val.is_true() = {}", bool_val.is_true()); // true
    println!("number.is_number() = {}", number.is_number()); // true
    println!("string.is_string() = {}", string.is_string()); // true
    println!("array.is_array() = {}", array.is_array());     // true
    println!("object.is_object() = {}", object.is_object()); // true
}
```

#### 类型转换

**方法签名**
```rust
pub fn as_string(&self) -> Option<&str>
pub fn as_number(&self) -> Option<f64>
pub fn as_bool(&self) -> Option<bool>
pub fn as_array(&self) -> Option<&Vec<JsonNode>>
pub fn as_object(&self) -> Option<&Vec<(String, JsonNode)>>
```

**使用示例**
```rust
use lx_json::JsonNode;

fn main() {
    let string = JsonNode::String("hello".to_string());
    let number = JsonNode::Number(42.0);
    let bool_val = JsonNode::new_bool(true);
    
    if let Some(s) = string.as_string() {
        println!("String: {}", s);
    }
    
    if let Some(n) = number.as_number() {
        println!("Number: {}", n);
    }
    
    if let Some(b) = bool_val.as_bool() {
        println!("Bool: {}", b);
    }
}
```

### 7. 工具功能 (Utilities)

#### 复制

**函数签名**
```rust
pub fn duplicate(node: &JsonNode, recurse: bool) -> JsonNode
```

**参数说明**
- `node`: 要复制的节点
- `recurse`: 如果为 true，递归复制所有子节点（深拷贝）；如果为 false，只复制当前节点，容器变为空（浅拷贝）

**返回值**
- 新的 JsonNode

**使用示例**
```rust
use lx_json::{JsonNode, duplicate};

fn main() {
    let original = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    // 深拷贝
    let deep_copy = duplicate(&original, true);
    
    // 浅拷贝
    let shallow_copy = duplicate(&original, false);
    
    println!("{:?}", deep_copy);
    println!("{:?}", shallow_copy);
}
```

#### 比较

**函数签名**
```rust
pub fn compare(a: &JsonNode, b: &JsonNode, case_sensitive: bool) -> bool
```

**参数说明**
- `a`: 第一个节点
- `b`: 第二个节点
- `case_sensitive`: 如果为 true，字符串比较区分大小写；如果为 false，不区分大小写

**返回值**
- 如果相等返回 true，否则返回 false

**使用示例**
```rust
use lx_json::{JsonNode, compare};

fn main() {
    let node1 = JsonNode::String("hello".to_string());
    let node2 = JsonNode::String("HELLO".to_string());
    
    // 区分大小写比较
    let case_sensitive = compare(&node1, &node2, true);
    println!("Case sensitive: {}", case_sensitive);  // false
    
    // 不区分大小写比较
    let case_insensitive = compare(&node1, &node2, false);
    println!("Case insensitive: {}", case_insensitive);  // true
}
```

### 8. JSON Patch (RFC 6902)

JSON Patch 是一种用于描述对 JSON 文档进行修改的格式（RFC 6902）。

#### 生成补丁

**函数签名**
```rust
pub fn generate_patches(from: &JsonNode, to: &JsonNode, case_sensitive: bool) -> Result<JsonNode>
```

**参数说明**
- `from`: 原始 JSON 值
- `to`: 修改后的 JSON 值
- `case_sensitive`: 是否区分大小写

**返回值**
- `Ok(JsonNode)`: 包含补丁操作的 JSON 数组
- `Err(JsonError)`: 错误

**使用示例**
```rust
use lx_json::{JsonNode, generate_patches};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let from = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let to = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let patches = generate_patches(&from, &to, true)?;
    println!("{:?}", patches);
    
    Ok(())
}
```

#### 应用补丁

**函数签名**
```rust
pub fn apply_patches(target: &mut JsonNode, patches: &JsonNode, case_sensitive: bool) -> Result<()>
```

**参数说明**
- `target`: 目标 JSON 值（可变）
- `patches`: 补丁文档（补丁操作数组）
- `case_sensitive`: 是否区分大小写

**返回值**
- `Ok(())`: 成功
- `Err(JsonError)`: 错误

**使用示例**
```rust
use lx_json::{JsonNode, apply_patches};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut target = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patches = JsonNode::create_array(vec![
        JsonNode::create_object(vec![
            ("op".to_string(), JsonNode::String("replace".to_string())),
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("Jane".to_string())),
        ]),
    ]);
    
    apply_patches(&mut target, &patches, true)?;
    println!("{:?}", target);
    
    Ok(())
}
```

#### 添加补丁到数组

**函数签名**
```rust
pub fn add_patch_to_array(
    patches: &mut JsonNode,
    op: &str,
    path: &str,
    value: &JsonNode
) -> Result<()>
```

**参数说明**
- `patches`: 补丁数组（可变）
- `op`: 操作类型（"add", "remove", "replace", "move", "copy", "test"）
- `path`: JSON Pointer 路径
- `value`: 操作的值

**返回值**
- `Ok(())`: 成功
- `Err(JsonError)`: 错误

**使用示例**
```rust
use lx_json::{JsonNode, add_patch_to_array};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut patches = JsonNode::new_array();
    
    add_patch_to_array(
        &mut patches,
        "add",
        "/newField",
        &JsonNode::String("newValue".to_string())
    )?;
    
    println!("{:?}", patches);
    
    Ok(())
}
```

**支持的补丁操作**

| 操作 | 说明 | 参数 |
|------|------|------|
| "add" | 在指定位置添加值 | path, value |
| "remove" | 删除指定位置的值 | path |
| "replace" | 替换指定位置的值 | path, value |
| "move" | 将值从一个位置移动到另一个位置 | path, from |
| "copy" | 将值从一个位置复制到另一个位置 | path, from |
| "test" | 测试指定位置的值是否等于给定值 | path, value |

### 9. JSON Merge Patch (RFC 7396)

JSON Merge Patch 是一种更简单的 JSON 文档合并方式（RFC 7396）。

**函数签名**
```rust
pub fn merge_patch(target: &mut JsonNode, patch: &JsonNode, case_sensitive: bool) -> Result<()>
```

**参数说明**
- `target`: 目标 JSON 值（可变）
- `patch`: 合并补丁文档
- `case_sensitive`: 是否区分大小写

**返回值**
- `Ok(())`: 成功
- `Err(JsonError)`: 错误

**使用示例**
```rust
use lx_json::{JsonNode, merge_patch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut target = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let patch = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
        ("city".to_string(), JsonNode::String("New York".to_string())),
    ]);
    
    merge_patch(&mut target, &patch, true)?;
    println!("{:?}", target);
    
    Ok(())
}
```

**合并规则**
- 如果补丁中的值是 `null`，则从目标中删除该字段
- 如果两者都是对象，则递归合并
- 对于其他情况，目标值被补丁值替换

## 完整使用示例

### 示例 1: 解析复杂 JSON

```rust
use lx_json::{parse, print};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
    {
        "name": "John",
        "age": 30,
        "is_student": false,
        "hobbies": ["reading", "gaming", "coding"],
        "address": {
            "city": "New York",
            "country": "USA",
            "zip": "10001"
        },
        "scores": [95, 88, 92, 87]
    }
    "#;
    
    let node = parse(json)?;
    
    // 访问嵌套对象
    if let Some(address) = node.get("address") {
        if let Some(city) = address.get("city").and_then(|v| v.as_string()) {
            println!("City: {}", city);
        }
    }
    
    // 访问数组
    if let Some(hobbies) = node.get("hobbies").and_then(|v| v.as_array()) {
        println!("Hobbies:");
        for hobby in hobbies {
            if let Some(s) = hobby.as_string() {
                println!("  - {}", s);
            }
        }
    }
    
    // 计算分数平均值
    if let Some(scores) = node.get("scores").and_then(|v| v.as_array()) {
        let sum: f64 = scores.iter()
            .filter_map(|v| v.as_number())
            .sum();
        let avg = sum / scores.len() as f64;
        println!("Average score: {:.2}", avg);
    }
    
    Ok(())
}
```

### 示例 2: 创建复杂 JSON

```rust
use lx_json::{JsonNode, print};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建根对象
    let mut root = JsonNode::new_object();
    
    // 添加基本类型
    root.add_string_to_object("name", "John")?;
    root.add_number_to_object("age", 30.0)?;
    root.add_bool_to_object("is_student", false)?;
    
    // 添加数组
    let mut hobbies = JsonNode::new_array();
    hobbies.add_item_to_array(JsonNode::String("reading".to_string()))?;
    hobbies.add_item_to_array(JsonNode::String("gaming".to_string()))?;
    hobbies.add_item_to_array(JsonNode::String("coding".to_string()))?;
    root.add_item_to_object("hobbies", hobbies)?;
    
    // 添加嵌套对象
    let mut address = JsonNode::new_object();
    address.add_string_to_object("city", "New York")?;
    address.add_string_to_object("country", "USA")?;
    address.add_string_to_object("zip", "10001")?;
    root.add_item_to_object("address", address)?;
    
    // 打印格式化的 JSON
    let output = print(&root);
    println!("{}", output);
    
    Ok(())
}
```

### 示例 3: 使用 JSON Pointer 查询

```rust
use lx_json::{parse, get_pointer, get_pointer_mut};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
    {
        "user": {
            "name": "John",
            "profile": {
                "email": "john@example.com",
                "age": 30
            }
        },
        "posts": [
            {"id": 1, "title": "First Post"},
            {"id": 2, "title": "Second Post"}
        ]
    }
    "#;
    
    let mut node = parse(json)?;
    
    // 获取用户邮箱
    let email = get_pointer(&node, "/user/profile/email")?;
    println!("Email: {:?}", email);
    
    // 获取第一篇文章的标题
    let title = get_pointer(&node, "/posts/0/title")?;
    println!("First post title: {:?}", title);
    
    // 修改用户的年龄
    if let Some(age) = get_pointer_mut(&mut node, "/user/profile/age") {
        *age = JsonNode::Number(31.0);
    }
    
    println!("{:?}", node);
    
    Ok(())
}
```

### 示例 4: 使用 JSON Patch

```rust
use lx_json::{JsonNode, generate_patches, apply_patches, print};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 原始数据
    let original = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
        ("city".to_string(), JsonNode::String("Boston".to_string())),
    ]);
    
    // 修改后的数据
    let modified = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
        ("age".to_string(), JsonNode::Number(31.0)),
        ("city".to_string(), JsonNode::String("New York".to_string())),
    ]);
    
    // 生成补丁
    let patches = generate_patches(&original, &modified, true)?;
    println!("Patches:");
    println!("{}", print(&patches));
    
    // 应用补丁到另一个对象
    let mut target = original.clone();
    apply_patches(&mut target, &patches, true)?;
    
    println!("After applying patches:");
    println!("{}", print(&target));
    
    Ok(())
}
```

### 示例 5: 使用 JSON Merge Patch

```rust
use lx_json::{JsonNode, merge_patch, print};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 原始数据
    let mut target = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
        ("city".to_string(), JsonNode::String("Boston".to_string())),
        ("email".to_string(), JsonNode::String("john@old.com".to_string())),
    ]);
    
    // 合并补丁（修改 name 和 city，删除 email，添加 country）
    let patch = JsonNode::create_object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
        ("city".to_string(), JsonNode::String("New York".to_string())),
        ("email".to_string(), JsonNode::Null),  // null 表示删除
        ("country".to_string(), JsonNode::String("USA".to_string())),
    ]);
    
    merge_patch(&mut target, &patch, true)?;
    
    println!("After merge:");
    println!("{}", print(&target));
    
    Ok(())
}
```

## 错误处理

LX-json 使用 `Result<T, JsonError>` 模式进行错误处理。`JsonError` 枚举包含所有可能的错误类型。

### 错误类型

| 错误类型 | 说明 |
|---------|------|
| `UnexpectedEndOfInput` | 意外的输入结束 |
| `UnexpectedCharacter` | 遇到意外的字符 |
| `InvalidNumber` | 无效的数字格式 |
| `InvalidString` | 无效的字符串格式 |
| `SyntaxError` | 语法错误 |
| `NestingLimitExceeded` | 超过嵌套深度限制 |
| `InvalidEscapeSequence` | 无效的转义序列 |
| `DuplicateKey` | 重复的对象键 |
| `TrailingComma` | 数组或对象中有尾随逗号 |
| `ExpectedNullTerminator` | 预期 null 终止符 |
| `TrailingCharacters` | JSON 文档后有尾随字符 |
| `InvalidType` | 操作的类型无效 |
| `IndexOutOfBounds` | 数组索引越界 |
| `KeyNotFound` | 键未找到 |
| `InvalidPatchOperation` | 无效的补丁操作 |
| `PatchPathNotFound` | 补丁路径未找到 |
| `PatchTestFailed` | 补丁测试失败 |
| `MergeError` | 合并错误 |

### 错误处理示例

```rust
use lx_json::{parse, JsonError};

fn main() {
    let invalid_json = r#"{"key": "value"#;  // 缺少闭合括号
    
    match parse(invalid_json) {
        Ok(node) => println!("{:?}", node),
        Err(e) => {
            eprintln!("解析错误:");
            eprintln!("  {}", e);
            
            // 根据错误类型进行不同处理
            match e {
                JsonError::UnexpectedEndOfInput { position, expected } => {
                    eprintln!("  位置 {}: 预期 '{}', 但输入已结束", position, expected);
                }
                JsonError::UnexpectedCharacter { character, position, expected } => {
                    eprintln!("  位置 {}: 预期 '{}', 但找到 '{}'", position, expected, character);
                }
                _ => {
                    eprintln!("  其他错误");
                }
            }
        }
    }
}
```

## 性能优化建议

### 1. 选择合适的序列化方法

```rust
use lx_json::{parse, print, print_unformatted};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"key":"value"}"#;
    let node = parse(json)?;
    
    // 需要可读性时使用 print
    let formatted = print(&node);
    
    // 需要性能时使用 print_unformatted
    let compact = print_unformatted(&node);
    
    Ok(())
}
```

### 2. 重用 JsonNode

```rust
use lx_json::JsonNode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut node = JsonNode::new_object();
    
    // 重用同一个对象
    for i in 0..1000 {
        node.add_number_to_object("value", i as f64)?;
        // ... 处理 ...
        node.delete_item_from_object("value")?;
    }
    
    Ok(())
}
```

### 3. 使用浅拷贝减少内存分配

```rust
use lx_json::{JsonNode, duplicate};

fn main() {
    let large_node = JsonNode::create_array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        // ... 更多元素
    ]);
    
    // 只需要容器结构时使用浅拷贝
    let shallow = duplicate(&large_node, false);
    
    // 需要完整副本时使用深拷贝
    let deep = duplicate(&large_node, true);
}
```

## 注意事项

### 字符编码

LX-json 假设输入为 UTF-8 编码，不处理字符编码转换。

### 浮点数精度

LX-json 使用 `f64` 存储数字，可能存在精度损失。例如：

```rust
use lx_json::{parse, print};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"0.1 + 0.2"#;  // 注意：这不是有效的 JSON
    let node = parse("0.3")?;
    println!("{:?}", node);  // 0.3
    
    // 浮点数精度问题
    let n = 0.1 + 0.2;
    let node = JsonNode::Number(n);
    println!("{}", print(&node));  // 可能输出 0.30000000000000004
    
    Ok(())
}
```

### 线程安全

LX-json 本身不是线程安全的。在多线程环境中使用时，需要适当的同步机制（如 `Mutex` 或 `Arc`）。

### 内存管理

Rust 的所有权系统自动管理内存。`JsonNode` 实现了 `Drop` trait，会自动释放资源。但在某些情况下，显式删除大对象可能有助于及时释放内存：

```rust
use lx_json::JsonNode;

fn main() {
    let large_node = JsonNode::create_array(vec![
        // ... 大量数据
    ]);
    
    // large_node 离开作用域时会自动释放
    // 也可以使用 std::mem::drop 显式释放
    drop(large_node);
}
```

### 嵌套深度限制

默认最大嵌套深度为 1000。可以通过 `ParseOptions` 调整：

```rust
use lx_json::{parse_with_opts, ParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"a":{"b":{"c":{...}}}}"#;
    
    let options = ParseOptions {
        nesting_limit: 2000,  // 增加嵌套深度限制
        require_null_terminated: false,
    };
    
    let node = parse_with_opts(json, options)?;
    
    Ok(())
}
```

## 与 cJSON 的对比

| 特性 | cJSON (C) | LX-json (Rust) |
|------|-----------|----------------|
| 类型系统 | 结构体 + 类型标志 | 枚举（编译时类型安全） |
| 内存管理 | 手动管理 | 自动管理（所有权系统） |
| 错误处理 | 返回 NULL 和错误指针 | Result<T, JsonError> |
| 线程安全 | 不安全 | 需要外部同步 |
| 字符串处理 | C 字符串 | Rust String（UTF-8） |
| 空值检查 | 需要显式检查 | Option<T> 类型 |
| API 风格 | C 风格 | Rust 惯用风格 |

## API 快速参考

### 解析函数
- `parse(input: &str) -> Result<JsonNode>`
- `parse_with_length(input: &str, length: usize) -> Result<JsonNode>`
- `parse_with_opts(input: &str, options: ParseOptions) -> Result<JsonNode>`

### 序列化函数
- `print(node: &JsonNode) -> String`
- `print_unformatted(node: &JsonNode) -> String`
- `print_buffered(node: &JsonNode, prebuffer: i32, fmt: bool) -> String`
- `print_preallocated(node: &JsonNode, buffer: &mut String, format: bool) -> Result<()>`
- `minify(json: &str) -> String`

### 创建函数
- `JsonNode::new_null() -> JsonNode`
- `JsonNode::new_bool(b: bool) -> JsonNode`
- `JsonNode::new_true() -> JsonNode`
- `JsonNode::new_false() -> JsonNode`
- `JsonNode::new_number(n: f64) -> JsonNode`
- `JsonNode::new_string(s: impl Into<String>) -> JsonNode`
- `JsonNode::new_array() -> JsonNode`
- `JsonNode::create_array(values: Vec<JsonNode>) -> JsonNode`
- `JsonNode::new_object() -> JsonNode`
- `JsonNode::create_object(pairs: Vec<(String, JsonNode)>) -> JsonNode`

### 查询函数
- `get_array_size(node: &JsonNode) -> Result<usize>`
- `get_array_item(node: &JsonNode, index: usize) -> Result<&JsonNode>`
- `get_object_item(node: &JsonNode, key: &str) -> Result<&JsonNode>`
- `get_object_item_case_sensitive(node: &JsonNode, key: &str) -> Result<&JsonNode>`
- `has_object_item(node: &JsonNode, key: &str) -> Result<bool>`
- `get_string_value(node: &JsonNode) -> Result<&str>`
- `get_number_value(node: &JsonNode) -> Result<f64>`
- `get_pointer(node: &JsonNode, pointer: &str) -> Result<&JsonNode>`
- `get_pointer_mut(node: &mut JsonNode, pointer: &str) -> Result<&mut JsonNode>`

### 补丁函数
- `generate_patches(from: &JsonNode, to: &JsonNode, case_sensitive: bool) -> Result<JsonNode>`
- `apply_patches(target: &mut JsonNode, patches: &JsonNode, case_sensitive: bool) -> Result<()>`
- `add_patch_to_array(patches: &mut JsonNode, op: &str, path: &str, value: &JsonNode) -> Result<()>`

### 合并函数
- `merge_patch(target: &mut JsonNode, patch: &JsonNode, case_sensitive: bool) -> Result<()>`

### 工具函数
- `duplicate(node: &JsonNode, recurse: bool) -> JsonNode`
- `compare(a: &JsonNode, b: &JsonNode, case_sensitive: bool) -> bool`

## 测试

项目包含全面的测试套件，位于 `test.rs` 文件中（2500+ 行）。

运行测试：

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_parse_object

# 运行测试并显示输出
cargo test -- --nocapture

# 运行测试并生成覆盖率报告
cargo test -- --nocapture
```

## 许可证

MIT License - 详见 LICENSE 文件

## 联系方式

- 项目主页: https://github.com/lingxi/lx-json
- 问题反馈: GitHub Issues

## 版本历史

### 0.1.0 (当前版本)
- 初始版本发布
- 完整的 JSON 解析和序列化支持
- JSON Pointer (RFC 6901)
- JSON Patch (RFC 6902)
- JSON Merge Patch (RFC 7396)
- 完整的错误处理
- 全面的测试覆盖

---

**注意**: 本手册基于 LX-json 0.1.0 版本编写。如需最新信息，请访问官方仓库。