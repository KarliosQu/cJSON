# LX-json 验收文档

## 1. 项目概述

LX-json 是一个使用 Rust 语言实现的轻量级 JSON 解析器库，参考了 cJSON 项目的设计理念。该库提供了类型安全的 JSON 解析和序列化功能，支持所有标准 JSON 类型。

## 2. 编译验证

### 2.1 编译项目

```bash
cargo build
```

**预期结果**: 编译成功，无错误和警告。

### 2.2 编译优化版本

```bash
cargo build --release
```

**预期结果**: 编译成功，生成优化的可执行文件。

### 2.3 检查代码质量

```bash
cargo clippy
```

**预期结果**: 无 clippy 警告。

## 3. 单元测试

### 3.1 运行所有测试

```bash
cargo test
```

**预期结果**: 所有测试通过。

### 3.2 运行测试并显示输出

```bash
cargo test -- --nocapture
```

**预期结果**: 所有测试通过，并显示测试输出。

## 4. 功能验收

### 4.1 基本解析功能

#### 测试用例 1: 解析 null

```rust
use lx_json::parse;

let result = parse("null");
assert_eq!(result.unwrap(), lx_json::JsonNode::Null);
```

**验收标准**: 能够正确解析 null 值。

#### 测试用例 2: 解析布尔值

```rust
use lx_json::parse;

let true_result = parse("true");
let false_result = parse("false");

assert_eq!(true_result.unwrap(), lx_json::JsonNode::Bool(true));
assert_eq!(false_result.unwrap(), lx_json::JsonNode::Bool(false));
```

**验收标准**: 能够正确解析 true 和 false 布尔值。

#### 测试用例 3: 解析数字

```rust
use lx_json::parse;

// 整数
assert_eq!(parse("42").unwrap(), lx_json::JsonNode::Number(42.0));
assert_eq!(parse("-42").unwrap(), lx_json::JsonNode::Number(-42.0));

// 浮点数
assert_eq!(parse("3.14").unwrap(), lx_json::JsonNode::Number(3.14));
assert_eq!(parse("-3.14").unwrap(), lx_json::JsonNode::Number(-3.14));

// 科学计数法
assert_eq!(parse("1e5").unwrap(), lx_json::JsonNode::Number(100000.0));
assert_eq!(parse("1.5e-3").unwrap(), lx_json::JsonNode::Number(0.0015));
```

**验收标准**: 能够正确解析整数、浮点数和科学计数法表示的数字。

#### 测试用例 4: 解析字符串

```rust
use lx_json::parse;

// 简单字符串
assert_eq!(
    parse(r#""hello""#).unwrap(),
    lx_json::JsonNode::String("hello".to_string())
);

// 带转义字符的字符串
assert_eq!(
    parse(r#""hello\nworld""#).unwrap(),
    lx_json::JsonNode::String("hello\nworld".to_string())
);

// 带引号的字符串
assert_eq!(
    parse(r#""\"quoted\"""#).unwrap(),
    lx_json::JsonNode::String("\"quoted\"".to_string())
);
```

**验收标准**: 能够正确解析字符串，包括转义字符。

#### 测试用例 5: 解析数组

```rust
use lx_json::parse;

// 空数组
assert_eq!(
    parse("[]").unwrap(),
    lx_json::JsonNode::Array(vec![])
);

// 简单数组
assert_eq!(
    parse("[1, 2, 3]").unwrap(),
    lx_json::JsonNode::Array(vec![
        lx_json::JsonNode::Number(1.0),
        lx_json::JsonNode::Number(2.0),
        lx_json::JsonNode::Number(3.0),
    ])
);

// 嵌套数组
assert_eq!(
    parse("[[1], [2], [3]]").unwrap(),
    lx_json::JsonNode::Array(vec![
        lx_json::JsonNode::Array(vec![lx_json::JsonNode::Number(1.0)]),
        lx_json::JsonNode::Array(vec![lx_json::JsonNode::Number(2.0)]),
        lx_json::JsonNode::Array(vec![lx_json::JsonNode::Number(3.0)]),
    ])
);
```

**验收标准**: 能够正确解析数组，包括嵌套数组。

