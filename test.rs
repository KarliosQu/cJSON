use lx_json::{
    JsonNode, JsonError,
    parse, parse_with_length, parse_with_opts, ParseOptions,
    print, print_unformatted, minify,
    compare, duplicate,
    get_array_size, get_array_item,
    get_object_item, get_object_item_case_sensitive, has_object_item,
    get_string_value, get_number_value, get_pointer,
    generate_patches, apply_patches, add_patch_to_array,
    merge_patch,
};

// ============================================================================
// 解析功能测试模块 (Parsing Tests)
// ============================================================================

#[test]
fn test_parse_null() {
    let result = parse("null");
    assert_eq!(result.unwrap(), JsonNode::Null);
}

#[test]
fn test_parse_boolean() {
    let true_result = parse("true");
    let false_result = parse("false");
    
    assert_eq!(true_result.unwrap(), JsonNode::Bool(true));
    assert_eq!(false_result.unwrap(), JsonNode::Bool(false));
}

#[test]
fn test_parse_number() {
    // 整数
    assert_eq!(parse("42").unwrap(), JsonNode::Number(42.0));
    assert_eq!(parse("-42").unwrap(), JsonNode::Number(-42.0));
    
    // 浮点数
    assert_eq!(parse("3.14").unwrap(), JsonNode::Number(3.14));
    assert_eq!(parse("-3.14").unwrap(), JsonNode::Number(-3.14));
    
    // 科学计数法
    assert_eq!(parse("1e5").unwrap(), JsonNode::Number(100000.0));
    assert_eq!(parse("1.5e-3").unwrap(), JsonNode::Number(0.0015));
}

#[test]
fn test_parse_string() {
    // 简单字符串
    assert_eq!(
        parse(r#""hello""#).unwrap(),
        JsonNode::String("hello".to_string())
    );
    
    // 带转义字符的字符串
    assert_eq!(
        parse(r#""hello\nworld""#).unwrap(),
        JsonNode::String("hello\nworld".to_string())
    );
    
    // 带引号的字符串
    assert_eq!(
        parse(r#""\"quoted\"""#).unwrap(),
        JsonNode::String("\"quoted\"".to_string())
    );
}

#[test]
fn test_parse_array() {
    // 空数组
    assert_eq!(
        parse("[]").unwrap(),
        JsonNode::Array(vec![])
    );
    
    // 简单数组
    assert_eq!(
        parse("[1, 2, 3]").unwrap(),
        JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::Number(2.0),
            JsonNode::Number(3.0),
        ])
    );
    
    // 混合类型数组
    assert_eq!(
        parse("[1, \"hello\", null]").unwrap(),
        JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::String("hello".to_string()),
            JsonNode::Null,
        ])
    );
}

#[test]
fn test_parse_object() {
    // 空对象
    assert_eq!(
        parse("{}").unwrap(),
        JsonNode::Object(vec![])
    );
    
    // 简单对象
    assert_eq!(
        parse(r#"{"key": "value"}"#).unwrap(),
        JsonNode::Object(vec![
            ("key".to_string(), JsonNode::String("value".to_string())),
        ])
    );
    
    // 多键对象
    assert_eq!(
        parse(r#"{"name": "John", "age": 30}"#).unwrap(),
        JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
            ("age".to_string(), JsonNode::Number(30.0)),
        ])
    );
}

#[test]
fn test_parse_nested() {
    // 嵌套结构
    assert_eq!(
        parse(r#"{"user": {"name": "John", "age": 30}}"#).unwrap(),
        JsonNode::Object(vec![
            ("user".to_string(), JsonNode::Object(vec![
                ("name".to_string(), JsonNode::String("John".to_string())),
                ("age".to_string(), JsonNode::Number(30.0)),
            ])),
        ])
    );
    
    // 数组中的对象
    assert_eq!(
        parse(r#"[{"name": "John"}, {"name": "Jane"}]"#).unwrap(),
        JsonNode::Array(vec![
            JsonNode::Object(vec![
                ("name".to_string(), JsonNode::String("John".to_string())),
            ]),
            JsonNode::Object(vec![
                ("name".to_string(), JsonNode::String("Jane".to_string())),
            ]),
        ])
    );
}

#[test]
fn test_parse_invalid() {
    // 无效 JSON 应该返回错误
    assert!(parse("{invalid}").is_err());
    assert!(parse("{'name': 'John'}").is_err());
    assert!(parse("undefined").is_err());
    assert!(parse("[1, 2,]").is_err());
}

#[test]
fn test_parse_with_length() {
    let json = r#"{"name": "John"}"#"";
    let result = parse_with_length(json, json.len());
    assert!(result.is_ok());
    
    // 限制长度应该仍然能够解析
    let result = parse_with_length(json, 20);
    assert!(result.is_ok());
}

#[test]
fn test_parse_with_opts() {
    let json = r#"{"name": "John"}"#"";
    
    // 默认选项
    let result = parse_with_opts(json, None, None, false);
    assert!(result.is_ok());
    
    // 要求 null 结束
    let result = parse_with_opts(json, None, None, true);
    assert!(result.is_err()); // 没有 null 结束符
}

// ============================================================================
// 序列化功能测试模块 (Serialization Tests)
// ============================================================================

#[test]
fn test_print_formatted() {
    let node = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let output = print(&node);
    assert!(output.contains("name"));
    assert!(output.contains("John"));
    assert!(output.contains("age"));
    assert!(output.contains("30"));
}

#[test]
fn test_print_unformatted() {
    let node = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    let output = print_unformatted(&node);
    assert_eq!(output, "[1,2,3]");
}

#[test]
fn test_print_nested() {
    let node = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
        ])),
    ]);
    
    let output = print(&node);
    assert!(output.contains("user"));
    assert!(output.contains("name"));
    assert!(output.contains("John"));
}

#[test]
fn test_minify() {
    let json_with_whitespace = r#"{
        "name": "John",
        "age": 30
    }"#;
    
    let minified = minify(json_with_whitespace);
    assert!(!minified.contains("\n"));
    assert!(!minified.contains("  "));
    assert!(minified.contains("name"));
    assert!(minified.contains("John"));
}

// ============================================================================
// 创建功能测试模块 (Creation Tests)
// ============================================================================

#[test]
fn test_create_null() {
    let node = JsonNode::Null;
    assert!(node.is_null());
}

#[test]
fn test_create_bool() {
    let true_node = JsonNode::Bool(true);
    assert!(true_node.is_true());
    
    let false_node = JsonNode::Bool(false);
    assert!(false_node.is_false());
}

#[test]
fn test_create_number() {
    let node = JsonNode::Number(42.0);
    assert!(node.is_number());
    assert_eq!(node.as_number(), Some(42.0));
}

#[test]
fn test_create_string() {
    let node = JsonNode::String("hello".to_string());
    assert!(node.is_string());
    assert_eq!(node.as_string(), Some("hello"));
}

#[test]
fn test_create_array() {
    let node = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);
    assert!(node.is_array());
    assert_eq!(node.len(), 2);
}

#[test]
fn test_create_object() {
    let node = JsonNode::Object(vec![
        ("key".to_string(), JsonNode::String("value".to_string())),
    ]);
    assert!(node.is_object());
    assert_eq!(node.len(), 1);
}

#[test]
fn test_create_raw() {
    let node = JsonNode::Raw("raw".to_string());
    assert!(node.is_raw());
}

