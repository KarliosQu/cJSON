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

### 4.6 JSON 生成功能

#### 测试用例 1: 动态添加元素到数组

```rust
use lx_json::JsonNode;

let mut arr = JsonNode::new_array();
arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
arr.add_item_to_array(JsonNode::String("hello".to_string())).unwrap();
arr.add_item_to_array(JsonNode::Bool(true)).unwrap();

assert_eq!(arr.len(), 3);
assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
assert_eq!(arr.get_at(1), Some(&JsonNode::String("hello".to_string())));
assert_eq!(arr.get_at(2), Some(&JsonNode::Bool(true)));
```

**验收标准**: 能够动态添加元素到数组，支持所有 JSON 类型。

#### 测试用例 2: 动态添加元素到对象

```rust
use lx_json::JsonNode;

let mut obj = JsonNode::new_object();
obj.add_item_to_object("name", JsonNode::String("Alice".to_string())).unwrap();
obj.add_item_to_object("age", JsonNode::Number(30.0)).unwrap();

assert_eq!(obj.len(), 2);
assert_eq!(obj.get("name"), Some(&JsonNode::String("Alice".to_string())));
assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
```

**验收标准**: 能够动态添加键值对到对象，支持所有 JSON 类型。

#### 测试用例 3: 便捷对象字段添加方法

```rust
use lx_json::JsonNode;

let mut obj = JsonNode::new_object();
obj.add_string_to_object("name", "Alice").unwrap();
obj.add_number_to_object("age", 30.0).unwrap();
obj.add_bool_to_object("active", true).unwrap();

assert_eq!(obj.len(), 3);
assert_eq!(obj.get("name"), Some(&JsonNode::String("Alice".to_string())));
assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
assert_eq!(obj.get("active"), Some(&JsonNode::Bool(true)));
```

**验收标准**: 便捷方法能够正确添加对应类型的字段到对象。

#### 测试用例 4: 批量创建数组

```rust
use lx_json::JsonNode;

// 批量创建整数数组
let int_arr = JsonNode::create_int_array(&[1, 2, 3, 4, 5]);
assert_eq!(int_arr.len(), 5);
assert_eq!(int_arr.get_at(0), Some(&JsonNode::Number(1.0)));

// 批量创建浮点数数组
let float_arr = JsonNode::create_float_array(&[1.1f32, 2.2f32, 3.3f32]);
assert_eq!(float_arr.len(), 3);
assert_eq!(float_arr.get_at(0), Some(&JsonNode::Number(1.1)));

// 批量创建双精度数组
let double_arr = JsonNode::create_double_array(&[1.5, 2.5, 3.5, 4.5]);
assert_eq!(double_arr.len(), 4);
assert_eq!(double_arr.get_at(0), Some(&JsonNode::Number(1.5)));

// 批量创建字符串数组
let str_arr = JsonNode::create_string_array(&["hello", "world", "rust"]);
assert_eq!(str_arr.len(), 3);
assert_eq!(str_arr.get_at(0), Some(&JsonNode::String("hello".to_string())));
```

**验收标准**: 批量创建数组方法能够正确从切片创建数组。

#### 测试用例 5: 类型错误处理

```rust
use lx_json::{JsonNode, JsonError};

// 尝试向非数组添加元素
let mut node = JsonNode::new_object();
let result = node.add_item_to_array(JsonNode::Number(1.0));
assert!(result.is_err());
match result {
    Err(JsonError::InvalidType { expected, found }) => {
        assert_eq!(expected, "Array");
        assert_eq!(found, "Object");
    }
    _ => panic!("Expected InvalidType error"),
}

// 尝试向非对象添加键值对
let mut node = JsonNode::new_array();
let result = node.add_item_to_object("key", JsonNode::String("value".to_string()));
assert!(result.is_err());
match result {
    Err(JsonError::InvalidType { expected, found }) => {
        assert_eq!(expected, "Object");
        assert_eq!(found, "Array");
    }
    _ => panic!("Expected InvalidType error"),
}
```

**验收标准**: 类型不匹配时返回明确的 `InvalidType` 错误信息。

#### 测试用例 6: 链式调用构建复杂 JSON