#### 测试用例 6: 解析对象

```rust
use lx_json::parse;

// 空对象
assert_eq!(
    parse("{}").unwrap(),
    lx_json::JsonNode::Object(vec![])
);

// 简单对象
assert_eq!(
    parse(r#"{"key": "value"}"#).unwrap(),
    lx_json::JsonNode::Object(vec![(
        "key".to_string(),
        lx_json::JsonNode::String("value".to_string()),
    )])
);

// 嵌套对象
assert_eq!(
    parse(r#"{"outer": {"inner": "value"}}"#).unwrap(),
    lx_json::JsonNode::Object(vec![(
        "outer".to_string(),
        lx_json::JsonNode::Object(vec![(
            "inner".to_string(),
            lx_json::JsonNode::String("value".to_string()),
        )]),
    )])
);
```

**验收标准**: 能够正确解析对象，包括嵌套对象。

### 4.2 复杂 JSON 解析

#### 测试用例 7: 复杂嵌套结构

```rust
use lx_json::parse;

let json = r#"{
    "name": "John",
    "age": 30,
    "isStudent": false,
    "hobbies": ["reading", "gaming"],
    "address": {
        "city": "New York",
        "country": "USA"
    }
}"#;

let result = parse(json);
assert!(result.is_ok());

let node = result.unwrap();
assert_eq!(node.get("name").and_then(|v| v.as_string()), Some("John"));
assert_eq!(node.get("age").and_then(|v| v.as_number()), Some(30.0));
assert_eq!(node.get("isStudent").and_then(|v| v.as_bool()), Some(false));
```

**验收标准**: 能够正确解析复杂的嵌套 JSON 结构。

### 4.3 带选项的解析

#### 测试用例 8: 带长度限制的解析

```rust
use lx_json::parse_with_length;

// 截断的 JSON 应该解析失败
let result = parse_with_length(r#"{"key": "value"}"#, 10);
assert!(result.is_err());
```

**验收标准**: parse_with_length 函数能够正确处理长度限制。

#### 测试用例 9: 带选项的解析

```rust
use lx_json::{parse_with_opts, ParseOptions};

// 要求 null 终止符
let opts = ParseOptions::new().with_null_terminated(true);
let result = parse_with_opts("null extra", opts);
assert!(result.is_err());

// 不要求 null 终止符
let opts = ParseOptions::new().with_null_terminated(false);
let result = parse_with_opts("null extra", opts);
assert!(result.is_ok());
```

**验收标准**: parse_with_opts 函数能够正确处理各种选项。

#### 测试用例 10: 嵌套深度限制

```rust
use lx_json::{parse_with_opts, ParseOptions};

let opts = ParseOptions::new().with_nesting_limit(2);
let deep_json = "[[[1]]]"; // 嵌套深度 3
let result = parse_with_opts(deep_json, opts);
assert!(result.is_err());
```

**验收标准**: 能够正确限制嵌套深度，防止栈溢出。

### 4.4 序列化功能

#### 测试用例 11: 格式化输出

```rust
use lx_json::{parse, print};

let json = r#"[1, 2, 3]"#;
let node = parse(json).unwrap();
let output = print(&node);

assert!(output.contains("["));
assert!(output.contains("1"));
assert!(output.contains("2"));
assert!(output.contains("3"));
assert!(output.contains("\n")); // 应该有换行符
```

**验收标准**: print 函数能够生成格式化的 JSON 输出。

#### 测试用例 12: 紧凑输出

```rust
use lx_json::{parse, print_unformatted};

let json = r#"[1, 2, 3]"#;
let node = parse(json).unwrap();
let output = print_unformatted(&node);

assert_eq!(output, "[1,2,3]");
```

**验收标准**: print_unformatted 函数能够生成紧凑的 JSON 输出。

#### 测试用例 13: JSON 压缩

```rust
use lx_json::minify;

let json = r#"{
    "name": "John",
    "age": 30
}"#;
let minified = minify(json).unwrap();

assert_eq!(minified, r#"{"name":"John","age":30}"#);
```

**验收标准**: minify 函数能够正确压缩 JSON 字符串。