#[test]
fn test_create_int_array() {
    let numbers = vec![1, 2, 3];
    let node: JsonNode = numbers.iter().map(|&n| JsonNode::Number(n as f64)).collect();
    
    if let JsonNode::Array(arr) = node {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], JsonNode::Number(1.0));
        assert_eq!(arr[1], JsonNode::Number(2.0));
        assert_eq!(arr[2], JsonNode::Number(3.0));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_create_float_array() {
    let numbers = vec![1.0f32, 2.0f32, 3.0f32];
    let node: JsonNode = numbers.iter().map(|&n| JsonNode::Number(n as f64)).collect();
    
    if let JsonNode::Array(arr) = node {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], JsonNode::Number(1.0));
        assert_eq!(arr[1], JsonNode::Number(2.0));
        assert_eq!(arr[2], JsonNode::Number(3.0));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_create_double_array() {
    let numbers = vec![1.0f64, 2.0f64, 3.0f64];
    let node: JsonNode = numbers.iter().map(|&n| JsonNode::Number(n)).collect();
    
    if let JsonNode::Array(arr) = node {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], JsonNode::Number(1.0));
        assert_eq!(arr[1], JsonNode::Number(2.0));
        assert_eq!(arr[2], JsonNode::Number(3.0));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_create_string_array() {
    let strings = vec!["hello", "world", "test"];
    let node: JsonNode = strings.iter().map(|&s| JsonNode::String(s.to_string())).collect();
    
    if let JsonNode::Array(arr) = node {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], JsonNode::String("hello".to_string()));
        assert_eq!(arr[1], JsonNode::String("world".to_string()));
        assert_eq!(arr[2], JsonNode::String("test".to_string()));
    } else {
        panic!("Expected array");
    }
}

// ============================================================================
// 操作功能测试模块 (Manipulation Tests)
// ============================================================================

#[test]
fn test_add_item_to_array() {
    let mut arr = JsonNode::Array(vec![]);
    arr.add_item_to_array(JsonNode::Number(1.0)).unwrap();
    arr.add_item_to_array(JsonNode::String("hello".to_string())).unwrap();
    
    assert_eq!(arr.len(), 2);
}

#[test]
fn test_add_item_to_object() {
    let mut obj = JsonNode::Object(vec![]);
    obj.add_item_to_object("key", JsonNode::String("value".to_string())).unwrap();
    
    assert_eq!(obj.len(), 1);
    assert_eq!(obj.get("key"), Some(&JsonNode::String("value".to_string())));
}

#[test]
fn test_add_string_to_object() {
    let mut obj = JsonNode::Object(vec![]);
    obj.add_string_to_object("name", "John").unwrap();
    
    assert_eq!(obj.get("name"), Some(&JsonNode::String("John".to_string())));
}

#[test]
fn test_add_number_to_object() {
    let mut obj = JsonNode::Object(vec![]);
    obj.add_number_to_object("age", 30.0).unwrap();
    
    assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
}

#[test]
fn test_add_bool_to_object() {
    let mut obj = JsonNode::Object(vec![]);
    obj.add_bool_to_object("active", true).unwrap();
    
    assert_eq!(obj.get("active"), Some(&JsonNode::Bool(true)));
}

#[test]
fn test_delete_item_from_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    arr.delete_item_from_array(1).unwrap();
    
    assert_eq!(arr.len(), 2);
    assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
    assert_eq!(arr.get_at(1), Some(&JsonNode::Number(3.0)));
}

#[test]
fn test_delete_item_from_object() {
    let mut obj = JsonNode::Object(vec![
        ("key1".to_string(), JsonNode::String("value1".to_string())),
        ("key2".to_string(), JsonNode::String("value2".to_string())),
    ]);
    
    obj.delete_item_from_object("key1").unwrap();
    
    assert_eq!(obj.len(), 1);
    assert_eq!(obj.get("key1"), None);
}

#[test]
fn test_detach_item_from_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    let detached = arr.detach_item_from_array(1).unwrap();
    
    assert_eq!(detached, JsonNode::Number(2.0));
    assert_eq!(arr.len(), 2);
}

#[test]
fn test_detach_item_from_object() {
    let mut obj = JsonNode::Object(vec![
        ("key1".to_string(), JsonNode::String("value1".to_string())),
        ("key2".to_string(), JsonNode::String("value2".to_string())),
    ]);
    
    let detached = obj.detach_item_from_object("key1").unwrap();
    
    assert_eq!(detached, JsonNode::String("value1".to_string()));
    assert_eq!(obj.len(), 1);
}

#[test]
fn test_replace_item_in_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    arr.replace_item_in_array(1, JsonNode::Number(20.0)).unwrap();
    
    assert_eq!(arr.get_at(1), Some(&JsonNode::Number(20.0)));
}

#[test]
fn test_replace_item_in_object() {
    let mut obj = JsonNode::Object(vec![
        ("key".to_string(), JsonNode::String("old".to_string())),
    ]);
    
    obj.replace_item_in_object("key", JsonNode::String("new".to_string())).unwrap();
    
    assert_eq!(obj.get("key"), Some(&JsonNode::String("new".to_string())));
}

#[test]
fn test_insert_item_in_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(3.0),
    ]);
    
    arr.insert_item_in_array(1, JsonNode::Number(2.0)).unwrap();
    
    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
    assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.0)));
    assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.0)));
}

// ============================================================================
// 查询功能测试模块 (Query Tests)
// ============================================================================

#[test]
fn test_get_array_size() {
    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    assert_eq!(get_array_size(&arr).unwrap(), 3);
}

#[test]
fn test_get_array_item() {
    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    
    let item = get_array_item(&arr, 1).unwrap();
    assert_eq!(item, &JsonNode::Number(2.0));
}

#[test]
fn test_get_object_item() {
    let obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let item = get_object_item(&obj, "name").unwrap();
    assert_eq!(item, &JsonNode::String("John".to_string()));
}

#[test]
fn test_get_object_item_case_sensitive() {
    let obj = JsonNode::Object(vec![
        ("Name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    // 不区分大小写
    let item = get_object_item(&obj, "name").unwrap();
    assert_eq!(item, &JsonNode::String("John".to_string()));
    
    // 区分大小写
    let item = get_object_item_case_sensitive(&obj, "name");
    assert!(item.is_err());
}

#[test]
fn test_has_object_item() {
    let obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    assert!(has_object_item(&obj, "name"));
    assert!(!has_object_item(&obj, "age"));
}

#[test]
fn test_get_string_value() {
    let node = JsonNode::String("hello".to_string());
    assert_eq!(get_string_value(&node), Some("hello"));
}

#[test]
fn test_get_number_value() {
    let node = JsonNode::Number(42.0);
    assert_eq!(get_number_value(&node), Some(42.0));
}

#[test]
fn test_get_pointer() {
    let obj = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
        ])),
    ]);
    
    let item = get_pointer(&obj, "/user/name").unwrap();
    assert_eq!(item, &JsonNode::String("John".to_string()));
}

// ============================================================================
// 类型检查功能测试模块 (Type Checking Tests)
// ============================================================================

#[test]
fn test_is_null() {
    let node = JsonNode::Null;
    assert!(node.is_null());
}

#[test]
fn test_is_bool() {
    let node = JsonNode::Bool(true);
    assert!(node.is_bool());
}

#[test]
fn test_is_true() {
    let node = JsonNode::Bool(true);
    assert!(node.is_true());
}

#[test]
fn test_is_false() {
    let node = JsonNode::Bool(false);
    assert!(node.is_false());
}

#[test]
fn test_is_number() {
    let node = JsonNode::Number(42.0);
    assert!(node.is_number());
}

#[test]
fn test_is_string() {
    let node = JsonNode::String("hello".to_string());
    assert!(node.is_string());
}

#[test]
fn test_is_array() {
    let node = JsonNode::Array(vec![]);
    assert!(node.is_array());
}

#[test]
fn test_is_object() {
    let node = JsonNode::Object(vec![]);
    assert!(node.is_object());
}

#[test]
fn test_is_raw() {
    let node = JsonNode::Raw("raw".to_string());
    assert!(node.is_raw());
}

// ============================================================================
// JSON Pointer 功能测试模块 (JSON Pointer Tests - RFC 6901)
// ============================================================================

#[test]
fn test_get_pointer_simple() {
    let obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let item = get_pointer(&obj, "/name").unwrap();
    assert_eq!(item, &JsonNode::String("John".to_string()));
}

#[test]
fn test_get_pointer_nested() {
    let obj = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
            ("age".to_string(), JsonNode::Number(30.0)),
        ])),
    ]);
    
    let name = get_pointer(&obj, "/user/name").unwrap();
    assert_eq!(name, &JsonNode::String("John".to_string()));
    
    let age = get_pointer(&obj, "/user/age").unwrap();
    assert_eq!(age, &JsonNode::Number(30.0));
}