```rust
use lx_json::JsonNode;

// 构建复杂 JSON 结构
let mut root = JsonNode::new_object();
root.add_string_to_object("name", "Alice").unwrap();
root.add_number_to_object("age", 30.0).unwrap();
root.add_bool_to_object("active", true).unwrap();

// 添加数组
let mut hobbies = JsonNode::new_array();
hobbies.add_item_to_array(JsonNode::String("reading".to_string())).unwrap();
hobbies.add_item_to_array(JsonNode::String("gaming".to_string())).unwrap();
root.add_item_to_object("hobbies", hobbies).unwrap();

// 添加嵌套对象
let mut address = JsonNode::new_object();
address.add_string_to_object("city", "New York").unwrap();
address.add_string_to_object("country", "USA").unwrap();
root.add_item_to_object("address", address).unwrap();

// 验证结构
assert_eq!(root.len(), 4);
assert_eq!(root.get("name").and_then(|v| v.as_string()), Some("Alice"));
assert_eq!(root.get("age").and_then(|v| v.as_number()), Some(30.0));
assert_eq!(root.get("active").and_then(|v| v.as_bool()), Some(true));
```

**验收标准**: 支持通过链式调用构建复杂的 JSON 结构。

#### 测试用例 7: 便捷null/true/false/raw添加方法

```rust
use lx_json::JsonNode;

let mut obj = JsonNode::new_object();
obj.add_null_to_object("nullable").unwrap();
obj.add_true_to_object("flag_true").unwrap();
obj.add_false_to_object("flag_false").unwrap();
obj.add_raw_to_object("raw_json", r#"{\"nested\": \"value\"}"#).unwrap();

assert_eq!(obj.len(), 4);
assert_eq!(obj.get("nullable"), Some(&JsonNode::Null));
assert_eq!(obj.get("flag_true"), Some(&JsonNode::Bool(true)));
assert_eq!(obj.get("flag_false"), Some(&JsonNode::Bool(false)));
assert_eq!(obj.get("raw_json").and_then(|v| v.as_string()), Some(r#"{\"nested\": \"value\"}"#));
```

**验收标准**: 便捷方法能够正确添加 null、true、false 和 raw JSON 值到对象。

#### 测试用例 8: 数组删除和分离操作

```rust
use lx_json::JsonNode;

let mut arr = JsonNode::new_array();
arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
arr.add_item_to_array(JsonNode::Number(2.0)).unwrap();
arr.add_item_to_array(JsonNode::Number(3.0)).unwrap();
arr.add_item_to_array(JsonNode::Number(4.0)).unwrap();

assert_eq!(arr.len(), 4);

// 删除索引2的元素（值为3.0）
arr.delete_item_from_array(2).unwrap();
assert_eq!(arr.len(), 3);
assert_eq!(arr.get_at(2), Some(&JsonNode::Number(4.0)));

// 分离索引0的元素（值为1.0）
let detached = arr.detach_item_from_array(0).unwrap();
assert_eq!(detached, JsonNode::Number(1.0));
assert_eq!(arr.len(), 2);
```

**验收标准**: 能够正确从数组中删除和分离元素，`delete` 会直接删除，`detach` 会返回被删除的元素的所有权。

#### 测试用例 9: 对象删除和分离操作

```rust
use lx_json::JsonNode;

let mut obj = JsonNode::new_object();
obj.add_string_to_object("name", "Alice").unwrap();
obj.add_number_to_object("age", 30.0).unwrap();
obj.add_bool_to_object("active", true).unwrap();

assert_eq!(obj.len(), 3);

// 删除键 "age"
obj.delete_item_from_object("age").unwrap();
assert_eq!(obj.len(), 2);
assert_eq!(obj.get("age"), None);

// 分离键 "name"
let detached = obj.detach_item_from_object("name").unwrap();
assert_eq!(detached, JsonNode::String("Alice".to_string()));
assert_eq!(obj.len(), 1);
```

**验收标准**: 能够正确从对象中删除和分离键值对，`delete` 会直接删除，`detach` 会返回被删除的值的所有权。

#### 测试用例 10: 数组替换和插入操作

```rust
use lx_json::JsonNode;

let mut arr = JsonNode::new_array();
arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
arr.add_item_to_array(JsonNode::Number(2.0)).unwrap();
arr.add_item_to_array(JsonNode::Number(3.0)).unwrap();

// 替换索引1的元素
arr.replace_item_in_array(1, JsonNode::Number(20.0)).unwrap();
assert_eq!(arr.get_at(1), Some(&JsonNode::Number(20.0)));

// 在索引1处插入新元素
arr.insert_item_in_array(1, JsonNode::String("inserted".to_string())).unwrap();
assert_eq!(arr.len(), 4);
assert_eq!(arr.get_at(1), Some(&JsonNode::String("inserted".to_string())));
assert_eq!(arr.get_at(2), Some(&JsonNode::Number(20.0)));
```

**验收标准**: 能够正确替换数组中的元素和在指定位置插入新元素。

#### 测试用例 11: 对象替换操作

