// 测试模块：包含LX-json库的所有测试用例
// 每个测试都用中文注释说明其作用

#[cfg(test)]
mod tests {
    use crate::*;

    // ==================== parser.rs 测试 ====================

    /// 测试解析 null 值
    #[test]
    fn test_parse_null() {
        let result = parse("null");
        assert_eq!(result, Ok(JsonNode::Null));
    }

    /// 测试解析布尔值（true 和 false）
    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true"), Ok(JsonNode::Bool(true)));
        assert_eq!(parse("false"), Ok(JsonNode::Bool(false)));
    }

    /// 测试解析各种数字格式（整数、负数、小数、科学计数法）
    #[test]
    fn test_parse_number() {
        assert_eq!(parse("42"), Ok(JsonNode::Number(42.0)));
        assert_eq!(parse("-42"), Ok(JsonNode::Number(-42.0)));
        assert_eq!(parse("3.14"), Ok(JsonNode::Number(3.14)));
        assert_eq!(parse("-3.14"), Ok(JsonNode::Number(-3.14)));
        assert_eq!(parse("1e5"), Ok(JsonNode::Number(100000.0)));
        assert_eq!(parse("1.5e-3"), Ok(JsonNode::Number(0.0015)));
    }

    /// 测试解析字符串
    #[test]
    fn test_parse_string() {
        assert_eq!(parse(r#""hello""#), Ok(JsonNode::String("hello".to_string())));
        assert_eq!(parse(r#""world""#), Ok(JsonNode::String("world".to_string())));
    }

    /// 测试解析带转义字符的字符串
    #[test]
    fn test_parse_string_with_escapes() {
        assert_eq!(parse(r#""hello\nworld""#), Ok(JsonNode::String("hello\nworld".to_string())));
        assert_eq!(parse(r#""\"quoted\"""#), Ok(JsonNode::String("\"quoted\"".to_string())));
        assert_eq!(parse(r#""\\t\\\\""#), Ok(JsonNode::String("\\t\\".to_string())));
    }

    /// 测试解析数组（空数组和含元素的数组）
    #[test]
    fn test_parse_array() {
        let result = parse("[]");
        assert_eq!(result, Ok(JsonNode::Array(vec![])));

        let result = parse("[1, 2, 3]");
        assert_eq!(
            result,
            Ok(JsonNode::Array(vec![
                JsonNode::Number(1.0),
                JsonNode::Number(2.0),
                JsonNode::Number(3.0),
            ]))
        );
    }

    /// 测试解析嵌套数组
    #[test]
    fn test_parse_nested_array() {
        let result = parse("[[1], [2], [3]]");
        assert_eq!(
            result,
            Ok(JsonNode::Array(vec![
                JsonNode::Array(vec![JsonNode::Number(1.0)]),
                JsonNode::Array(vec![JsonNode::Number(2.0)]),
                JsonNode::Array(vec![JsonNode::Number(3.0)]),
            ]))
        );
    }

    /// 测试解析对象（空对象和含键值对的对象）
    #[test]
    fn test_parse_object() {
        let result = parse("{}");
        assert_eq!(result, Ok(JsonNode::Object(vec![])));

        let result = parse(r#"{"key": "value"}"#);
        assert_eq!(
            result,
            Ok(JsonNode::Object(vec![(
                "key".to_string(),
                JsonNode::String("value".to_string()),
            )]))
        );
    }

    /// 测试解析嵌套对象
    #[test]
    fn test_parse_nested_object() {
        let result = parse(r#"{"outer": {"inner": "value"}}"#);
        assert_eq!(
            result,
            Ok(JsonNode::Object(vec![(
                "outer".to_string(),
                JsonNode::Object(vec![(
                    "inner".to_string(),
                    JsonNode::String("value".to_string()),
                )]),
            )]))
        );
    }

    /// 测试解析复杂的 JSON 结构
    #[test]
    fn test_parse_complex_json() {
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
    }

    /// 测试带长度限制的解析
    #[test]
    fn test_parse_with_length() {
        let result = parse_with_length(r#"{"key": "value"}"#, 10);
        assert!(result.is_err()); // 截断的 JSON 应该失败
    }

    /// 测试带选项的解析（null 终止符要求）
    #[test]
    fn test_parse_with_options() {
        // 要求 null 终止符
        let opts = ParseOptions::new().with_null_terminated(true);
        let result = parse_with_opts("null extra", opts);
        assert!(result.is_err());

        // 不要求 null 终止符
        let opts = ParseOptions::new().with_null_terminated(false);
        let result = parse_with_opts("null extra", opts);
        assert!(result.is_ok());
    }

    /// 测试解析无效的 JSON
    #[test]
    fn test_invalid_json() {
        assert!(parse("{").is_err()); // 未闭合的对象
        assert!(parse("[").is_err()); // 未闭合的数组
        assert!(parse(r#""unclosed string"#).is_err()); // 未闭合的字符串
        assert!(parse("123abc").is_err()); // 无效的数字后缀
    }

    /// 测试不允许尾随逗号
    #[test]
    fn test_trailing_comma() {
        assert!(parse("[1, 2,]").is_err());
        assert!(parse(r#"{"a": 1,}"#).is_err());
    }

    /// 测试嵌套深度限制
    #[test]
    fn test_nesting_limit() {
        let opts = ParseOptions::new().with_nesting_limit(2);
        let deep_json = "[[[1]]]"; // 嵌套深度 3
        let result = parse_with_opts(deep_json, opts);
        assert!(result.is_err());
    }

    /// 测试空白字符处理
    #[test]
    fn test_whitespace_handling() {
        assert_eq!(parse("  null  "), Ok(JsonNode::Null));
        assert_eq!(parse("\ntrue\n"), Ok(JsonNode::Bool(true)));
        assert_eq!(parse("\t42\t"), Ok(JsonNode::Number(42.0)));
    }

    /// 测试空输入
    #[test]
    fn test_empty_input() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    /// 测试 Unicode 转义序列
    #[test]
    fn test_unicode_escape() {
        let result = parse(r#""\u0041""#);
        assert_eq!(result, Ok(JsonNode::String("A".to_string())));
    }

    /// 测试重复键（应该报错）
    #[test]
    fn test_duplicate_key() {
        let result = parse(r#"{"key": 1, "key": 2}"#);
        assert!(result.is_err());
    }

    // ==================== error.rs 测试 ====================

    /// 测试错误信息的显示
    #[test]
    fn test_error_display() {
        let err = JsonError::UnexpectedEndOfInput {
            position: 10,
            expected: "value".to_string(),
        };
        assert!(err.to_string().contains("Unexpected end of input"));
        assert!(err.to_string().contains("position 10"));

        let err2 = JsonError::InvalidNumber {
            position: 5,
            reason: "Invalid digit".to_string(),
        };
        assert!(err2.to_string().contains("Invalid number"));
        assert!(err2.to_string().contains("position 5"));

        // 测试新增的错误类型
        let err3 = JsonError::IndexOutOfBounds { index: 5, length: 3 };
        assert!(err3.to_string().contains("Index out of bounds"));

        let err4 = JsonError::KeyNotFound { key: "missing".to_string() };
        assert!(err4.to_string().contains("Key not found"));
    }

    // ==================== types.rs 测试 ====================

    /// 测试类型检查方法（is_null, is_bool, is_number 等）
    #[test]
    fn test_type_checks() {
        let null = JsonNode::Null;
        let bool_val = JsonNode::Bool(true);
        let num = JsonNode::Number(42.0);
        let str_val = JsonNode::String("hello".to_string());
        let arr = JsonNode::Array(vec![]);
        let obj = JsonNode::Object(vec![]);

        assert!(null.is_null());
        assert!(bool_val.is_bool());
        assert!(num.is_number());
        assert!(str_val.is_string());
        assert!(arr.is_array());
        assert!(obj.is_object());
    }

    /// 测试值访问方法（as_string, as_number, as_bool, len, get, get_at）
    #[test]
    fn test_value_accessors() {
        let str_val = JsonNode::String("hello".to_string());
        assert_eq!(str_val.as_string(), Some("hello"));

        let num = JsonNode::Number(42.0);
        assert_eq!(num.as_number(), Some(42.0));

        let bool_val = JsonNode::Bool(true);
        assert_eq!(bool_val.as_bool(), Some(true));

        let arr = JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]);
        assert_eq!(arr.len(), 2);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));

        let obj = JsonNode::Object(vec![("key".to_string(), JsonNode::Number(42.0))]);
        assert_eq!(obj.len(), 1);
        assert_eq!(obj.get("key"), Some(&JsonNode::Number(42.0)));
    }

    /// 测试构造函数（new_null, new_true, new_false 等）
    #[test]
    fn test_constructors() {
        assert_eq!(JsonNode::new_null(), JsonNode::Null);
        assert_eq!(JsonNode::new_true(), JsonNode::Bool(true));
        assert_eq!(JsonNode::new_false(), JsonNode::Bool(false));
        assert_eq!(JsonNode::new_bool(true), JsonNode::Bool(true));
        assert_eq!(JsonNode::new_number(42.0), JsonNode::Number(42.0));
        assert_eq!(JsonNode::new_string("hello"), JsonNode::String("hello".to_string()));
        assert_eq!(JsonNode::new_array(), JsonNode::Array(vec![]));
        assert_eq!(JsonNode::new_object(), JsonNode::Object(vec![]));
    }

    /// 测试向数组添加元素
    #[test]
    fn test_add_item_to_array() {
        let mut arr = JsonNode::new_array();
        arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
        arr.add_item_to_array(JsonNode::String("hello".to_string())).unwrap();
        arr.add_item_to_array(JsonNode::Bool(true)).unwrap();
        
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
        assert_eq!(arr.get_at(1), Some(&JsonNode::String("hello".to_string())));
        assert_eq!(arr.get_at(2), Some(&JsonNode::Bool(true)));
    }

    /// 测试向非数组类型添加元素时的错误处理
    #[test]
    fn test_add_item_to_array_invalid_type() {
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
    }

    /// 测试向对象添加元素
    #[test]
    fn test_add_item_to_object() {
        let mut obj = JsonNode::new_object();
        obj.add_item_to_object("name", JsonNode::String("Alice".to_string())).unwrap();
        obj.add_item_to_object("age", JsonNode::Number(30.0)).unwrap();
        
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("name"), Some(&JsonNode::String("Alice".to_string())));
        assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
    }

    /// 测试向非对象类型添加元素时的错误处理
    #[test]
    fn test_add_item_to_object_invalid_type() {
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
    }

    /// 测试便捷添加方法（add_string_to_object, add_number_to_object, add_bool_to_object）
    #[test]
    fn test_convenience_add_methods() {
        let mut obj = JsonNode::new_object();
        obj.add_string_to_object("name", "Alice").unwrap();
        obj.add_number_to_object("age", 30.0).unwrap();
        obj.add_bool_to_object("active", true).unwrap();
        
        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get("name"), Some(&JsonNode::String("Alice".to_string())));
        assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
        assert_eq!(obj.get("active"), Some(&JsonNode::Bool(true)));
    }

    /// 测试创建整数数组
    #[test]
    fn test_create_int_array() {
        let values: &[i64] = &[1, 2, 3, 4, 5];
        let arr = JsonNode::create_int_array(values);
        
        assert_eq!(arr.len(), 5);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
        assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.0)));
        assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.0)));
        assert_eq!(arr.get_at(3), Some(&JsonNode::Number(4.0)));
        assert_eq!(arr.get_at(4), Some(&JsonNode::Number(5.0)));
    }

    /// 测试创建浮点数组
    #[test]
    fn test_create_float_array() {
        let values: &[f32] = &[1.1, 2.2, 3.3];
        let arr = JsonNode::create_float_array(values);
        
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.1)));
        assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.2)));
        assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.3)));
    }

    /// 测试创建双精度浮点数组
    #[test]
    fn test_create_double_array() {
        let values: &[f64] = &[1.5, 2.5, 3.5, 4.5];
        let arr = JsonNode::create_double_array(values);
        
        assert_eq!(arr.len(), 4);
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.5)));
        assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.5)));
        assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.5)));
        assert_eq!(arr.get_at(3), Some(&JsonNode::Number(4.5)));
    }

    /// 测试创建字符串数组
    #[test]
    fn test_create_string_array() {
        let values: &[&str] = &["hello", "world", "rust"];
        let arr = JsonNode::create_string_array(values);
        
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_at(0), Some(&JsonNode::String("hello".to_string())));
        assert_eq!(arr.get_at(1), Some(&JsonNode::String("world".to_string())));
        assert_eq!(arr.get_at(2), Some(&JsonNode::String("rust".to_string())));
    }

    /// 测试便捷添加 null/true/false/raw 方法
    #[test]
    fn test_add_null_true_false_raw_to_object() {
        let mut obj = JsonNode::new_object();
        obj.add_null_to_object("nullable").unwrap();
        obj.add_true_to_object("flag_true").unwrap();
        obj.add_false_to_object("flag_false").unwrap();
        obj.add_raw_to_object("raw_json", r#"{"nested": "value"}"#).unwrap();
        
        assert_eq!(obj.len(), 4);
        assert_eq!(obj.get("nullable"), Some(&JsonNode::Null));
        assert_eq!(obj.get("flag_true"), Some(&JsonNode::Bool(true)));
        assert_eq!(obj.get("flag_false"), Some(&JsonNode::Bool(false)));
        assert_eq!(obj.get("raw_json").and_then(|v| v.as_string()), Some(r#"{"nested": "value"}"#));
    }

    /// 测试从数组删除元素
    #[test]
    fn test_delete_item_from_array() {
        let mut arr = JsonNode::new_array();
        arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
        arr.add_item_to_array(JsonNode::Number(2.0)).unwrap();
        arr.add_item_to_array(JsonNode::Number(3.0)).unwrap();
        
        assert_eq!(arr.len(), 3);
        
        arr.delete_item_from_array(1).unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr.get_at(1), Some(&JsonNode::Number(3.0)));
    }

    /// 测试从数组分离元素并返回所有权
    #[test]
    fn test_detach_item_from_array() {
        let mut arr = JsonNode::new_array();
        arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
        arr.add_item_to_array(JsonNode::Number(2.0)).unwrap();
        
        let detached = arr.detach_item_from_array(0).unwrap();
        assert_eq!(detached, JsonNode::Number(1.0));
        assert_eq!(arr.len(), 1);
    }

    /// 测试从对象删除元素
    #[test]
    fn test_delete_item_from_object() {
        let mut obj = JsonNode::new_object();
        obj.add_string_to_object("name", "Alice").unwrap();
        obj.add_number_to_object("age", 30.0).unwrap();
        
        assert_eq!(obj.len(), 2);
        
        obj.delete_item_from_object("age").unwrap();
        assert_eq!(obj.len(), 1);
        assert_eq!(obj.get("age"), None);
    }

    /// 测试从对象分离元素并返回所有权
    #[test]
    fn test_detach_item_from_object() {
        let mut obj = JsonNode::new_object();
        obj.add_string_to_object("name", "Alice").unwrap();
        obj.add_number_to_object("age", 30.0).unwrap();
        
        let detached = obj.detach_item_from_object("name").unwrap();
        assert_eq!(detached, JsonNode::String("Alice".to_string()));
        assert_eq!(obj.len(), 1);
    }

    /// 测试替换数组中的元素
    #[test]
    fn test_replace_item_in_array() {
        let mut arr = JsonNode::new_array();
        arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
        arr.add_item_to_array(JsonNode::Number(2.0)).unwrap();
        
        arr.replace_item_in_array(0, JsonNode::Number(10.0)).unwrap();
        assert_eq!(arr.get_at(0), Some(&JsonNode::Number(10.0)));
    }

    /// 测试替换对象中的元素
    #[test]
    fn test_replace_item_in_object() {
        let mut obj = JsonNode::new_object();
        obj.add_string_to_object("name", "Alice").unwrap();
        obj.add_number_to_object("age", 30.0).unwrap();
        
        obj.replace_item_in_object("age", JsonNode::Number(25.0)).unwrap();
        assert_eq!(obj.get("age").and_then(|v| v.as_number()), Some(25.0));
    }

    /// 测试在数组中插入元素
    #[test]
    fn test_insert_item_in_array() {
        let mut arr = JsonNode::new_array();
        arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
        arr.add_item_to_array(JsonNode::Number(3.0)).unwrap();
        
        arr.insert_item_in_array(1, JsonNode::Number(2.0)).unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.0)));
    }

    /// 测试索引越界错误
    #[test]
    fn test_index_out_of_bounds_error() {
        let mut arr = JsonNode::new_array();
        arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
        
        let result = arr.delete_item_from_array(5);
        assert!(result.is_err());
        match result {
            Err(JsonError::IndexOutOfBounds { index, length }) => {
                assert_eq!(index, 5);
                assert_eq!(length, 1);
            }
            _ => panic!("Expected IndexOutOfBounds error"),
        }
    }

    /// 测试键不存在错误
    #[test]
    fn test_key_not_found_error() {
        let mut obj = JsonNode::new_object();
        obj.add_string_to_object("name", "Alice").unwrap();
        
        let result = obj.delete_item_from_object("missing");
        assert!(result.is_err());
        match result {
            Err(JsonError::KeyNotFound { key }) => {
                assert_eq!(key, "missing");
            }
            _ => panic!("Expected KeyNotFound error"),
        }
    }

    // ==================== serializer.rs 测试 ====================

    /// 测试序列化 null 值
    #[test]
    fn test_print_null() {
        let node = JsonNode::Null;
        assert_eq!(print(&node), "null");
        assert_eq!(print_unformatted(&node), "null");
    }

    /// 测试序列化布尔值
    #[test]
    fn test_print_bool() {
        assert_eq!(print(&JsonNode::Bool(true)), "true");
        assert_eq!(print(&JsonNode::Bool(false)), "false");
    }

    /// 测试序列化数字
    #[test]
    fn test_print_number() {
        assert_eq!(print(&JsonNode::Number(42.0)), "42");
        assert_eq!(print(&JsonNode::Number(3.14)), "3.14");
        assert_eq!(print(&JsonNode::Number(-5.5)), "-5.5");
    }

    /// 测试序列化字符串
    #[test]
    fn test_print_string() {
        assert_eq!(print(&JsonNode::String("hello".to_string())), r#""hello""#);
        assert_eq!(print(&JsonNode::String("hello\nworld".to_string())), r#""hello\nworld""#);
    }

    /// 测试序列化数组
    #[test]
    fn test_print_array() {
        let arr = JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::Number(2.0),
            JsonNode::Number(3.0),
        ]);
        let formatted = print(&arr);
        assert!(formatted.contains("["));
        assert!(formatted.contains("1"));
        assert!(formatted.contains("2"));
        assert!(formatted.contains("3"));

        let unformatted = print_unformatted(&arr);
        assert_eq!(unformatted, "[1,2,3]");
    }

    /// 测试序列化对象
    #[test]
    fn test_print_object() {
        let obj = JsonNode::Object(vec![
            ("key".to_string(), JsonNode::String("value".to_string())),
        ]);
        let formatted = print(&obj);
        assert!(formatted.contains("{"));
        assert!(formatted.contains("key"));
        assert!(formatted.contains("value"));

        let unformatted = print_unformatted(&obj);
        assert_eq!(unformatted, r#"{"key":"value"}"#);
    }

    /// 测试 JSON 压缩功能（移除空白字符）
    #[test]
    fn test_minify() {
        let json = r#"{
            "name": "John",
            "age": 30
        }"#;
        let minified = minify(json).unwrap();
        assert_eq!(minified, r#"{"name":"John","age":30}"#);
    }

    /// 测试压缩时保留字符串中的空白字符
    #[test]
    fn test_minify_preserves_string_whitespace() {
        let json = r#"{"name": "John Doe"}"#;
        let minified = minify(json).unwrap();
        assert_eq!(minified, r#"{"name":"John Doe"}"#);
    }

    /// 测试压缩无效的 JSON
    #[test]
    fn test_minify_invalid() {
        let json = r#"{"unclosed": "string}"#;
        let result = minify(json);
        assert!(result.is_err());
    }

    /// 测试序列化和解析的往返一致性
    #[test]
    fn test_round_trip() {
        let original = r#"{"name": "John", "age": 30, "active": true}"#;
        let parsed = parse(original).unwrap();
        let serialized = print_unformatted(&parsed);
        let reparsed = parse(&serialized).unwrap();
        assert_eq!(parsed, reparsed);
    }

    /// 测试缓冲式打印功能
    #[test]
    fn test_print_buffered() {
        let obj = JsonNode::new_object();
        let formatted = print(&obj);
        let buffered = print_buffered(&obj, 100, true);
        assert_eq!(buffered, formatted);
        
        let arr = JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]);
        let unformatted = print_unformatted(&arr);
        let buffered_unformatted = print_buffered(&arr, 100, false);
        assert_eq!(buffered_unformatted, unformatted);
    }

    /// 测试预分配缓冲区打印功能（长度足够）
    #[test]
    fn test_print_preallocated_sufficient() {
        let obj = JsonNode::new_object();
        let formatted = print(&obj);
        let result = print_preallocated(&obj, 100, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), formatted);
    }

    /// 测试预分配缓冲区打印功能（长度不足）
    #[test]
    fn test_print_preallocated_insufficient() {
        let obj = JsonNode::new_object();
        let result = print_preallocated(&obj, 1, true);
        assert!(result.is_none());
    }

    // ==================== lib.rs 测试 ====================

    /// 测试版本号不为空
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    /// 测试基本的解析功能
    #[test]
    fn test_basic_parsing() {
        let json = r#"{"key": "value"}"#;
        let result = parse(json);
        assert!(result.is_ok());
    }

    /// 测试基本的序列化功能
    #[test]
    fn test_basic_serialization() {
        let node = JsonNode::Object(vec![
            ("key".to_string(), JsonNode::String("value".to_string())),
        ]);
        let output = print(&node);
        assert!(output.contains("key"));
        assert!(output.contains("value"));
    }
}