#[test]
fn test_get_pointer_array() {
    let obj = JsonNode::Object(vec![
        ("items".to_string(), JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::Number(2.0),
            JsonNode::Number(3.0),
        ])),
    ]);
    
    let item = get_pointer(&obj, "/items/1").unwrap();
    assert_eq!(item, &JsonNode::Number(2.0));
}

#[test]
fn test_get_pointer_escape() {
    let obj = JsonNode::Object(vec![
        ("a~b".to_string(), JsonNode::String("value".to_string())),
    ]);
    
    // ~0 代表 ~，~1 代表 /
    let item = get_pointer(&obj, "/a~0b").unwrap();
    assert_eq!(item, &JsonNode::String("value".to_string()));
}

// ============================================================================
// JSON Patch 功能测试模块 (JSON Patch Tests - RFC 6902)
// ============================================================================

#[test]
fn test_generate_patches_simple() {
    let from = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let to = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
    ]);
    
    let patches = generate_patches(&from, &to, true);
    assert!(patches.len() > 0);
}

#[test]
fn test_generate_patches_nested() {
    let from = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
        ])),
    ]);
    
    let to = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("Jane".to_string())),
        ])),
    ]);
    
    let patches = generate_patches(&from, &to, true);
    assert!(patches.len() > 0);
}

#[test]
fn test_apply_patches_add() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("add".to_string())),
            ("path".to_string(), JsonNode::String("/age".to_string())),
            ("value".to_string(), JsonNode::Number(30.0)),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches, true);
    assert!(result.is_ok());
    assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
}

#[test]
fn test_apply_patches_remove() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("remove".to_string())),
            ("path".to_string(), JsonNode::String("/age".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches, true);
    assert!(result.is_ok());
    assert_eq!(obj.get("age"), None);
}

#[test]
fn test_apply_patches_replace() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("replace".to_string())),
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("Jane".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches, true);
    assert!(result.is_ok());
    assert_eq!(obj.get("name"), Some(&JsonNode::String("Jane".to_string())));
}

#[test]
fn test_apply_patches_test() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("test".to_string())),
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("John".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches, true);
    assert!(result.is_ok());
}

#[test]
fn test_apply_patches_move() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("move".to_string())),
            ("from".to_string(), JsonNode::String("/name".to_string())),
            ("path".to_string(), JsonNode::String("/new_name".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches, true);
    assert!(result.is_ok());
    assert_eq!(obj.get("name"), None);
    assert_eq!(obj.get("new_name"), Some(&JsonNode::String("John".to_string())));
}

#[test]
fn test_apply_patches_copy() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("copy".to_string())),
            ("from".to_string(), JsonNode::String("/name".to_string())),
            ("path".to_string(), JsonNode::String("/copy_name".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches, true);
    assert!(result.is_ok());
    assert_eq!(obj.get("name"), Some(&JsonNode::String("John".to_string())));
    assert_eq!(obj.get("copy_name"), Some(&JsonNode::String("John".to_string())));
}

#[test]
fn test_add_patch_to_array() {
    let mut patches = JsonNode::Array(vec![]);
    
    add_patch_to_array(&mut patches, "add", "/key", &JsonNode::String("value".to_string()));
    
    assert_eq!(patches.len(), 1);
}

// ============================================================================
// JSON Merge Patch 功能测试模块 (JSON Merge Patch Tests - RFC 7396)
// ============================================================================

#[test]
fn test_merge_patch_simple() {
    let mut target = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patch = JsonNode::Object(vec![
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let result = merge_patch(&mut target, &patch, true);
    assert!(result.is_ok());
}

#[test]
fn test_merge_patch_nested() {
    let mut target = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
        ])),
    ]);
    
    let patch = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("age".to_string(), JsonNode::Number(30.0)),
        ])),
    ]);
    
    let result = merge_patch(&mut target, &patch, true);
    assert!(result.is_ok());
}

#[test]
fn test_merge_patch_delete() {
    let mut target = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);
    
    let patch = JsonNode::Object(vec![
        ("age".to_string(), JsonNode::Null),
    ]);
    
    let result = merge_patch(&mut target, &patch, true);
    assert!(result.is_ok());
}

#[test]
fn test_merge_patch_replace() {
    let mut target = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patch = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
    ]);
    
    let result = merge_patch(&mut target, &patch, true);
    assert!(result.is_ok());
}

// ============================================================================
// 错误处理测试模块 (Error Handling Tests)
// ============================================================================

#[test]
fn test_error_invalid_type() {
    let arr = JsonNode::Array(vec![]);
    let result = arr.sort_object(true);
    assert!(result.is_err());
}

#[test]
fn test_error_index_out_of_bounds() {
    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
    ]);
    
    // 尝试访问不存在的索引
    let item = get_array_item(&arr, 10);
    assert!(item.is_err());
}

#[test]
fn test_error_key_not_found() {
    let obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    // 尝试获取不存在的键
    let item = get_object_item(&obj, "nonexistent");
    assert!(item.is_err());
}

#[test]
fn test_error_parse_invalid() {
    let invalid_json = "{invalid}";
    let result = parse(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_error_patch_invalid_operation() {
    let mut obj = JsonNode::Object(vec![]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("invalid_op".to_string())),
            ("path".to_string(), JsonNode::String("/key".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches);
    assert!(result.is_err());
}

#[test]
fn test_error_patch_path_not_found() {
    let mut obj = JsonNode::Object(vec![]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("remove".to_string())),
            ("path".to_string(), JsonNode::String("/nonexistent".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches);
    assert!(result.is_err());
}

#[test]
fn test_error_patch_test_failed() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);
    
    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("test".to_string())),
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("Jane".to_string())),
        ]),
    ]);
    
    let result = apply_patches(&mut obj, &patches);
    assert!(result.is_err());
}

// ============================================================================
// 边界条件和特殊情况测试模块 (Edge Cases and Special Cases Tests)
// ============================================================================

#[test]
fn test_empty_array() {
    let arr = JsonNode::Array(vec![]);
    assert_eq!(get_array_size(&arr), 0);
    assert!(arr.is_array());
}

#[test]
fn test_empty_object() {
    let obj = JsonNode::Object(vec![]);
    assert_eq!(obj.len(), 0);
    assert!(obj.is_object());
}

#[test]
fn test_large_numbers() {
    let large_num = 999999999999999999.0;
    let node = JsonNode::Number(large_num);
    assert_eq!(node.as_number(), Some(large_num));
}

#[test]
fn test_special_characters() {
    let special = r#""hello\tworld\n\b\r\f""#;
    let result = parse(special);
    assert!(result.is_ok());
}

#[test]
fn test_unicode() {
    let unicode = r#""你好\u4e16\u754c""#;
    let result = parse(unicode);
    assert!(result.is_ok());
}

#[test]
fn test_deep_nesting() {
    let nested_json = r#"{"a":{"b":{"c":{"d":{"e":"value"}}}}}"#;
    let result = parse(nested_json);
    assert!(result.is_ok());
}

// ============================================================================
// 原有测试 (Existing Tests)
// ============================================================================

#[test]
fn test_duplicate_simple() {
    // Test simple types
    let null_node = JsonNode::Null;
    assert_eq!(duplicate(&null_node, true), null_node);

    let bool_node = JsonNode::Bool(true);
    assert_eq!(duplicate(&bool_node, true), bool_node);

    let number_node = JsonNode::Number(42.0);
    assert_eq!(duplicate(&number_node, true), number_node);

    let string_node = JsonNode::String("hello".to_string());
    assert_eq!(duplicate(&string_node, true), string_node);

    let raw_node = JsonNode::Raw("raw".to_string());
    assert_eq!(duplicate(&raw_node, true), raw_node);
}

#[test]
fn test_duplicate_nested() {
    // Test nested structures with deep copy
    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::String("test".to_string()),
        JsonNode::Array(vec![JsonNode::Number(2.0)]),
    ]);

    let copied = duplicate(&arr, true);
    assert_eq!(arr, copied);

    let obj = JsonNode::Object(vec![
        ("key1".to_string(), JsonNode::String("value1".to_string())),
        ("key2".to_string(), JsonNode::Number(42.0)),
    ]);

    let copied_obj = duplicate(&obj, true);
    assert_eq!(obj, copied_obj);
}

