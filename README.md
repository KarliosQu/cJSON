# LX-json

一个使用 Rust 语言实现的轻量级 JSON 解析器库，参考了 cJSON 项目的设计理念。

## 项目简介

LX-json 是一个简洁、高效的 JSON 解析器，提供了类型安全的 JSON 解析和序列化功能。该项目使用纯 Rust 实现，无外部依赖，符合 Rust 惯用风格。

### 特性

- **类型安全**: 使用 Rust 的 enum 类型系统，确保编译时的类型安全
- **完整支持**: 支持所有标准 JSON 类型（null, boolean, number, string, array, object）
- **无依赖**: 纯 Rust 实现，无外部依赖
- **错误处理**: 完善的错误处理机制，提供详细的错误信息
- **高性能**: 高效的解析算法，优化的内存使用
- **安全**: 遵守 Rust 的所有权和借用规则，无 unsafe 代码
- **可配置**: 支持自定义解析选项（嵌套深度限制等）

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
lx-json = "0.1.0"
```

## 使用示例

### 基本解析

```rust
use lx_json::{parse, JsonNode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name": "John", "age": 30}"#;
    let node = parse(json)?;
    
    if let Some(name) = node.get("name").and_then(|v| v.as_string()) {
        println!("Name: {}", name);
    }
    
    if let Some(age) = node.get("age").and_then(|v| v.as_number()) {
        println!("Age: {}", age);
    }
    
    Ok(())
}
```

### 解析复杂数据结构

```rust
use lx_json::{parse, JsonNode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{
        "name": "John",
        "age": 30,
        "is_student": false,
        "hobbies": ["reading", "gaming"],
        "address": {
            "city": "New York",
            "country": "USA"
        }
    }"#;
    
    let node = parse(json)?;
    
    // 访问嵌套对象
    if let Some(address) = node.get("address") {
        if let Some(city) = address.get("city").and_then(|v| v.as_string()) {
            println!("City: {}", city);
        }
    }
    
    // 访问数组
    if let Some(hobbies) = node.get("hobbies").and_then(|v| v.as_array()) {
        for hobby in hobbies {
            if let Some(s) = hobby.as_string() {
                println!("Hobby: {}", s);
            }
        }
    }
    
    Ok(())
}
```

### 创建 JSON 值

```rust
use lx_json::JsonNode;

fn main() {
    // 创建各种类型的 JSON 值
    let null = JsonNode::new_null();
    let bool_val = JsonNode::new_bool(true);
    let number = JsonNode::new_number(42.0);
    let string = JsonNode::new_string("hello");
    
    // 创建数组
    let array = JsonNode::create_array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    // 创建对象
    let object = JsonNode::create_object(vec![
        ("key".to_string(), JsonNode::String("value".to_string())),
        ("number".to_string(), JsonNode::Number(42.0)),
    ]);
    
    println!("Array: {}", array);
    println!("Object: {}", object);
}
```

### 序列化

```rust
use lx_json::{parse, print, print_unformatted};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name": "John", "age": 30}"#;
    let node = parse(json)?;
    
    // 格式化输出
    let formatted = print(&node);
    println!("Formatted:\n{}", formatted);
    
    // 紧凑输出
    let unformatted = print_unformatted(&node);
    println!("Unformatted: {}", unformatted);
    
    Ok(())
}
```

### 带选项的解析

```rust
use lx_json::{parse_with_opts, ParseOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name": "John"}"#;
    
    // 设置嵌套深度限制
    let opts = ParseOptions::new().with_nesting_limit(100);
    let node = parse_with_opts(json, opts)?;
    
    println!("Parsed: {}", node);
    
    // 要求 null 终止符
    let opts = ParseOptions::new().with_null_terminated(true);
    let result = parse_with_opts("null extra", opts);
    assert!(result.is_err()); // 应该失败
    
    Ok(())
}
```

### JSON 压缩

```rust
use lx_json::minify;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{
        "name": "John",
        "age": 30
    }"#;
    
    let minified = minify(json)?;
    println!("Minified: {}", minified);
    // 输出: {"name":"John","age":30}
    
    Ok(())
}
```

## API 文档

### 核心函数

- `parse(json: &str) -> Result<JsonNode>` - 基本解析函数
- `parse_with_length(json: &str, length: usize) -> Result<JsonNode>` - 带长度限制的解析
- `parse_with_opts(json: &str, opts: ParseOptions) -> Result<JsonNode>` - 带选项的解析
- `print(node: &JsonNode) -> String` - 格式化输出
- `print_unformatted(node: &JsonNode) -> String` - 紧凑输出
- `minify(json: &str) -> Result<String>` - JSON 压缩

### JsonNode 方法

- `is_null()`, `is_bool()`, `is_number()`, `is_string()`, `is_array()`, `is_object()` - 类型检查
- `as_string()`, `as_number()`, `as_bool()`, `as_array()`, `as_object()` - 值访问
- `get(key: &str) -> Option<&JsonNode>` - 获取对象中的值
- `get_at(index: usize) -> Option<&JsonNode>` - 获取数组中的值
- `len()` - 获取数组或对象的长度
- `is_empty()` - 检查是否为空

## 编译和测试

```bash
# 编译项目
cargo build

# 运行测试
cargo test

# 代码质量检查
cargo clippy

# 生成文档
cargo doc --open
```

## 验收测试

详细的验收说明请参阅 [ACCEPTANCE.md](ACCEPTANCE.md)。

## 许可证

MIT License

## 作者

灵犀 (Lingxi)

## 版本

当前版本: 0.1.0