#### 测试用例 14: 往返测试

```rust
use lx_json::{parse, print_unformatted};

let original = r#"{"name": "John", "age": 30, "active": true}"#;
let parsed = parse(original).unwrap();
let serialized = print_unformatted(&parsed);
let reparsed = parse(&serialized).unwrap();

assert_eq!(parsed, reparsed);
```

**验收标准**: 解析和序列化的往返测试应该保持数据一致性。

### 4.5 错误处理

#### 测试用例 15: 无效 JSON

```rust
use lx_json::parse;

assert!(parse("{").is_err()); // 未闭合的对象
assert!(parse("[").is_err()); // 未闭合的数组
assert!(parse(r#""unclosed string"#).is_err()); // 未闭合的字符串
assert!(parse("123abc").is_err()); // 无效的数字后缀
```

**验收标准**: 能够正确识别和报告各种无效的 JSON 输入。

#### 测试用例 16: 尾随逗号

```rust
use lx_json::parse;

assert!(parse("[1, 2,]").is_err());
assert!(parse(r#"{"a": 1,}"#).is_err());
```

**验收标准**: 能够正确拒绝尾随逗号（符合 JSON 规范）。

#### 测试用例 17: 重复键

```rust
use lx_json::parse;

let result = parse(r#"{"key": 1, "key": 2}"#);
assert!(result.is_err());
```

**验收标准**: 能够正确检测并拒绝重复的对象键。

### 4.6 类型访问功能

#### 测试用例 18: 类型检查

```rust
use lx_json::{parse, JsonNode};

let node = parse("null").unwrap();
assert!(node.is_null());

let node = parse("true").unwrap();
assert!(node.is_bool());

let node = parse("42").unwrap();
assert!(node.is_number());

let node = parse(r#""hello""#).unwrap();
assert!(node.is_string());

let node = parse("[]").unwrap();
assert!(node.is_array());

let node = parse("{}").unwrap();
assert!(node.is_object());
```

**验收标准**: 类型检查函数能够正确识别各种 JSON 类型。

#### 测试用例 19: 值访问

```rust
use lx_json::{parse, JsonNode};

let node = parse(r#"{"key": "value"}"#).unwrap();
assert_eq!(node.get("key").and_then(|v| v.as_string()), Some("value"));

let node = parse(r#"[1, 2, 3]"#).unwrap();
assert_eq!(node.get_at(0), Some(&JsonNode::Number(1.0)));
assert_eq!(node.len(), 3);
```

**验收标准**: 值访问函数能够正确获取 JSON 值。

## 5. 验收标准总结

所有测试用例必须通过，且满足以下条件：

1. ✅ 代码能够通过 `cargo build` 无错误编译
2. ✅ 代码能够通过 `cargo clippy` 无警告检查
3. ✅ 所有单元测试通过（`cargo test`）
4. ✅ 支持所有 JSON 标准类型（null, boolean, number, string, array, object）
5. ✅ 提供基本解析函数 `parse()`
6. ✅ 提供带长度限制的解析函数 `parse_with_length()`
7. ✅ 提供带选项的解析函数 `parse_with_opts()`
8. ✅ 提供格式化输出函数 `print()`
9. ✅ 提供紧凑输出函数 `print_unformatted()`
10. ✅ 提供 JSON 压缩函数 `minify()`
11. ✅ 正确处理无效 JSON 输入，返回明确的错误信息
12. ✅ 代码符合 Rust 惯用风格，无 unsafe 代码
13. ✅ 完善的错误处理，使用 Result<T, E>
14. ✅ 遵守 Rust 的所有权和借用规则

## 6. 验收命令

执行以下命令完成验收：

```bash
# 编译检查
cargo build

# 代码质量检查
cargo clippy

# 运行所有测试
cargo test

# 运行测试并显示输出
cargo test -- --nocapture

# 运行文档测试
cargo test --doc
```

## 7. 验收通过标志

- 所有测试通过
- 无编译错误和警告
- 代码符合 Rust 惯用风格
- 文档完整且准确
- 所有功能按预期工作

---

**验收日期**: [待填写]
**验收人员**: [待填写]
**验收结果**: [待填写]