#[test]
fn test_duplicate_recurse_false() {
    // Test shallow copy (recurse = false)
    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::String("test".to_string()),
    ]);

    let shallow = duplicate(&arr, false);
    assert!(matches!(shallow, JsonNode::Array(_)));
    assert_eq!(shallow.len(), 0);

    let obj = JsonNode::Object(vec![
        ("key".to_string(), JsonNode::String("value".to_string())),
    ]);

    let shallow_obj = duplicate(&obj, false);
    assert!(matches!(shallow_obj, JsonNode::Object(_)));
    assert_eq!(shallow_obj.len(), 0);
}

#[test]
fn test_compare_equal() {
    // Test equal values
    let node1 = JsonNode::String("hello".to_string());
    let node2 = JsonNode::String("hello".to_string());
    assert!(compare(&node1, &node2, true));

    let num1 = JsonNode::Number(42.0);
    let num2 = JsonNode::Number(42.0);
    assert!(compare(&num1, &num2, true));

    let arr1 = JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]);
    let arr2 = JsonNode::Array(vec![JsonNode::Number(1.0), JsonNode::Number(2.0)]);
    assert!(compare(&arr1, &arr2, true));
}

#[test]
fn test_compare_not_equal() {
    // Test unequal values
    let node1 = JsonNode::String("hello".to_string());
    let node2 = JsonNode::String("world".to_string());
    assert!(!compare(&node1, &node2, true));

    let num1 = JsonNode::Number(42.0);
    let num2 = JsonNode::Number(43.0);
    assert!(!compare(&num1, &num2, true));

    let bool1 = JsonNode::Bool(true);
    let bool2 = JsonNode::Bool(false);
    assert!(!compare(&bool1, &bool2, true));
}

#[test]
fn test_compare_case_sensitive() {
    // Test case-sensitive comparison
    let node1 = JsonNode::String("hello".to_string());
    let node2 = JsonNode::String("HELLO".to_string());
    assert!(!compare(&node1, &node2, true));

    let raw1 = JsonNode::Raw("test".to_string());
    let raw2 = JsonNode::Raw("TEST".to_string());
    assert!(!compare(&raw1, &raw2, true));
}

#[test]
fn test_compare_case_insensitive() {
    // Test case-insensitive comparison
    let node1 = JsonNode::String("hello".to_string());
    let node2 = JsonNode::String("HELLO".to_string());
    assert!(compare(&node1, &node2, false));

    let raw1 = JsonNode::Raw("test".to_string());
    let raw2 = JsonNode::Raw("TEST".to_string());
    assert!(compare(&raw1, &raw2, false));
}

#[test]
fn test_compare_arrays() {
    // Test array comparison
    let arr1 = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::String("test".to_string()),
    ]);
    let arr2 = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::String("test".to_string()),
    ]);
    assert!(compare(&arr1, &arr2, true));

    // Different lengths
    let arr3 = JsonNode::Array(vec![JsonNode::Number(1.0)]);
    assert!(!compare(&arr1, &arr3, true));

    // Nested arrays
    let nested1 = JsonNode::Array(vec![
        JsonNode::Array(vec![JsonNode::Number(1.0)]),
    ]);
    let nested2 = JsonNode::Array(vec![
        JsonNode::Array(vec![JsonNode::Number(1.0)]),
    ]);
    assert!(compare(&nested1, &nested2, true));
}

#[test]
fn test_compare_objects() {
    // Test object comparison
    let obj1 = JsonNode::Object(vec![
        ("key1".to_string(), JsonNode::String("value1".to_string())),
        ("key2".to_string(), JsonNode::Number(42.0)),
    ]);
    let obj2 = JsonNode::Object(vec![
        ("key1".to_string(), JsonNode::String("value1".to_string())),
        ("key2".to_string(), JsonNode::Number(42.0)),
    ]);
    assert!(compare(&obj1, &obj2, true));

    // Different values
    let obj3 = JsonNode::Object(vec![
        ("key1".to_string(), JsonNode::String("different".to_string())),
        ("key2".to_string(), JsonNode::Number(42.0)),
    ]);
    assert!(!compare(&obj1, &obj3, true));

    // Nested objects
    let nested1 = JsonNode::Object(vec![
        ("outer".to_string(), JsonNode::Object(vec![
            ("inner".to_string(), JsonNode::String("value".to_string())),
        ])),
    ]);
    let nested2 = JsonNode::Object(vec![
        ("outer".to_string(), JsonNode::Object(vec![
            ("inner".to_string(), JsonNode::String("value".to_string())),
        ])),
    ]);
    assert!(compare(&nested1, &nested2, true));
}

#[test]
fn test_compare_mismatched_types() {
    // Test comparison of different types
    let null_node = JsonNode::Null;
    let bool_node = JsonNode::Bool(true);
    assert!(!compare(&null_node, &bool_node, true));

    let string_node = JsonNode::String("test".to_string());
    let number_node = JsonNode::Number(42.0);
    assert!(!compare(&string_node, &number_node, true));

    let arr_node = JsonNode::Array(vec![]);
    let obj_node = JsonNode::Object(vec![]);
    assert!(!compare(&arr_node, &obj_node, true));
}

#[test]
fn test_compare_object_keys_case_sensitive() {
    // Test object key comparison with case sensitivity
    let obj1 = JsonNode::Object(vec![
        ("Key".to_string(), JsonNode::String("value".to_string())),
    ]);
    let obj2 = JsonNode::Object(vec![
        ("key".to_string(), JsonNode::String("value".to_string())),
    ]);

    // Case-sensitive: keys are different
    assert!(!compare(&obj1, &obj2, true));

    // Case-insensitive: keys are the same
    assert!(compare(&obj1, &obj2, false));
}

#[test]
fn test_delete_item_from_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);

    arr.delete_item_from_array(1).unwrap();

    assert_eq!(arr.len(), 2);
    assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
    assert_eq!(arr.get_at(1), Some(&JsonNode::Number(3.0)));
}

#[test]
fn test_delete_item_from_object() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Alice".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
        ("city".to_string(), JsonNode::String("New York".to_string())),
    ]);

    obj.delete_item_from_object("age").unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("name"), Some(&JsonNode::String("Alice".to_string())));
    assert_eq!(obj.get("age"), None);
    assert_eq!(obj.get("city"), Some(&JsonNode::String("New York".to_string())));
}

#[test]
fn test_detach_item_from_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::String("hello".to_string()),
        JsonNode::Bool(true),
    ]);

    let item = arr.detach_item_from_array(1).unwrap();

    assert_eq!(arr.len(), 2);
    assert_eq!(item, JsonNode::String("hello".to_string()));
    assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
    assert_eq!(arr.get_at(1), Some(&JsonNode::Bool(true)));
}

#[test]
fn test_detach_item_from_object() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Alice".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    let item = obj.detach_item_from_object("name").unwrap();

    assert_eq!(obj.len(), 1);
    assert_eq!(item, JsonNode::String("Alice".to_string()));
    assert_eq!(obj.get("name"), None);
    assert_eq!(obj.get("age"), Some(&JsonNode::Number(30.0)));
}

#[test]
fn test_replace_item_in_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);

    arr.replace_item_in_array(1, JsonNode::String("replaced".to_string())).unwrap();

    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
    assert_eq!(arr.get_at(1), Some(&JsonNode::String("replaced".to_string())));
    assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.0)));
}

#[test]
fn test_replace_item_in_object() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Alice".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    obj.replace_item_in_object("age", JsonNode::Number(31.0)).unwrap();

    assert_eq!(obj.len(), 2);
    assert_eq!(obj.get("age"), Some(&JsonNode::Number(31.0)));
}

#[test]
fn test_insert_item_in_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(3.0),
    ]);

    arr.insert_item_in_array(1, JsonNode::Number(2.0)).unwrap();

    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
    assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.0)));
    assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.0)));
}

#[test]
fn test_delete_item_from_array_out_of_bounds() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);

    let result = arr.delete_item_from_array(5);
    assert!(result.is_err());
    match result {
        Err(JsonError::IndexOutOfBounds { index, length }) => {
            assert_eq!(index, 5);
            assert_eq!(length, 2);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_delete_item_from_object_key_not_found() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Alice".to_string())),
    ]);

    let result = obj.delete_item_from_object("age");
    assert!(result.is_err());
    match result {
        Err(JsonError::KeyNotFound { key }) => {
            assert_eq!(key, "age");
        }
        _ => panic!("Expected KeyNotFound error"),
    }
}