```rust
use lx_json::JsonNode;

let mut obj = JsonNode::new_object();
obj.add_string_to_object("name", "Alice").unwrap();
obj.add_number_to_object("age", 30.0).unwrap();

// 替换 "age" 的值
obj.replace_item_in_object("age", JsonNode::Number(25.0)).unwrap();
assert_eq!(obj.get("age").and_then(|v| v.as_number()), Some(25.0));
```

**验收标准**: 能够正确替换对象中指定键的值。

#### 测试用例 12: 高级打印功能

```rust
use lx_json::{JsonNode, print, print_unformatted, print_buffered, print_preallocated};

let obj = JsonNode::new_object();
let formatted = print(&obj);
let unformatted = print_unformatted(&obj);

// 测试 print_buffered
let buffered = print_buffered(&obj, 100, true);
assert_eq!(buffered, formatted);

let buffered_unformatted = print_buffered(&obj, 100, false);
assert_eq!(buffered_unformatted, unformatted);

// 测试 print_preallocated - 长度足够时返回 Some
let result = print_preallocated(&obj, 100, true);
assert!(result.is_some());
assert_eq!(result.unwrap(), formatted);

// 测试 print_preallocated - 长度不足时返回 None
let result = print_preallocated(&obj, 1, true);
assert!(result.is_none());
```

**验收标准**: `print_buffered` 能够使用预分配的缓冲区进行序列化，`print_preallocated` 能够检查序列化结果是否在指定长度范围内。

#### 测试用例 13: 错误处理（索引越界、键不存在）

```rust
use lx_json::{JsonNode, JsonError};

let mut arr = JsonNode::new_array();
arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();

// 测试索引越界
let result = arr.delete_item_from_array(5);
assert!(result.is_err());
match result {
    Err(JsonError::IndexOutOfBounds { index, length }) => {
        assert_eq!(index, 5);
        assert_eq!(length, 1);
    }
    _ => panic!("Expected IndexOutOfBounds error"),
}

// 测试插入索引越界（大于长度）
let result = arr.insert_item_in_array(5, JsonNode::Number(2.0));
assert!(result.is_err());

let mut obj = JsonNode::new_object();
obj.add_string_to_object("name", "Alice").unwrap();

// 测试键不存在
let result = obj.delete_item_from_object("nonexistent");
assert!(result.is_err());
match result {
    Err(JsonError::KeyNotFound { key }) => {
        assert_eq!(key, "nonexistent");
    }
    _ => panic!("Expected KeyNotFound error"),
}

let result = obj.replace_item_in_object("nonexistent", JsonNode::String("Bob".to_string()));
assert!(result.is_err());

let result = obj.detach_item_from_object("nonexistent");
assert!(result.is_err());
```

**验收标准**: 能够正确处理索引越界和键不存在的错误情况，返回精确的错误信息。

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
15. ✅ 支持动态添加元素到数组（`add_item_to_array()`）
16. ✅ 支持动态添加元素到对象（`add_item_to_object()`）
17. ✅ 支持便捷方法添加字段（`add_string_to_object()`, `add_number_to_object()`, `add_bool_to_object()`）
18. ✅ 支持便捷方法添加 null/true/false/raw（`add_null_to_object()`, `add_true_to_object()`, `add_false_to_object()`, `add_raw_to_object()`）
19. ✅ 支持数组删除操作（`delete_item_from_array()`）
20. ✅ 支持数组分离操作（`detach_item_from_array()`）
21. ✅ 支持对象删除操作（`delete_item_from_object()`）
22. ✅ 支持对象分离操作（`detach_item_from_object()`）
23. ✅ 支持数组替换操作（`replace_item_in_array()`）
24. ✅ 支持对象替换操作（`replace_item_in_object()`）
25. ✅ 支持数组插入操作（`insert_item_in_array()`）
26. ✅ 提供缓冲式打印功能（`print_buffered()`）
27. ✅ 提供预分配缓冲区打印功能（`print_preallocated()`）
28. ✅ 支持索引越界错误（`IndexOutOfBounds`）
29. ✅ 支持键不存在错误（`KeyNotFound`）
30. ✅ 支持批量创建数组（`create_int_array()`, `create_float_array()`, `create_double_array()`, `create_string_array()`）
31. ✅ 支持类型检查方法（`is_null()`, `is_bool()`, `is_number()`, `is_string()`, `is_array()`, `is_object()`）
32. ✅ 支持值访问方法（`as_string()`, `as_number()`, `as_bool()`, `as_array()`, `as_object()`）
17. ✅ 提供便捷的对象字段添加方法（`add_string_to_object()`, `add_number_to_object()`, `add_bool_to_object()`）
18. ✅ 支持批量创建数组（`create_int_array()`, `create_float_array()`, `create_double_array()`, `create_string_array()`）
19. ✅ 正确处理类型不匹配错误，返回 `InvalidType` 错误

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