#[test]
fn test_delete_item_from_array_invalid_type() {
    let mut node = JsonNode::String("not an array".to_string());

    let result = node.delete_item_from_array(0);
    assert!(result.is_err());
    match result {
        Err(JsonError::InvalidType { expected, found }) => {
            assert_eq!(expected, "Array");
            assert_eq!(found, "String");
        }
        _ => panic!("Expected InvalidType error"),
    }
}

#[test]
fn test_delete_item_from_object_invalid_type() {
    let mut node = JsonNode::Number(42.0);

    let result = node.delete_item_from_object("key");
    assert!(result.is_err());
    match result {
        Err(JsonError::InvalidType { expected, found }) => {
            assert_eq!(expected, "Object");
            assert_eq!(found, "Number");
        }
        _ => panic!("Expected InvalidType error"),
    }
}

#[test]
fn test_detach_item_from_array_out_of_bounds() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);

    let result = arr.detach_item_from_array(5);
    assert!(result.is_err());
    match result {
        Err(JsonError::IndexOutOfBounds { index, length }) => {
            assert_eq!(index, 5);
            assert_eq!(length, 2);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_detach_item_from_object_key_not_found() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Alice".to_string())),
    ]);

    let result = obj.detach_item_from_object("age");
    assert!(result.is_err());
    match result {
        Err(JsonError::KeyNotFound { key }) => {
            assert_eq!(key, "age");
        }
        _ => panic!("Expected KeyNotFound error"),
    }
}

#[test]
fn test_replace_item_in_array_out_of_bounds() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);

    let result = arr.replace_item_in_array(5, JsonNode::Number(3.0));
    assert!(result.is_err());
    match result {
        Err(JsonError::IndexOutOfBounds { index, length }) => {
            assert_eq!(index, 5);
            assert_eq!(length, 2);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_replace_item_in_object_key_not_found() {
    let mut obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Alice".to_string())),
    ]);

    let result = obj.replace_item_in_object("age", JsonNode::Number(30.0));
    assert!(result.is_err());
    match result {
        Err(JsonError::KeyNotFound { key }) => {
            assert_eq!(key, "age");
        }
        _ => panic!("Expected KeyNotFound error"),
    }
}

#[test]
fn test_insert_item_in_array_out_of_bounds() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);

    let result = arr.insert_item_in_array(5, JsonNode::Number(3.0));
    assert!(result.is_err());
    match result {
        Err(JsonError::IndexOutOfBounds { index, length }) => {
            assert_eq!(index, 5);
            assert_eq!(length, 2);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_insert_item_in_array_invalid_type() {
    let mut node = JsonNode::String("not an array".to_string());

    let result = node.insert_item_in_array(0, JsonNode::Number(1.0));
    assert!(result.is_err());
    match result {
        Err(JsonError::InvalidType { expected, found }) => {
            assert_eq!(expected, "Array");
            assert_eq!(found, "String");
        }
        _ => panic!("Expected InvalidType error"),
    }
}

#[test]
fn test_insert_item_at_end_of_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);

    arr.insert_item_in_array(2, JsonNode::Number(3.0)).unwrap();

    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get_at(2), Some(&JsonNode::Number(3.0)));
}

#[test]
fn test_insert_item_at_beginning_of_array() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);

    arr.insert_item_in_array(0, JsonNode::Number(1.0)).unwrap();

    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get_at(0), Some(&JsonNode::Number(1.0)));
    assert_eq!(arr.get_at(1), Some(&JsonNode::Number(2.0)));
}

// ============ Query Function Tests ============

#[test]
fn test_get_array_size() {
    use lx_json::get_array_size;

    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    assert_eq!(get_array_size(&arr).unwrap(), 3);

    let empty_arr = JsonNode::Array(vec![]);
    assert_eq!(get_array_size(&empty_arr).unwrap(), 0);

    let not_arr = JsonNode::String("not an array".to_string());
    assert!(get_array_size(&not_arr).is_err());
}

#[test]
fn test_get_array_item() {
    use lx_json::get_array_item;

    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::String("hello".to_string()),
        JsonNode::Bool(true),
    ]);

    assert_eq!(
        get_array_item(&arr, 0).unwrap(),
        &JsonNode::Number(1.0)
    );
    assert_eq!(
        get_array_item(&arr, 1).unwrap(),
        &JsonNode::String("hello".to_string())
    );
    assert_eq!(
        get_array_item(&arr, 2).unwrap(),
        &JsonNode::Bool(true)
    );

    // Index out of bounds
    assert!(get_array_item(&arr, 3).is_err());
    assert!(get_array_item(&arr, 10).is_err());

    // Not an array
    let not_arr = JsonNode::String("not an array".to_string());
    assert!(get_array_item(&not_arr, 0).is_err());
}

#[test]
fn test_get_object_item() {
    use lx_json::get_object_item;

    let obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
        ("city".to_string(), JsonNode::String("New York".to_string())),
    ]);

    assert_eq!(
        get_object_item(&obj, "name").unwrap(),
        &JsonNode::String("John".to_string())
    );
    assert_eq!(
        get_object_item(&obj, "age").unwrap(),
        &JsonNode::Number(30.0)
    );
    assert_eq!(
        get_object_item(&obj, "city").unwrap(),
        &JsonNode::String("New York".to_string())
    );

    // Key not found
    assert!(get_object_item(&obj, "country").is_err());

    // Not an object
    let not_obj = JsonNode::String("not an object".to_string());
    assert!(get_object_item(&not_obj, "key").is_err());
}

#[test]
fn test_get_object_item_case_sensitive() {
    use lx_json::get_object_item_case_sensitive;

    let obj = JsonNode::Object(vec![
        ("Name".to_string(), JsonNode::String("John".to_string())),
        ("name".to_string(), JsonNode::String("Jane".to_string())),
        ("NAME".to_string(), JsonNode::String("Bob".to_string())),
    ]);

    // Each case should be distinct
    assert_eq!(
        get_object_item_case_sensitive(&obj, "Name").unwrap(),
        &JsonNode::String("John".to_string())
    );
    assert_eq!(
        get_object_item_case_sensitive(&obj, "name").unwrap(),
        &JsonNode::String("Jane".to_string())
    );
    assert_eq!(
        get_object_item_case_sensitive(&obj, "NAME").unwrap(),
        &JsonNode::String("Bob".to_string())
    );

    // Key not found (different case)
    assert!(get_object_item_case_sensitive(&obj, "NaMe").is_err());
}

#[test]
fn test_has_object_item() {
    use lx_json::has_object_item;

    let obj = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    assert!(has_object_item(&obj, "name"));
    assert!(has_object_item(&obj, "age"));
    assert!(!has_object_item(&obj, "city"));

    // Not an object
    let not_obj = JsonNode::String("not an object".to_string());
    assert!(!has_object_item(&not_obj, "key"));
}

#[test]
fn test_get_string_value() {
    use lx_json::get_string_value;

    let str_node = JsonNode::String("hello world".to_string());
    assert_eq!(get_string_value(&str_node), Some("hello world"));

    let empty_str_node = JsonNode::String("".to_string());
    assert_eq!(get_string_value(&empty_str_node), Some(""));

    let num_node = JsonNode::Number(42.0);
    assert_eq!(get_string_value(&num_node), None);

    let bool_node = JsonNode::Bool(true);
    assert_eq!(get_string_value(&bool_node), None);

    let null_node = JsonNode::Null;
    assert_eq!(get_string_value(&null_node), None);
}

#[test]
fn test_get_number_value() {
    use lx_json::get_number_value;

    let int_node = JsonNode::Number(42.0);
    assert_eq!(get_number_value(&int_node), Some(42.0));

    let float_node = JsonNode::Number(3.14159);
    assert_eq!(get_number_value(&float_node), Some(3.14159));

    let str_node = JsonNode::String("not a number".to_string());
    assert_eq!(get_number_value(&str_node), None);

    let bool_node = JsonNode::Bool(true);
    assert_eq!(get_number_value(&bool_node), None);

    let null_node = JsonNode::Null;
    assert_eq!(get_number_value(&null_node), None);
}

#[test]
fn test_get_pointer_root() {
    use lx_json::get_pointer;

    let root = JsonNode::Object(vec![
        ("key".to_string(), JsonNode::String("value".to_string())),
    ]);

    // Pointer "/" refers to the entire document
    let result = get_pointer(&root, "/").unwrap();
    assert_eq!(result, &root);
}

#[test]
fn test_get_pointer_simple_object() {
    use lx_json::get_pointer;

    let root = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
            ("age".to_string(), JsonNode::Number(30.0)),
        ])),
        ("city".to_string(), JsonNode::String("New York".to_string())),
    ]);

    assert_eq!(
        get_pointer(&root, "/user/name").unwrap(),
        &JsonNode::String("John".to_string())
    );
    assert_eq!(
        get_pointer(&root, "/user/age").unwrap(),
        &JsonNode::Number(30.0)
    );
    assert_eq!(
        get_pointer(&root, "/city").unwrap(),
        &JsonNode::String("New York".to_string())
    );
}

#[test]
fn test_get_pointer_array_index() {
    use lx_json::get_pointer;

    let root = JsonNode::Object(vec![
        ("items".to_string(), JsonNode::Array(vec![
            JsonNode::String("item1".to_string()),
            JsonNode::String("item2".to_string()),
            JsonNode::String("item3".to_string()),
        ])),
    ]);

    assert_eq!(
        get_pointer(&root, "/items/0").unwrap(),
        &JsonNode::String("item1".to_string())
    );
    assert_eq!(
        get_pointer(&root, "/items/1").unwrap(),
        &JsonNode::String("item2".to_string())
    );
    assert_eq!(
        get_pointer(&root, "/items/2").unwrap(),
        &JsonNode::String("item3".to_string())
    );
}

#[test]
fn test_get_pointer_nested() {
    use lx_json::get_pointer;

    let root = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("address".to_string(), JsonNode::Object(vec![
                ("city".to_string(), JsonNode::String("New York".to_string())),
                ("country".to_string(), JsonNode::String("USA".to_string())),
            ])),
        ])),
        ("tags".to_string(), JsonNode::Array(vec![
            JsonNode::Object(vec![(
                "name".to_string(),
                JsonNode::String("tag1".to_string()),
            )]),
        ])),
    ]);

    assert_eq!(
        get_pointer(&root, "/user/address/city").unwrap(),
        &JsonNode::String("New York".to_string())
    );
    assert_eq!(
        get_pointer(&root, "/user/address/country").unwrap(),
        &JsonNode::String("USA".to_string())
    );
    assert_eq!(
        get_pointer(&root, "/tags/0/name").unwrap(),
        &JsonNode::String("tag1".to_string())
    );
}

#[test]
fn test_get_pointer_invalid_path() {
    use lx_json::{get_pointer, JsonError};

    let root = JsonNode::Object(vec![(
        "user".to_string(),
        JsonNode::String("John".to_string()),
    )]);

    // Key not found
    match get_pointer(&root, "/nonexistent") {
        Err(JsonError::KeyNotFound { key }) => assert_eq!(key, "nonexistent"),
        _ => panic!("Expected KeyNotFound error"),
    }
}

#[test]
fn test_get_pointer_index_out_of_bounds() {
    use lx_json::{get_pointer, JsonError};

    let root = JsonNode::Object(vec![
        (
            "items".to_string(),
            JsonNode::Array(vec![
                JsonNode::String("item1".to_string()),
                JsonNode::String("item2".to_string()),
            ]),
        ),
    ]);

    // Index out of bounds
    match get_pointer(&root, "/items/10") {
        Err(JsonError::IndexOutOfBounds { index, length }) => {
            assert_eq!(index, 10);
            assert_eq!(length, 2);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_get_pointer_key_not_found() {
    use lx_json::{get_pointer, JsonError};

    let root = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
        ])),
    ]);

    // Nested key not found
    match get_pointer(&root, "/user/age") {
        Err(JsonError::KeyNotFound { key }) => assert_eq!(key, "age"),
        _ => panic!("Expected KeyNotFound error"),
    }
}

#[test]
fn test_get_pointer_with_tilde() {
    use lx_json::get_pointer;

    let root = JsonNode::Object(vec![
        ("key~with~tilde".to_string(), JsonNode::String("value".to_string())),
    ]);

    // ~0 represents ~ in JSON Pointer
    assert_eq!(
        get_pointer(&root, "/key~0with~0tilde").unwrap(),
        &JsonNode::String("value".to_string())
    );
}

#[test]
fn test_get_pointer_with_slash() {
    use lx_json::get_pointer;

    let root = JsonNode::Object(vec![
        (
            "key/with/slash".to_string(),
            JsonNode::String("value".to_string()),
        ),
    ]);

    // ~1 represents / in JSON Pointer
    assert_eq!(
        get_pointer(&root, "/key~1with~1slash").unwrap(),
        &JsonNode::String("value".to_string())
    );
}

#[test]
fn test_is_true() {
    let true_node = JsonNode::new_true();
    assert!(true_node.is_true());
    assert!(!true_node.is_false());

    let false_node = JsonNode::new_false();
    assert!(!false_node.is_true());

    let null_node = JsonNode::new_null();
    assert!(!null_node.is_true());

    let number_node = JsonNode::new_number(42.0);
    assert!(!number_node.is_true());

    let string_node = JsonNode::new_string("true");
    assert!(!string_node.is_true());

    let array_node = JsonNode::new_array();
    assert!(!array_node.is_true());

    let object_node = JsonNode::new_object();
    assert!(!object_node.is_true());

    let raw_node = JsonNode::new_raw("true");
    assert!(!raw_node.is_true());
}

#[test]
fn test_is_false() {
    let false_node = JsonNode::new_false();
    assert!(false_node.is_false());
    assert!(!false_node.is_true());

    let true_node = JsonNode::new_true();
    assert!(!true_node.is_false());

    let null_node = JsonNode::new_null();
    assert!(!null_node.is_false());

    let number_node = JsonNode::new_number(0.0);
    assert!(!number_node.is_false());

    let string_node = JsonNode::new_string("false");
    assert!(!string_node.is_false());

    let array_node = JsonNode::new_array();
    assert!(!array_node.is_false());

    let object_node = JsonNode::new_object();
    assert!(!object_node.is_false());

    let raw_node = JsonNode::new_raw("false");
    assert!(!raw_node.is_false());
}

#[test]
fn test_is_invalid() {
    let null_node = JsonNode::new_null();
    assert!(!null_node.is_invalid());

    let true_node = JsonNode::new_true();
    assert!(!true_node.is_invalid());

    let false_node = JsonNode::new_false();
    assert!(!false_node.is_invalid());

    let number_node = JsonNode::new_number(42.0);
    assert!(!number_node.is_invalid());

    let string_node = JsonNode::new_string("hello");
    assert!(!string_node.is_invalid());

    let array_node = JsonNode::new_array();
    assert!(!array_node.is_invalid());

    let object_node = JsonNode::new_object();
    assert!(!object_node.is_invalid());

    let raw_node = JsonNode::new_raw("raw");
    assert!(!raw_node.is_invalid());
}

#[test]
fn test_is_raw() {
    let raw_node = JsonNode::new_raw("{\"key\": \"value\"}");
    assert!(raw_node.is_raw());

    let null_node = JsonNode::new_null();
    assert!(!null_node.is_raw());

    let bool_node = JsonNode::new_true();
    assert!(!bool_node.is_raw());

    let number_node = JsonNode::new_number(42.0);
    assert!(!number_node.is_raw());

    let string_node = JsonNode::new_string("hello");
    assert!(!string_node.is_raw());

    let array_node = JsonNode::new_array();
    assert!(!array_node.is_raw());

    let object_node = JsonNode::new_object();
    assert!(!object_node.is_raw());
}
// JSON Patch Tests

#[test]
fn test_generate_patches_simple_replace() {
    use lx_json::generate_patches;
    
    let from = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    let to = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    let patches = generate_patches(&from, &to, true).unwrap();
    assert_eq!(patches.len(), 1);

    if let JsonNode::Array(arr) = patches {
        let patch = &arr[0];
        assert_eq!(patch.get("op").unwrap(), &JsonNode::String("replace".to_string()));
        assert_eq!(patch.get("path").unwrap(), &JsonNode::String("/name".to_string()));
    }
}

#[test]
fn test_generate_patches_add_remove() {
    use lx_json::generate_patches;
    
    let from = JsonNode::Object(vec![
        ("a".to_string(), JsonNode::Number(1.0)),
        ("b".to_string(), JsonNode::Number(2.0)),
    ]);

    let to = JsonNode::Object(vec![
        ("b".to_string(), JsonNode::Number(2.0)),
        ("c".to_string(), JsonNode::Number(3.0)),
    ]);

    let patches = generate_patches(&from, &to, true).unwrap();
    assert_eq!(patches.len(), 2);
}

#[test]
fn test_generate_patches_nested_objects() {
    use lx_json::generate_patches;
    
    let from = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("John".to_string())),
            ("age".to_string(), JsonNode::Number(30.0)),
        ])),
    ]);

    let to = JsonNode::Object(vec![
        ("user".to_string(), JsonNode::Object(vec![
            ("name".to_string(), JsonNode::String("Jane".to_string())),
            ("age".to_string(), JsonNode::Number(30.0)),
        ])),
    ]);

    let patches = generate_patches(&from, &to, true).unwrap();
    assert!(patches.len() > 0);
}

#[test]
fn test_generate_patches_arrays() {
    use lx_json::generate_patches;
    
    let from = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);

    let to = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(3.0),
        JsonNode::Number(4.0),
    ]);

    let patches = generate_patches(&from, &to, true).unwrap();
    assert!(patches.len() > 0);
}

#[test]
fn test_apply_patches_add() {
    use lx_json::apply_patches;
    
    let mut target = JsonNode::Object(vec![]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("add".to_string())),
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("John".to_string())),
        ]),
    ]);

    assert!(apply_patches(&mut target, &patches, true).is_ok());
    assert_eq!(target.get("name").unwrap(), &JsonNode::String("John".to_string()));
}

#[test]
fn test_apply_patches_remove() {
    use lx_json::apply_patches;
    
    let mut target = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("remove".to_string())),
            ("path".to_string(), JsonNode::String("/age".to_string())),
        ]),
    ]);

    assert!(apply_patches(&mut target, &patches, true).is_ok());
    assert!(target.get("age").is_none());
}

#[test]
fn test_apply_patches_replace() {
    use lx_json::apply_patches;
    
    let mut target = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
    ]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("replace".to_string())),
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("Jane".to_string())),
        ]),
    ]);

    assert!(apply_patches(&mut target, &patches, true).is_ok());
    assert_eq!(target.get("name").unwrap(), &JsonNode::String("Jane".to_string()));
}

#[test]
fn test_apply_patches_move() {
    use lx_json::apply_patches;
    
    let mut target = JsonNode::Object(vec![
        ("a".to_string(), JsonNode::Number(1.0)),
        ("b".to_string(), JsonNode::Number(2.0)),
    ]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("move".to_string())),
            ("from".to_string(), JsonNode::String("/a".to_string())),
            ("path".to_string(), JsonNode::String("/c".to_string())),
        ]),
    ]);

    assert!(apply_patches(&mut target, &patches, true).is_ok());
    assert!(target.get("a").is_none());
    assert_eq!(target.get("c").unwrap(), &JsonNode::Number(1.0));
}

#[test]
fn test_apply_patches_copy() {
    use lx_json::apply_patches;
    
    let mut target = JsonNode::Object(vec![
        ("a".to_string(), JsonNode::Number(1.0)),
    ]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("copy".to_string())),
            ("from".to_string(), JsonNode::String("/a".to_string())),
            ("path".to_string(), JsonNode::String("/b".to_string())),
        ]),
    ]);

    assert!(apply_patches(&mut target, &patches, true).is_ok());
    assert_eq!(target.get("a").unwrap(), &JsonNode::Number(1.0));
    assert_eq!(target.get("b").unwrap(), &JsonNode::Number(1.0));
}

#[test]
fn test_apply_patches_test() {
    use lx_json::apply_patches;
    
    let mut target = JsonNode::Object(vec![
        ("value".to_string(), JsonNode::Number(42.0)),
    ]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("test".to_string())),
            ("path".to_string(), JsonNode::String("/value".to_string())),
            ("value".to_string(), JsonNode::Number(42.0)),
        ]),
    ]);

    assert!(apply_patches(&mut target, &patches, true).is_ok());
}

#[test]
fn test_apply_patches_test_failed() {
    use lx_json::{apply_patches, JsonError};
    
    let mut target = JsonNode::Object(vec![
        ("value".to_string(), JsonNode::Number(42.0)),
    ]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("test".to_string())),
            ("path".to_string(), JsonNode::String("/value".to_string())),
            ("value".to_string(), JsonNode::Number(43.0)),
        ]),
    ]);

    let result = apply_patches(&mut target, &patches, true);
    assert!(result.is_err());
    if let Err(JsonError::PatchTestFailed { .. }) = result {
        // Expected error
    } else {
        panic!("Expected PatchTestFailed error");
    }
}

#[test]
fn test_apply_patches_invalid_operation() {
    use lx_json::{apply_patches, JsonError};
    
    let mut target = JsonNode::Object(vec![]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("invalid".to_string())),
            ("path".to_string(), JsonNode::String("/test".to_string())),
        ]),
    ]);

    let result = apply_patches(&mut target, &patches, true);
    assert!(result.is_err());
    if let Err(JsonError::InvalidPatchOperation { .. }) = result {
        // Expected error
    } else {
        panic!("Expected InvalidPatchOperation error");
    }
}

#[test]
fn test_apply_patches_path_not_found() {
    use lx_json::apply_patches;
    
    let mut target = JsonNode::Object(vec![]);

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("remove".to_string())),
            ("path".to_string(), JsonNode::String("/nonexistent".to_string())),
        ]),
    ]);

    let result = apply_patches(&mut target, &patches, true);
    assert!(result.is_err());
}

#[test]
fn test_add_patch_to_array() {
    use lx_json::add_patch_to_array;
    
    let mut patches = JsonNode::Array(vec![]);
    let result = add_patch_to_array(
        &mut patches,
        "add",
        "/baz",
        Some(&JsonNode::String("qux".to_string())),
    );

    assert!(result.is_ok());
    if let JsonNode::Array(arr) = patches {
        assert_eq!(arr.len(), 1);
        let patch = &arr[0];
        assert_eq!(patch.get("op").unwrap(), &JsonNode::String("add".to_string()));
        assert_eq!(patch.get("path").unwrap(), &JsonNode::String("/baz".to_string()));
    }
}

// JSON Merge Patch Tests

#[test]
fn test_merge_patch_simple_replace() {
    use lx_json::merge_patch;
    
    let mut target = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    let patch = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
    ]);

    assert!(merge_patch(&mut target, &patch, true).is_ok());
    assert_eq!(target.get("name").unwrap(), &JsonNode::String("Jane".to_string()));
    assert_eq!(target.get("age").unwrap(), &JsonNode::Number(30.0));
}

#[test]
fn test_merge_patch_null_deletes() {
    use lx_json::merge_patch;
    
    let mut target = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("John".to_string())),
        ("age".to_string(), JsonNode::Number(30.0)),
    ]);

    let patch = JsonNode::Object(vec![
        ("age".to_string(), JsonNode::Null),
    ]);

    assert!(merge_patch(&mut target, &patch, true).is_ok());
    assert_eq!(target.get("name").unwrap(), &JsonNode::String("John".to_string()));
    assert!(target.get("age").is_none());
}

#[test]
fn test_merge_patch_nested_objects() {
    use lx_json::merge_patch;
    
    let mut target = JsonNode::Object(vec![
        ("address".to_string(), JsonNode::Object(vec![
            ("city".to_string(), JsonNode::String("Boston".to_string())),
            ("country".to_string(), JsonNode::String("USA".to_string())),
        ])),
    ]);

    let patch = JsonNode::Object(vec![
        ("address".to_string(), JsonNode::Object(vec![
            ("city".to_string(), JsonNode::String("New York".to_string())),
            ("state".to_string(), JsonNode::String("NY".to_string())),
        ])),
    ]);

    assert!(merge_patch(&mut target, &patch, true).is_ok());
}

#[test]
fn test_merge_patch_case_sensitive() {
    use lx_json::merge_patch;
    
    let mut target = JsonNode::Object(vec![
        ("Name".to_string(), JsonNode::String("John".to_string())),
    ]);

    let patch = JsonNode::Object(vec![
        ("Name".to_string(), JsonNode::String("Jane".to_string())),
    ]);

    assert!(merge_patch(&mut target, &patch, true).is_ok());
    assert_eq!(target.get("Name").unwrap(), &JsonNode::String("Jane".to_string()));
}

#[test]
fn test_merge_patch_case_insensitive() {
    use lx_json::merge_patch;
    
    let mut target = JsonNode::Object(vec![
        ("Name".to_string(), JsonNode::String("John".to_string())),
    ]);

    let patch = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("Jane".to_string())),
    ]);

    assert!(merge_patch(&mut target, &patch, false).is_ok());
}

#[test]
fn test_merge_patch_primitive_replacement() {
    use lx_json::merge_patch;
    
    let mut target = JsonNode::String("old");
    let patch = JsonNode::String("new");

    assert!(merge_patch(&mut target, &patch, true).is_ok());
    assert_eq!(target, JsonNode::String("new".to_string()));
}

#[test]
fn test_merge_patch_null_replacement() {
    use lx_json::merge_patch;
    
    let mut target = JsonNode::String("old");
    let patch = JsonNode::Null;

    assert!(merge_patch(&mut target, &patch, true).is_ok());
    assert_eq!(target, JsonNode::Null);
}

// Object Sorting Tests

#[test]
fn test_sort_object_case_sensitive() {
    let mut obj = JsonNode::Object(vec![
        ("zebra".to_string(), JsonNode::Number(3.0)),
        ("apple".to_string(), JsonNode::Number(1.0)),
        ("banana".to_string(), JsonNode::Number(2.0)),
    ]);

    assert!(obj.sort_object(true).is_ok());

    if let JsonNode::Object(pairs) = &obj {
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["apple", "banana", "zebra"]);
    }
}

#[test]
fn test_sort_object_case_insensitive() {
    let mut obj = JsonNode::Object(vec![
        ("Zebra".to_string(), JsonNode::Number(3.0)),
        ("apple".to_string(), JsonNode::Number(1.0)),
        ("Banana".to_string(), JsonNode::Number(2.0)),
    ]);

    assert!(obj.sort_object(false).is_ok());

    if let JsonNode::Object(pairs) = &obj {
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["apple", "Banana", "Zebra"]);
    }
}

#[test]
fn test_sort_object_invalid_type() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
    ]);

    let result = arr.sort_object(true);
    assert!(result.is_err());
    if let Err(JsonError::InvalidType { .. }) = result {
        // Expected error
    } else {
        panic!("Expected InvalidType error");
    }
}

#[test]
fn test_sort_object_empty() {
    let mut obj = JsonNode::Object(vec![]);
    assert!(obj.sort_object(true).is_ok());
    assert_eq!(obj.len(), 0);
}

#[test]
fn test_sort_object_preserves_values() {
    let mut obj = JsonNode::Object(vec![
        ("b".to_string(), JsonNode::Number(2.0)),
        ("a".to_string(), JsonNode::Number(1.0)),
        ("c".to_string(), JsonNode::Number(3.0)),
    ]);\n\n    assert!(obj.sort_object(true).is_ok());\n\n    assert_eq!(obj.get(\"a\").unwrap(), &JsonNode::Number(1.0));\n    assert_eq!(obj.get(\"b\").unwrap(), &JsonNode::Number(2.0));\n    assert_eq!(obj.get(\"c\").unwrap(), &JsonNode::Number(3.0));\n}\n\n// Tests for print_buffered() and print_preallocated()\n\n#[test]\nfn test_print_buffered_formatted() {\n    let node = JsonNode::Object(vec![\n        (\"name\".to_string(), JsonNode::String(\"John\".to_string())),\n        (\"age\".to_string(), JsonNode::Number(30.0)),\n        (\"active\".to_string(), JsonNode::Bool(true)),\n    ]);\n    \n    let result = lx_json::print_buffered(&node, 100, true);\n    \n    assert!(result.contains(\"name\"));\n    assert!(result.contains(\"John\"));\n    assert!(result.contains(\"age\"));\n    assert!(result.contains(\"30\"));\n    assert!(result.contains(\"active\"));\n    assert!(result.contains(\"true\"));\n    // Formatted output should contain newlines\n    assert!(result.contains('\\n'));\n}\n\n#[test]\nfn test_print_buffered_unformatted() {\n    let node = JsonNode::Object(vec![\n        (\"name\".to_string(), JsonNode::String(\"John\".to_string())),\n        (\"age\".to_string(), JsonNode::Number(30.0)),\n    ]);\n    \n    let result = lx_json::print_buffered(&node, 100, false);\n    \n    assert!(result.contains(\"name\"));\n    assert!(result.contains(\"John\"));\n    assert!(result.contains(\"age\"));\n    assert!(result.contains(\"30\"));\n    // Unformatted output should not contain newlines\n    assert!(!result.contains('\\n'));\n}\n\n#[test]\nfn test_print_preallocated_formatted() {\n    let node = JsonNode::Object(vec![\n        (\"key\".to_string(), JsonNode::String(\"value\".to_string())),\n        (\"number\".to_string(), JsonNode::Number(42.0)),\n    ]);\n    \n    let mut buffer = String::with_capacity(100);\n    let result = lx_json::print_preallocated(&node, &mut buffer, true);\n    \n    assert!(result.is_ok());\n    assert!(buffer.contains(\"key\"));\n    assert!(buffer.contains(\"value\"));\n    assert!(buffer.contains(\"number\"));\n    assert!(buffer.contains(\"42\"));\n    // Formatted output should contain newlines\n    assert!(buffer.contains('\\n'));\n}\n\n#[test]\nfn test_print_preallocated_unformatted() {\n    let node = JsonNode::Array(vec![\n        JsonNode::Number(1.0),\n        JsonNode::Number(2.0),\n        JsonNode::Number(3.0),\n    ]);\n    \n    let mut buffer = String::with_capacity(50);\n    let result = lx_json::print_preallocated(&node, &mut buffer, false);\n    \n    assert!(result.is_ok());\n    assert_eq!(buffer, \"[1,2,3]\");\n}\n\n#[test]\nfn test_print_preallocated_clears_buffer() {\n    let node = JsonNode::String(\"test\".to_string());\n    \n    let mut buffer = String::from(\"old content that should be cleared\");\n    let result = lx_json::print_preallocated(&node, &mut buffer, false);\n    \n    assert!(result.is_ok());\n    assert_eq!(buffer, \"\\\"test\\\"\");\n    assert!(!buffer.contains(\"old content\"));\n}\n\n#[test]\nfn test_print_preallocated_with_null() {\n    let node = JsonNode::Null;\n    \n    let mut buffer = String::new();\n    let result = lx_json::print_preallocated(&node, &mut buffer, true);\n    \n    assert!(result.is_ok());\n    assert_eq!(buffer, \"null\");\n}\n\n#[test]\nfn test_print_preallocated_with_complex_nested_structure() {\n    let node = JsonNode::Object(vec![\n        (\"user\".to_string(), JsonNode::Object(vec![\n            (\"name\".to_string(), JsonNode::String(\"Alice\".to_string())),\n            (\"age\".to_string(), JsonNode::Number(25.0)),\n        ])),\n        (\"tags\".to_string(), JsonNode::Array(vec![\n            JsonNode::String(\"rust\".to_string()),\n            JsonNode::String(\"json\".to_string()),\n        ])),\n    ]);\n    \n    let mut buffer = String::with_capacity(200);\n    let result = lx_json::print_preallocated(&node, &mut buffer, true);\n    \n    assert!(result.is_ok());\n    assert!(buffer.contains(\"user\"));\n    assert!(buffer.contains(\"Alice\"));\n    assert!(buffer.contains(\"tags\"));\n    assert!(buffer.contains(\"rust\"));\n}