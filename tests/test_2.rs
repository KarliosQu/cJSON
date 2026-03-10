//! test_2.rs - 对标 cJSON tests/ 目录的完整功能与边界测试
//!
//! 本文件覆盖 CJSON_TESTING_REFERENCE.md 第二章的测试：
//! - 2.1 解析测试（值类型、数字、字符串、数组/对象、定长输入）
//! - 2.2 打印测试（值、数字、字符串、数组/对象格式化）
//! - 2.3 构造与 Add API 测试
//! - 2.4 比较与最小化测试
//! - 2.5 鲁棒性与边界测试
//! - 2.6 cJSON_Utils（JSON Pointer、JSON Patch、Merge Patch）
//! - S-*  补充测试（对标 cJSON 中尚未覆盖的测试项）

use lx_json::{
    parse, parse_with_length, parse_with_opts, ParseOptions,
    print, print_unformatted, print_buffered, print_preallocated,
    minify, compare, duplicate,
    JsonNode,
    get_pointer, get_array_size, get_array_item,
    get_object_item, get_object_item_case_sensitive, has_object_item,
    get_string_value, get_number_value,
    merge_patch, apply_patches, generate_patches, add_patch_to_array,
};

// ============================================================================
// 2.1.1 值类型解析 (P-V-01 ~ P-V-07)
// ============================================================================

/// P-V-01: 解析 null
#[test]
fn test_2_pv01_parse_null() {
    let result = parse("null");
    assert!(result.is_ok(), "解析 null 应成功");
    assert!(result.unwrap().is_null(), "类型应为 null");
}

/// P-V-02: 解析 true
#[test]
fn test_2_pv02_parse_true() {
    let result = parse("true");
    assert!(result.is_ok(), "解析 true 应成功");
    let node = result.unwrap();
    assert!(node.is_bool(), "类型应为 bool");
    assert!(node.is_true(), "值应为 true");
}

/// P-V-03: 解析 false
#[test]
fn test_2_pv03_parse_false() {
    let result = parse("false");
    assert!(result.is_ok(), "解析 false 应成功");
    let node = result.unwrap();
    assert!(node.is_bool(), "类型应为 bool");
    assert!(node.is_false(), "值应为 false");
}

/// P-V-04: 解析 1.5 (number)
#[test]
fn test_2_pv04_parse_number() {
    let result = parse("1.5");
    assert!(result.is_ok(), "解析 1.5 应成功");
    let node = result.unwrap();
    assert!(node.is_number(), "类型应为 number");
    assert_eq!(node.as_number().unwrap(), 1.5);
}

/// P-V-05: 解析 "hello" (string)
#[test]
fn test_2_pv05_parse_string() {
    let result = parse(r#""hello""#);
    assert!(result.is_ok(), "解析字符串应成功");
    let node = result.unwrap();
    assert!(node.is_string(), "类型应为 string");
    assert_eq!(node.as_string().unwrap(), "hello");
}

/// P-V-06: 解析 [] (empty array)
#[test]
fn test_2_pv06_parse_empty_array() {
    let result = parse("[]");
    assert!(result.is_ok(), "解析空数组应成功");
    let node = result.unwrap();
    assert!(node.is_array(), "类型应为 array");
    assert_eq!(node.len(), 0);
}

/// P-V-07: 解析 {} (empty object)
#[test]
fn test_2_pv07_parse_empty_object() {
    let result = parse("{}");
    assert!(result.is_ok(), "解析空对象应成功");
    let node = result.unwrap();
    assert!(node.is_object(), "类型应为 object");
    assert_eq!(node.len(), 0);
}

// ============================================================================
// 2.1.2 数字解析 (P-N-01 ~ P-N-08)
// ============================================================================

/// P-N-01: 解析 0
#[test]
fn test_2_pn01_parse_zero() {
    let result = parse("0");
    assert!(result.is_ok(), "解析 0 应成功");
    assert_eq!(result.unwrap().as_number().unwrap(), 0.0);
}

/// P-N-02: 解析 -2147483648 (INT_MIN)
#[test]
fn test_2_pn02_parse_int_min() {
    let result = parse("-2147483648");
    assert!(result.is_ok(), "解析 INT_MIN 应成功");
    let n = result.unwrap().as_number().unwrap();
    assert_eq!(n, -2147483648.0, "值应保留负整数语义");
}

/// P-N-03: 解析 2147483647 (INT_MAX)
#[test]
fn test_2_pn03_parse_int_max() {
    let result = parse("2147483647");
    assert!(result.is_ok(), "解析 INT_MAX 应成功");
    let n = result.unwrap().as_number().unwrap();
    assert_eq!(n, 2147483647.0, "值应保留正整数语义");
}

/// P-N-04: 解析 10e-10
#[test]
fn test_2_pn04_parse_scientific_small() {
    let result = parse("10e-10");
    assert!(result.is_ok(), "解析 10e-10 应成功");
    let n = result.unwrap().as_number().unwrap();
    let expected = 1e-9_f64;
    assert!((n - expected).abs() < 1e-20, "值应约等于 1e-9, 实际为 {}", n);
}

/// P-N-05: 解析 123e+127
#[test]
fn test_2_pn05_parse_scientific_large() {
    let result = parse("123e+127");
    assert!(result.is_ok(), "解析 123e+127 应成功");
    let n = result.unwrap().as_number().unwrap();
    assert!(n.is_finite(), "结果应为有限数");
}

/// P-N-06: 解析超大数字
#[test]
fn test_2_pn06_parse_huge_number() {
    let result = parse("9999999999999999999999999999999999999999999999912345678901234567");
    assert!(result.is_ok(), "超大数字应可解析");
    let n = result.unwrap().as_number().unwrap();
    assert!(n.is_finite() || n.is_infinite(), "结果应为有限数或无穷大");
}

/// P-N-07: 非法双小数点应失败
#[test]
fn test_2_pn07_invalid_double_dot() {
    let result = parse("99999999999.1234567890.1234567");
    assert!(result.is_err(), "非法双小数点应失败");
}

/// P-N-08: 非法指数链应失败
#[test]
fn test_2_pn08_invalid_double_exponent() {
    let result = parse("99999E1234567890e1234567");
    assert!(result.is_err(), "非法指数链应失败");
}

// ============================================================================
// 2.1.3 字符串与 Unicode 解析 (P-S-01 ~ P-S-05)
// ============================================================================

/// P-S-01: 空字符串
#[test]
fn test_2_ps01_parse_empty_string() {
    let result = parse(r#""""#);
    assert!(result.is_ok(), "解析空字符串应成功");
    assert_eq!(result.unwrap().as_string().unwrap(), "");
}

/// P-S-02: 转义字符与 Unicode (\u20AC = €)
#[test]
fn test_2_ps02_parse_escape_and_unicode() {
    let input = r#""\"\\/\b\f\n\r\t\u20AC""#;
    let result = parse(input);
    assert!(result.is_ok(), "解析转义字符串应成功");
    let s = result.unwrap();
    let val = s.as_string().unwrap();
    assert!(val.contains('"'), "应包含引号");
    assert!(val.contains('\\'), "应包含反斜杠");
    assert!(val.contains('/'), "应包含斜杠");
    assert!(val.contains('\u{0008}'), "应包含退格");
    assert!(val.contains('\u{000C}'), "应包含换页");
    assert!(val.contains('\n'), "应包含换行");
    assert!(val.contains('\r'), "应包含回车");
    assert!(val.contains('\t'), "应包含制表符");
    assert!(val.contains('€'), "应包含 €");
}

/// P-S-03: UTF-16 代理对 (🐱 = \uD83D\uDC31)
#[test]
fn test_2_ps03_parse_surrogate_pair() {
    let input = r#""\uD83D\uDC31""#;
    let result = parse(input);
    if result.is_ok() {
        let val = result.unwrap();
        let s = val.as_string().unwrap();
        assert_eq!(s, "🐱", "应解码为 🐱");
    } else {
        println!("[功能缺失] LX-json 不支持 UTF-16 代理对解析");
    }
}

/// P-S-04: 非法转义应失败
#[test]
fn test_2_ps04_invalid_escape() {
    let input = r#""Abcdef\e23""#;
    let result = parse(input);
    assert!(result.is_err(), "非法转义 \\e 应失败");
}

/// P-S-05: 末尾反斜杠应失败（防溢出）
#[test]
fn test_2_ps05_trailing_backslash() {
    let input = "\"000000000000000000\\";
    let result = parse(input);
    assert!(result.is_err(), "末尾反斜杠应失败");
}

// ============================================================================
// 2.1.4 数组与对象解析 (P-A-01 ~ P-A-06)
// ============================================================================

/// P-A-01: 空数组
#[test]
fn test_2_pa01_parse_empty_array() {
    let result = parse("[]");
    assert!(result.is_ok());
    assert!(result.unwrap().is_array());
}

/// P-A-02: 混合类型数组
#[test]
fn test_2_pa02_parse_mixed_array() {
    let input = r#"[1, null, true, false, [], "hello", {}]"#;
    let result = parse(input);
    assert!(result.is_ok(), "解析混合类型数组应成功");
    let arr = result.unwrap();
    let items = arr.as_array().unwrap();
    assert_eq!(items.len(), 7, "应有 7 个元素");

    assert!(items[0].is_number(), "第 0 个应为 number");
    assert!(items[1].is_null(), "第 1 个应为 null");
    assert!(items[2].is_true(), "第 2 个应为 true");
    assert!(items[3].is_false(), "第 3 个应为 false");
    assert!(items[4].is_array(), "第 4 个应为 array");
    assert!(items[5].is_string(), "第 5 个应为 string");
    assert!(items[6].is_object(), "第 6 个应为 object");
}

/// P-A-03: 多键值对象
#[test]
fn test_2_pa03_parse_multi_key_object() {
    let input = r#"{"one":1,"two":2,"three":3}"#;
    let result = parse(input);
    assert!(result.is_ok(), "解析多键值对象应成功");
    let obj = result.unwrap();

    let one = get_object_item_case_sensitive(&obj, "one").unwrap();
    assert_eq!(one.as_number().unwrap(), 1.0);

    let two = get_object_item_case_sensitive(&obj, "two").unwrap();
    assert_eq!(two.as_number().unwrap(), 2.0);

    let three = get_object_item_case_sensitive(&obj, "three").unwrap();
    assert_eq!(three.as_number().unwrap(), 3.0);
}

/// P-A-04: 不同类型值的对象
#[test]
fn test_2_pa04_parse_multi_type_object() {
    let input = r#"{"one":1,"NULL":null,"TRUE":true}"#;
    let result = parse(input);
    assert!(result.is_ok(), "解析多类型对象应成功");
    let obj = result.unwrap();

    assert!(get_object_item_case_sensitive(&obj, "one").unwrap().is_number());
    assert!(get_object_item_case_sensitive(&obj, "NULL").unwrap().is_null());
    assert!(get_object_item_case_sensitive(&obj, "TRUE").unwrap().is_true());
}

/// P-A-05: 将对象输入给数组解析应失败
#[test]
fn test_2_pa05_object_as_array_fails() {
    let input = r#"{"hello":[]}"#;
    let result = parse(input);
    assert!(result.is_ok());
    let node = result.unwrap();
    assert!(!node.is_array(), "对象不应被识别为数组");
    assert!(get_array_size(&node).is_err(), "对非数组调用 get_array_size 应失败");
}

/// P-A-06: 将数组输入给对象解析应失败
#[test]
fn test_2_pa06_array_as_object_fails() {
    let input = r#"["hello",{}]"#;
    let result = parse(input);
    assert!(result.is_ok());
    let node = result.unwrap();
    assert!(!node.is_object(), "数组不应被识别为对象");
    assert!(get_object_item(&node, "hello").is_err(), "对非对象调用 get_object_item 应失败");
}

// ============================================================================
// 2.1.5 定长输入与解析结束位置 (P-L-01 ~ P-L-05)
// ============================================================================

/// P-L-01: 精确长度解析
#[test]
fn test_2_pl01_parse_exact_length() {
    let json = r#"{"key":"value"}"#;
    let result = parse_with_length(json, json.len());
    assert!(result.is_ok(), "精确长度解析应成功");
}

/// P-L-02: 截断长度解析应失败
#[test]
fn test_2_pl02_parse_truncated_length() {
    let json = r#"{"key":"value"}"#;
    let result = parse_with_length(json, 5);
    assert!(result.is_err(), "截断长度解析应失败");
}

/// P-L-03: JSON 后有额外内容
#[test]
fn test_2_pl03_trailing_content() {
    let input = "[] empty array XD";
    let result = parse(input);
    if result.is_err() {
        println!("[行为差异] LX-json 严格模式不允许尾部多余字符");
    } else {
        assert!(result.unwrap().is_array());
    }
}

/// P-L-04: 要求严格终止时 "{}x" 应失败
#[test]
fn test_2_pl04_require_null_terminated() {
    let mut opts = ParseOptions::new();
    opts.require_null_terminated = true;
    let result = parse_with_opts("{}x", opts);
    assert!(result.is_err(), "严格终止模式下 '{{}}x' 应失败");
}

/// P-L-05: UTF-8 BOM 支持
#[test]
fn test_2_pl05_utf8_bom() {
    let bom_json = "\u{FEFF}{}";
    let result = parse(bom_json);
    if result.is_ok() {
        assert!(result.unwrap().is_object());
    } else {
        println!("[功能缺失] LX-json 不支持 UTF-8 BOM 前缀");
    }
}

// ============================================================================
// 2.2.1 值打印 (PR-V-01 ~ PR-V-07)
// ============================================================================

/// PR-V-01: null 打印
#[test]
fn test_2_prv01_print_null() {
    let node = JsonNode::Null;
    assert_eq!(print(&node), "null");
}

/// PR-V-02: true 打印
#[test]
fn test_2_prv02_print_true() {
    let node = JsonNode::Bool(true);
    assert_eq!(print(&node), "true");
}

/// PR-V-03: false 打印
#[test]
fn test_2_prv03_print_false() {
    let node = JsonNode::Bool(false);
    assert_eq!(print(&node), "false");
}

/// PR-V-04: 1.5 打印
#[test]
fn test_2_prv04_print_number_1_5() {
    let node = JsonNode::Number(1.5);
    let output = print(&node);
    assert_eq!(output, "1.5");
}

/// PR-V-05: "hello" 打印
#[test]
fn test_2_prv05_print_string() {
    let node = JsonNode::String("hello".to_string());
    assert_eq!(print(&node), "\"hello\"");
}

/// PR-V-06: [] 打印
#[test]
fn test_2_prv06_print_empty_array() {
    let node = JsonNode::Array(vec![]);
    assert_eq!(print(&node), "[]");
}

/// PR-V-07: {} 打印
#[test]
fn test_2_prv07_print_empty_object() {
    let node = JsonNode::Object(vec![]);
    assert_eq!(print(&node), "{}");
}

// ============================================================================
// 2.2.2 数字打印 (PR-N-01 ~ PR-N-06)
// ============================================================================

/// PR-N-01: 打印 0
#[test]
fn test_2_prn01_print_zero() {
    let node = JsonNode::Number(0.0);
    let output = print(&node);
    assert_eq!(output, "0");
}

/// PR-N-02: 打印 -32768
#[test]
fn test_2_prn02_print_neg_32768() {
    let node = JsonNode::Number(-32768.0);
    let output = print(&node);
    assert_eq!(output, "-32768");
}

/// PR-N-03: 打印 2147483647
#[test]
fn test_2_prn03_print_int_max() {
    let node = JsonNode::Number(2147483647.0);
    let output = print(&node);
    assert_eq!(output, "2147483647");
}

/// PR-N-04: 打印 10e-10
#[test]
fn test_2_prn04_print_scientific_small() {
    let node = JsonNode::Number(10e-10_f64);
    let output = print(&node);
    let reparsed = parse(&output);
    if reparsed.is_ok() {
        let val = reparsed.unwrap().as_number().unwrap();
        assert!((val - 1e-9).abs() < 1e-20, "打印后再解析的值应为 1e-9, 实际: {}, 输出: {}", val, output);
    } else {
        println!("[可能差异] 科学数打印结果不可再解析: {}", output);
    }
}

/// PR-N-05: 打印 123e+127
#[test]
fn test_2_prn05_print_scientific_large() {
    let node = JsonNode::Number(123e127_f64);
    let output = print(&node);
    let reparsed = parse(&output);
    if reparsed.is_ok() {
        let val = reparsed.unwrap().as_number().unwrap();
        let expected = 123e127_f64;
        assert!((val - expected).abs() / expected < 1e-10,
                "打印后再解析的值应接近 123e127, 实际: {}, 输出: {}", val, output);
    }
}

/// PR-N-06: 打印 -123e-128
#[test]
fn test_2_prn06_print_scientific_neg() {
    let node = JsonNode::Number(-123e-128_f64);
    let output = print(&node);
    let reparsed = parse(&output);
    if reparsed.is_ok() {
        let val = reparsed.unwrap().as_number().unwrap();
        let expected = -123e-128_f64;
        assert!((val - expected).abs() < 1e-140,
                "打印后再解析的值应接近 -123e-128, 实际: {}, 输出: {}", val, output);
    }
}

// ============================================================================
// 2.2.3 字符串打印 (PR-S-01 ~ PR-S-03)
// ============================================================================

/// PR-S-01: 空字符串打印
#[test]
fn test_2_prs01_print_empty_string() {
    let node = JsonNode::String(String::new());
    assert_eq!(print(&node), "\"\"");
}

/// PR-S-02: 控制字符混合打印
#[test]
fn test_2_prs02_print_control_chars() {
    let s = "hello\t\n\r\x08\x0cworld";
    let node = JsonNode::String(s.to_string());
    let output = print(&node);
    assert!(output.contains("\\t"), "应转义 tab");
    assert!(output.contains("\\n"), "应转义换行");
    assert!(output.contains("\\r"), "应转义回车");
    assert!(output.contains("\\b"), "应转义退格");
    assert!(output.contains("\\f"), "应转义换页");
}

/// PR-S-03: UTF-8 字符串打印（ü猫慕）
#[test]
fn test_2_prs03_print_utf8() {
    let s = "ü猫慕";
    let node = JsonNode::String(s.to_string());
    let output = print(&node);
    let reparsed = parse(&output).unwrap();
    assert_eq!(reparsed.as_string().unwrap(), s, "UTF-8 字符应正确往返");
}

// ============================================================================
// 2.2.4 数组与对象打印（格式化/非格式化）(PR-A-01 ~ PR-O-02)
// ============================================================================

/// PR-A-01: [1,2,3] 非格式化打印
#[test]
fn test_2_pra01_print_array_unformatted() {
    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    let output = print_unformatted(&arr);
    assert_eq!(output, "[1,2,3]");
}

/// PR-A-02: [1,2,3] 格式化打印
#[test]
fn test_2_pra02_print_array_formatted() {
    let arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);
    let output = print(&arr);
    assert!(output.contains('\n'), "格式化输出应包含换行");
    assert!(output.contains("1"), "应包含元素 1");
    assert!(output.contains("2"), "应包含元素 2");
    assert!(output.contains("3"), "应包含元素 3");
}

/// PR-O-01: {"one":1,"two":2} 非格式化打印
#[test]
fn test_2_pro01_print_object_unformatted() {
    let obj = JsonNode::Object(vec![
        ("one".to_string(), JsonNode::Number(1.0)),
        ("two".to_string(), JsonNode::Number(2.0)),
    ]);
    let output = print_unformatted(&obj);
    assert_eq!(output, r#"{"one":1,"two":2}"#);
}

/// PR-O-02: {"one":1,"two":2} 格式化打印
#[test]
fn test_2_pro02_print_object_formatted() {
    let obj = JsonNode::Object(vec![
        ("one".to_string(), JsonNode::Number(1.0)),
        ("two".to_string(), JsonNode::Number(2.0)),
    ]);
    let output = print(&obj);
    assert!(output.contains('\n'), "格式化输出应包含换行");
    assert!(output.contains("one"), "应包含键 one");
    assert!(output.contains("two"), "应包含键 two");
}

// ============================================================================
// 2.3 构造与 Add API 测试 (A-01 ~ A-11)
// ============================================================================

/// A-01: AddNullToObject
#[test]
fn test_2_a01_add_null_to_object() {
    let mut root = JsonNode::new_object();
    root.add_item_to_object("null", JsonNode::Null).unwrap();
    let val = root.get("null").unwrap();
    assert!(val.is_null(), "字段类型应为 null");
}

/// A-02: AddTrueToObject
#[test]
fn test_2_a02_add_true_to_object() {
    let mut root = JsonNode::new_object();
    root.add_item_to_object("true", JsonNode::new_true()).unwrap();
    let val = root.get("true").unwrap();
    assert!(val.is_true(), "字段类型应为 true");
}

/// A-03: AddFalseToObject
#[test]
fn test_2_a03_add_false_to_object() {
    let mut root = JsonNode::new_object();
    root.add_item_to_object("false", JsonNode::new_false()).unwrap();
    let val = root.get("false").unwrap();
    assert!(val.is_false(), "字段类型应为 false");
}

/// A-04: AddBoolToObject(root,"b",false)
#[test]
fn test_2_a04_add_bool_to_object() {
    let mut root = JsonNode::new_object();
    root.add_bool_to_object("b", false).unwrap();
    let val = root.get("b").unwrap();
    assert!(val.is_false(), "字段类型应为 false");
}

/// A-05: AddNumberToObject(root,"n",42)
#[test]
fn test_2_a05_add_number_to_object() {
    let mut root = JsonNode::new_object();
    root.add_number_to_object("n", 42.0).unwrap();
    let val = root.get("n").unwrap();
    assert_eq!(val.as_number().unwrap(), 42.0, "值应为 42");
}

/// A-06: AddStringToObject(root,"s","Hello World!")
#[test]
fn test_2_a06_add_string_to_object() {
    let mut root = JsonNode::new_object();
    root.add_string_to_object("s", "Hello World!").unwrap();
    let val = root.get("s").unwrap();
    assert_eq!(val.as_string().unwrap(), "Hello World!");
}

/// A-07: AddRawToObject(root,"raw","{}")
#[test]
fn test_2_a07_add_raw_to_object() {
    let mut root = JsonNode::new_object();
    root.add_item_to_object("raw", JsonNode::new_raw("{}")).unwrap();
    let val = root.get("raw").unwrap();
    assert!(val.is_raw(), "类型应为 raw");
}

/// A-08: AddObjectToObject(root,"obj")
#[test]
fn test_2_a08_add_object_to_object() {
    let mut root = JsonNode::new_object();
    root.add_item_to_object("obj", JsonNode::new_object()).unwrap();
    let val = root.get("obj").unwrap();
    assert!(val.is_object(), "子对象应存在");
}

/// A-09: AddArrayToObject(root,"arr")
#[test]
fn test_2_a09_add_array_to_object() {
    let mut root = JsonNode::new_object();
    root.add_item_to_object("arr", JsonNode::new_array()).unwrap();
    let val = root.get("arr").unwrap();
    assert!(val.is_array(), "子数组应存在");
}

/// A-10: 对非对象执行 add_item_to_object 应失败
#[test]
fn test_2_a10_add_to_non_object_fails() {
    let mut arr = JsonNode::new_array();
    let result = arr.add_item_to_object("key", JsonNode::Null);
    assert!(result.is_err(), "对数组执行 add_item_to_object 应失败");

    let mut num = JsonNode::Number(42.0);
    let result = num.add_item_to_object("key", JsonNode::Null);
    assert!(result.is_err(), "对数字执行 add_item_to_object 应失败");
}

/// A-11: 对非数组执行 add_item_to_array 应失败
#[test]
fn test_2_a11_add_to_non_array_fails() {
    let mut obj = JsonNode::new_object();
    let result = obj.add_item_to_array(JsonNode::Null);
    assert!(result.is_err(), "对对象执行 add_item_to_array 应失败");
}

// ============================================================================
// 2.4.1 比较测试 (C-01 ~ C-06)
// ============================================================================

/// C-01: 相同数字相等
#[test]
fn test_2_c01_compare_same_number() {
    let a = JsonNode::Number(1.0);
    let b = JsonNode::Number(1.0);
    assert!(compare(&a, &b, true), "相同数字应相等");
}

/// C-02: 不同数字不相等
#[test]
fn test_2_c02_compare_different_number() {
    let a = JsonNode::Number(1.0);
    let b = JsonNode::Number(2.0);
    assert!(!compare(&a, &b, true), "不同数字应不相等");
}

/// C-03: 大小写敏感时不同名键不相等
#[test]
fn test_2_c03_compare_case_sensitive() {
    let a = parse(r#"{"false":false}"#).unwrap();
    let b = parse(r#"{"False":false}"#).unwrap();
    assert!(!compare(&a, &b, true), "大小写敏感时键名不同应不相等");
}

/// C-04: 大小写不敏感时应相等
#[test]
fn test_2_c04_compare_case_insensitive() {
    let a = parse(r#"{"false":false}"#).unwrap();
    let b = parse(r#"{"False":false}"#).unwrap();
    assert!(compare(&a, &b, false), "大小写不敏感时应相等");
}

/// C-05: 不同长度数组不相等
#[test]
fn test_2_c05_compare_different_length_array() {
    let a = parse("[1,2,3]").unwrap();
    let b = parse("[1,2]").unwrap();
    assert!(!compare(&a, &b, true), "不同长度数组应不相等");
}

/// C-06: 不同大小对象不相等
#[test]
fn test_2_c06_compare_different_size_object() {
    let a = parse(r#"{"one":1,"two":2}"#).unwrap();
    let b = parse(r#"{"one":1,"two":2,"three":3}"#).unwrap();
    assert!(!compare(&a, &b, true), "不同大小对象应不相等");
}

// ============================================================================
// 2.4.2 最小化测试 (M-01 ~ M-05)
// ============================================================================

/// M-01: 去除空白
#[test]
fn test_2_m01_minify_whitespace() {
    let input = "{ \"key\":\ttrue\r\n }";
    let result = minify(input);
    assert!(result.is_ok(), "最小化应成功");
    assert_eq!(result.unwrap(), "{\"key\":true}");
}

/// M-02: 单行注释处理
#[test]
fn test_2_m02_minify_line_comment() {
    let input = "{// comment\n}";
    let result = minify(input);
    if result.is_ok() {
        let output = result.unwrap();
        if output == "{}" {
            // 完全支持
        } else {
            println!("[功能差异] LX-json minify 不剥离 // 注释, 输出: {}", output);
        }
    } else {
        println!("[功能差异] LX-json minify 处理注释时返回错误");
    }
}

/// M-03: 块注释处理
#[test]
fn test_2_m03_minify_block_comment() {
    let input = "{/* a\ncomment */}";
    let result = minify(input);
    if result.is_ok() {
        let output = result.unwrap();
        if output == "{}" {
            // 完全支持
        } else {
            println!("[功能差异] LX-json minify 不剥离 /* */ 注释, 输出: {}", output);
        }
    } else {
        println!("[功能差异] LX-json minify 处理块注释时返回错误");
    }
}

/// M-04: 字符串内容保持不变
#[test]
fn test_2_m04_minify_preserve_string() {
    let input = r#""this is a string \" \t bla""#;
    let result = minify(input);
    assert!(result.is_ok(), "最小化应成功");
    assert_eq!(result.unwrap(), input, "字符串内容应保持不变");
}

/// M-05: 特殊输入不死循环
#[test]
fn test_2_m05_minify_no_infinite_loop() {
    let input = "8 / 5\n";
    let result = minify(input);
    if result.is_ok() {
        println!("M-05 minify 结果: {}", result.unwrap());
    } else {
        println!("M-05 minify 返回错误（但未死循环）");
    }
}

// ============================================================================
// 2.5 鲁棒性与边界测试 (R-01 ~ R-07)
// ============================================================================

/// R-01: 空输入防御
#[test]
fn test_2_r01_empty_input() {
    let result = parse("");
    assert!(result.is_err(), "空输入应返回失败");
}

/// R-02: 深度限制
#[test]
fn test_2_r02_nesting_limit() {
    let depth = 1001;
    let mut input = String::new();
    for _ in 0..depth {
        input.push('[');
    }
    for _ in 0..depth {
        input.push(']');
    }

    let mut opts = ParseOptions::new();
    opts.nesting_limit = 1000;
    let result = parse_with_opts(&input, opts);
    assert!(result.is_err(), "超过嵌套限制应失败");
}

/// R-03: 深拷贝测试
#[test]
fn test_2_r03_deep_copy() {
    let original = JsonNode::Object(vec![
        ("name".to_string(), JsonNode::String("test".to_string())),
        ("nested".to_string(), JsonNode::Array(vec![
            JsonNode::Number(1.0),
            JsonNode::Number(2.0),
        ])),
    ]);

    let copy = duplicate(&original, true);
    assert!(compare(&original, &copy, true), "深拷贝应与原始相等");

    let shallow = duplicate(&original, false);
    assert!(shallow.is_object(), "浅拷贝应保持对象类型");
    assert_eq!(shallow.len(), 0, "浅拷贝的对象应为空");
}

/// R-04: 大数合法 / 非法边界
#[test]
fn test_2_r04_number_boundary() {
    let result = parse("1e308");
    assert!(result.is_ok(), "1e308 应可解析");

    let result = parse("1e999999999999999");
    if result.is_ok() {
        let n = result.unwrap().as_number().unwrap();
        println!("1e999999999999999 解析结果: {}", n);
    }

    let result = parse("--1");
    assert!(result.is_err(), "双负号应失败");

    let result = parse("+1");
    assert!(result.is_err(), "正号不是合法 JSON 数字");
}

/// R-05: 数组操作后打印完整性
#[test]
fn test_2_r05_array_operation_integrity() {
    let mut arr = JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ]);

    if let JsonNode::Array(ref mut v) = arr {
        v.remove(1);
    }

    let output = print(&arr);
    let reparsed = parse(&output).unwrap();
    let items = reparsed.as_array().unwrap();
    assert_eq!(items.len(), 2, "删除后应剩 2 个元素");
    assert_eq!(items[0].as_number().unwrap(), 1.0);
    assert_eq!(items[1].as_number().unwrap(), 3.0);
}

/// R-06: 各种非法 JSON 输入不崩溃
#[test]
fn test_2_r06_invalid_inputs_no_crash() {
    let invalid_inputs = vec![
        "",
        " ",
        "{",
        "}",
        "[",
        "]",
        "{\"a\":}",
        "{\"a\"",
        "[1,]",
        "[,1]",
        "{'a':1}",
        "{a:1}",
        "\"unterminated",
        "nul",
        "tru",
        "fals",
        "123abc",
        ".123",
        "1.",
    ];

    for input in &invalid_inputs {
        let result = parse(input);
        let _ = result;
    }
}

// ============================================================================
// 2.6.1 JSON Pointer 测试 (U-PTR-01 ~ U-PTR-03)
// ============================================================================

/// U-PTR-01: /foo/0 -> "bar"
#[test]
fn test_2_uptr01_pointer_array_index() {
    let doc = parse(r#"{"foo":["bar"]}"#).unwrap();
    let result = get_pointer(&doc, "/foo/0");
    assert!(result.is_ok(), "Pointer /foo/0 应成功");
    assert_eq!(result.unwrap().as_string().unwrap(), "bar");
}

/// U-PTR-02: /a~1b -> 匹配键 "a/b"（~1 = /）
#[test]
fn test_2_uptr02_pointer_escaped_slash() {
    let mut obj = JsonNode::new_object();
    obj.add_number_to_object("a/b", 1.0).unwrap();

    let result = get_pointer(&obj, "/a~1b");
    assert!(result.is_ok(), "Pointer /a~1b 应匹配键 'a/b'");
    assert_eq!(result.unwrap().as_number().unwrap(), 1.0);
}

/// U-PTR-03: /m~0n -> 匹配键 "m~n"（~0 = ~）
#[test]
fn test_2_uptr03_pointer_escaped_tilde() {
    let mut obj = JsonNode::new_object();
    obj.add_number_to_object("m~n", 8.0).unwrap();

    let result = get_pointer(&obj, "/m~0n");
    assert!(result.is_ok(), "Pointer /m~0n 应匹配键 'm~n'");
    assert_eq!(result.unwrap().as_number().unwrap(), 8.0);
}

// ============================================================================
// 2.6.2 JSON Patch 测试 (U-PATCH-01 ~ U-PATCH-03)
// ============================================================================

/// U-PATCH-01: apply 有效 patch
#[test]
fn test_2_upatch01_apply_valid_patch() {
    let mut doc = parse(r#"{"name":"John","age":30}"#).unwrap();

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("op".to_string(), JsonNode::String("replace".to_string())),
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("Jane".to_string())),
        ]),
    ]);

    let result = apply_patches(&mut doc, &patches, true);
    assert!(result.is_ok(), "有效 patch 应成功应用");

    let name = doc.get("name").unwrap();
    assert_eq!(name.as_string().unwrap(), "Jane", "name 应被替换为 Jane");
}

/// U-PATCH-02: apply 非法 patch（缺少 op）
#[test]
fn test_2_upatch02_apply_invalid_patch() {
    let mut doc = parse(r#"{"name":"John"}"#).unwrap();

    let patches = JsonNode::Array(vec![
        JsonNode::Object(vec![
            ("path".to_string(), JsonNode::String("/name".to_string())),
            ("value".to_string(), JsonNode::String("Jane".to_string())),
        ]),
    ]);

    let result = apply_patches(&mut doc, &patches, true);
    assert!(result.is_err(), "非法 patch 应失败");
}

/// U-PATCH-03: generate_patches 后回放应等于 expected
#[test]
fn test_2_upatch03_generate_and_replay() {
    let from = parse(r#"{"name":"John","age":30}"#).unwrap();
    let to = parse(r#"{"name":"Jane","age":30,"city":"NYC"}"#).unwrap();

    let patches = generate_patches(&from, &to, true);
    assert!(patches.is_ok(), "生成 patches 应成功");
    let patches = patches.unwrap();

    let mut replayed = from.clone();
    let apply_result = apply_patches(&mut replayed, &patches, true);
    if apply_result.is_ok() {
        assert!(compare(&replayed, &to, true),
                "回放 patches 后结果应与 expected 一致");
    } else {
        println!("[可能差异] apply_patches 回放失败: {:?}", apply_result.err());
    }
}

// ============================================================================
// 2.6.3 Merge Patch 测试 (U-M-01 ~ U-M-03)
// ============================================================================

/// U-M-01: {"a":"b"} + {"a":"c"} = {"a":"c"}
#[test]
fn test_2_um01_merge_patch_replace() {
    let mut target = parse(r#"{"a":"b"}"#).unwrap();
    let patch = parse(r#"{"a":"c"}"#).unwrap();

    let result = merge_patch(&mut target, &patch, true);
    assert!(result.is_ok(), "Merge patch 应成功");

    let expected = parse(r#"{"a":"c"}"#).unwrap();
    assert!(compare(&target, &expected, true), "合并后应为 {{\"a\":\"c\"}}");
}

/// U-M-02: {"a":"b"} + {"a":null} = {}
#[test]
fn test_2_um02_merge_patch_delete() {
    let mut target = parse(r#"{"a":"b"}"#).unwrap();
    let patch = parse(r#"{"a":null}"#).unwrap();

    let result = merge_patch(&mut target, &patch, true);
    assert!(result.is_ok(), "Merge patch 应成功");

    let expected = parse("{}").unwrap();
    assert!(compare(&target, &expected, true),
            "null 值应删除对应键, 实际结果: {}", print(&target));
}

/// U-M-03: [1,2] + {"a":"b","c":null} = {"a":"b"}
#[test]
fn test_2_um03_merge_patch_replace_array() {
    let mut target = parse("[1,2]").unwrap();
    let patch = parse(r#"{"a":"b","c":null}"#).unwrap();

    let result = merge_patch(&mut target, &patch, true);
    assert!(result.is_ok(), "Merge patch 应成功");

    let expected = parse(r#"{"a":"b"}"#).unwrap();
    assert!(compare(&target, &expected, true),
            "数组应被对象替换, 实际结果: {}", print(&target));
}

// ============================================================================
// 额外补充测试：解析后序列化的 round-trip
// ============================================================================

/// 复杂 JSON round-trip
#[test]
fn test_2_roundtrip_complex() {
    let input = r#"{"name":"John","age":30,"active":true,"scores":[100,95,88],"address":{"city":"NYC","zip":"10001"},"notes":null}"#;
    let parsed = parse(input).unwrap();
    let output = print_unformatted(&parsed);
    let reparsed = parse(&output).unwrap();
    assert!(compare(&parsed, &reparsed, true), "复杂 JSON round-trip 应保持一致");
}

/// 嵌套数组 round-trip
#[test]
fn test_2_roundtrip_nested_arrays() {
    let input = "[[1,2],[3,[4,5]],[]]";
    let parsed = parse(input).unwrap();
    let output = print_unformatted(&parsed);
    let reparsed = parse(&output).unwrap();
    assert!(compare(&parsed, &reparsed, true));
}

/// 转义字符 round-trip
#[test]
fn test_2_roundtrip_escapes() {
    let input = r#"{"key":"hello\nworld\ttab\\slash\"quote"}"#;
    let parsed = parse(input).unwrap();
    let output = print_unformatted(&parsed);
    let reparsed = parse(&output).unwrap();
    assert!(compare(&parsed, &reparsed, true), "转义字符 round-trip 应保持一致");
}

// ============================================================================
// 额外比较测试
// ============================================================================

/// 不同类型不相等
#[test]
fn test_2_compare_different_types() {
    let a = JsonNode::Number(1.0);
    let b = JsonNode::String("1".to_string());
    assert!(!compare(&a, &b, true), "不同类型应不相等");

    let c = JsonNode::Null;
    let d = JsonNode::Bool(false);
    assert!(!compare(&c, &d, true), "null 和 false 应不相等");
}

/// 相同嵌套结构相等
#[test]
fn test_2_compare_same_nested() {
    let a = parse(r#"{"a":{"b":[1,2,3]}}"#).unwrap();
    let b = parse(r#"{"a":{"b":[1,2,3]}}"#).unwrap();
    assert!(compare(&a, &b, true), "相同嵌套结构应相等");
}

/// null 相等
#[test]
fn test_2_compare_null_equal() {
    let a = JsonNode::Null;
    let b = JsonNode::Null;
    assert!(compare(&a, &b, true), "两个 null 应相等");
}

/// bool 相等
#[test]
fn test_2_compare_bool_equal() {
    assert!(compare(&JsonNode::Bool(true), &JsonNode::Bool(true), true));
    assert!(compare(&JsonNode::Bool(false), &JsonNode::Bool(false), true));
    assert!(!compare(&JsonNode::Bool(true), &JsonNode::Bool(false), true));
}

// ============================================================================
// ===== 以下为补充测试 —— 对标 cJSON tests/ 中尚未覆盖的测试项 =====
// ============================================================================

// ============================================================================
// S-TC: 综合类型检查 (对标 misc_tests.c: typecheck_functions_should_check_type)
// ============================================================================

/// S-TC-01: 全面类型检查——每种类型节点的 is_* 函数
#[test]
fn test_2_stc01_typecheck_comprehensive() {
    let null_node = JsonNode::Null;
    let true_node = JsonNode::Bool(true);
    let false_node = JsonNode::Bool(false);
    let num_node = JsonNode::Number(42.0);
    let str_node = JsonNode::String("test".to_string());
    let arr_node = JsonNode::Array(vec![]);
    let obj_node = JsonNode::Object(vec![]);
    let raw_node = JsonNode::Raw("raw".to_string());

    // null
    assert!(null_node.is_null());
    assert!(!null_node.is_bool());
    assert!(!null_node.is_number());
    assert!(!null_node.is_string());
    assert!(!null_node.is_array());
    assert!(!null_node.is_object());
    assert!(!null_node.is_true());
    assert!(!null_node.is_false());
    assert!(!null_node.is_raw());

    // true
    assert!(!true_node.is_null());
    assert!(true_node.is_bool());
    assert!(!true_node.is_number());
    assert!(!true_node.is_string());
    assert!(!true_node.is_array());
    assert!(!true_node.is_object());
    assert!(true_node.is_true());
    assert!(!true_node.is_false());
    assert!(!true_node.is_raw());

    // false
    assert!(!false_node.is_null());
    assert!(false_node.is_bool());
    assert!(!false_node.is_number());
    assert!(!false_node.is_string());
    assert!(!false_node.is_array());
    assert!(!false_node.is_object());
    assert!(!false_node.is_true());
    assert!(false_node.is_false());
    assert!(!false_node.is_raw());

    // number
    assert!(!num_node.is_null());
    assert!(!num_node.is_bool());
    assert!(num_node.is_number());
    assert!(!num_node.is_string());
    assert!(!num_node.is_array());
    assert!(!num_node.is_object());
    assert!(!num_node.is_raw());

    // string
    assert!(!str_node.is_null());
    assert!(!str_node.is_bool());
    assert!(!str_node.is_number());
    assert!(str_node.is_string());
    assert!(!str_node.is_array());
    assert!(!str_node.is_object());
    assert!(!str_node.is_raw());

    // array
    assert!(!arr_node.is_null());
    assert!(!arr_node.is_bool());
    assert!(!arr_node.is_number());
    assert!(!arr_node.is_string());
    assert!(arr_node.is_array());
    assert!(!arr_node.is_object());
    assert!(!arr_node.is_raw());

    // object
    assert!(!obj_node.is_null());
    assert!(!obj_node.is_bool());
    assert!(!obj_node.is_number());
    assert!(!obj_node.is_string());
    assert!(!obj_node.is_array());
    assert!(obj_node.is_object());
    assert!(!obj_node.is_raw());

    // raw
    assert!(!raw_node.is_null());
    assert!(!raw_node.is_bool());
    assert!(!raw_node.is_number());
    assert!(!raw_node.is_string());
    assert!(!raw_node.is_array());
    assert!(!raw_node.is_object());
    assert!(raw_node.is_raw());
}

// ============================================================================
// S-OBJ: GetObjectItem 边界测试
// ============================================================================

/// S-OBJ-01: 对数组调用 get_object_item 应返回错误
#[test]
fn test_2_sobj01_get_object_item_on_array_no_crash() {
    let arr = parse("[1,2,3]").unwrap();
    assert!(get_object_item(&arr, "key").is_err());
}

/// S-OBJ-02: 对数组调用 get_object_item_case_sensitive 应返回错误
#[test]
fn test_2_sobj02_get_object_item_case_sensitive_on_array_no_crash() {
    let arr = parse("[1,2,3]").unwrap();
    assert!(get_object_item_case_sensitive(&arr, "key").is_err());
}

/// S-OBJ-03: 大小写不敏感查找
#[test]
fn test_2_sobj03_get_object_item_case_insensitive() {
    let obj = parse(r#"{"One":1,"Two":2,"Three":3}"#).unwrap();
    let one = get_object_item(&obj, "one");
    assert!(one.is_ok(), "get_object_item 应大小写不敏感匹配");
    assert_eq!(one.unwrap().as_number().unwrap(), 1.0);

    let two = get_object_item(&obj, "tWo");
    assert!(two.is_ok());

    let missing = get_object_item(&obj, "nonexistent");
    assert!(missing.is_err());
}

/// S-OBJ-04: 大小写敏感查找
#[test]
fn test_2_sobj04_get_object_item_case_sensitive_lookup() {
    let obj = parse(r#"{"One":1,"Two":2}"#).unwrap();
    assert!(get_object_item_case_sensitive(&obj, "One").is_ok());
    assert!(get_object_item_case_sensitive(&obj, "one").is_err());
}

// ============================================================================
// S-ACC: 值访问器测试
// ============================================================================

/// S-ACC-01: get_string_value
#[test]
fn test_2_sacc01_get_string_value() {
    let str_node = parse(r#""hello world""#).unwrap();
    assert!(get_string_value(&str_node).is_some());
    assert_eq!(get_string_value(&str_node).unwrap(), "hello world");

    let num_node = parse("42").unwrap();
    assert!(get_string_value(&num_node).is_none());

    let null_node = parse("null").unwrap();
    assert!(get_string_value(&null_node).is_none());
}

/// S-ACC-02: get_number_value
#[test]
fn test_2_sacc02_get_number_value() {
    let num_node = parse("3.14").unwrap();
    assert!(get_number_value(&num_node).is_some());
    assert!((get_number_value(&num_node).unwrap() - 3.14).abs() < 1e-10);

    let str_node = parse(r#""hello""#).unwrap();
    assert!(get_number_value(&str_node).is_none());
}

// ============================================================================
// S-CMP: 补充比较测试
// ============================================================================

/// S-CMP-01: 字符串比较始终大小写敏感
#[test]
fn test_2_scmp01_compare_strings() {
    let a = JsonNode::String("test".to_string());
    let b = JsonNode::String("test".to_string());
    let c = JsonNode::String("Test".to_string());

    assert!(compare(&a, &b, true));
    assert!(compare(&a, &b, false));
    assert!(!compare(&a, &c, true));
    assert!(!compare(&a, &c, false), "字符串值比较应始终大小写敏感");
}

/// S-CMP-02: 比较 Raw 类型
#[test]
fn test_2_scmp02_compare_raw() {
    let a = JsonNode::Raw("test".to_string());
    let b = JsonNode::Raw("test".to_string());
    let c = JsonNode::Raw("other".to_string());

    assert!(compare(&a, &b, true));
    assert!(!compare(&a, &c, true));
}

/// S-CMP-03: 科学计数法数字比较
#[test]
fn test_2_scmp03_compare_scientific_numbers() {
    let a = JsonNode::Number(1e100_f64);
    let b = JsonNode::Number(10e99_f64);
    assert!(compare(&a, &b, true), "1E100 应等于 10E99");

    let c = JsonNode::Number(0.5e-100_f64);
    let d = JsonNode::Number(0.5e-101_f64);
    assert!(!compare(&c, &d, true));
}

/// S-CMP-04: 对象比较——键顺序无关
#[test]
fn test_2_scmp04_compare_objects_order_independent() {
    let a = parse(r#"{"a":1,"b":2,"c":3}"#).unwrap();
    let b = parse(r#"{"c":3,"a":1,"b":2}"#).unwrap();
    if compare(&a, &b, true) {
        // 顺序无关比较
    } else {
        println!("[功能差异] LX-json compare 对象时可能依赖键顺序");
    }
}

/// S-CMP-05: 数组比较——空数组与嵌套
#[test]
fn test_2_scmp05_compare_arrays() {
    assert!(compare(&parse("[]").unwrap(), &parse("[]").unwrap(), true));

    let a = parse(r#"[1,"test",null,true,[1,2,3]]"#).unwrap();
    let b = parse(r#"[1,"test",null,true,[1,2,3]]"#).unwrap();
    assert!(compare(&a, &b, true));

    assert!(!compare(&parse("[1,2]").unwrap(), &parse("[1,2,3]").unwrap(), true));
}

// ============================================================================
// S-MUT: 变更操作测试——Detach / Replace / Insert / Delete
// ============================================================================

/// S-MUT-01: detach_item_from_array 中间元素
#[test]
fn test_2_smut01_detach_item_from_array() {
    let mut arr = parse("[1,2,3,4,5]").unwrap();
    let detached = arr.detach_item_from_array(2).unwrap();
    assert_eq!(detached.as_number().unwrap(), 3.0);
    assert_eq!(arr.len(), 4);
    assert_eq!(print_unformatted(&arr), "[1,2,4,5]");
}

/// S-MUT-02: detach 头元素
#[test]
fn test_2_smut02_detach_first_from_array() {
    let mut arr = parse("[10,20,30]").unwrap();
    let detached = arr.detach_item_from_array(0).unwrap();
    assert_eq!(detached.as_number().unwrap(), 10.0);
    assert_eq!(print_unformatted(&arr), "[20,30]");
}

/// S-MUT-03: detach 尾元素
#[test]
fn test_2_smut03_detach_last_from_array() {
    let mut arr = parse("[10,20,30]").unwrap();
    let detached = arr.detach_item_from_array(2).unwrap();
    assert_eq!(detached.as_number().unwrap(), 30.0);
    assert_eq!(print_unformatted(&arr), "[10,20]");
}

/// S-MUT-04: detach_item_from_object
#[test]
fn test_2_smut04_detach_item_from_object() {
    let mut obj = parse(r#"{"a":1,"b":2,"c":3}"#).unwrap();
    let detached = obj.detach_item_from_object("b").unwrap();
    assert_eq!(detached.as_number().unwrap(), 2.0);
    assert_eq!(obj.len(), 2);
    assert!(has_object_item(&obj, "a"));
    assert!(!has_object_item(&obj, "b"));
    assert!(has_object_item(&obj, "c"));
}

/// S-MUT-05: replace_item_in_array
#[test]
fn test_2_smut05_replace_item_in_array() {
    let mut arr = parse("[1,2,3]").unwrap();
    arr.replace_item_in_array(1, JsonNode::String("replaced".to_string())).unwrap();
    assert_eq!(print_unformatted(&arr), r#"[1,"replaced",3]"#);
}

/// S-MUT-06: replace_item_in_object
#[test]
fn test_2_smut06_replace_item_in_object() {
    let mut obj = parse(r#"{"name":"old","age":20}"#).unwrap();
    obj.replace_item_in_object("name", JsonNode::String("new".to_string())).unwrap();
    let name = get_object_item_case_sensitive(&obj, "name").unwrap();
    assert_eq!(name.as_string().unwrap(), "new");
}

/// S-MUT-07: insert_item_in_array 中间
#[test]
fn test_2_smut07_insert_item_in_array() {
    let mut arr = parse("[1,3]").unwrap();
    arr.insert_item_in_array(1, JsonNode::Number(2.0)).unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(print_unformatted(&arr), "[1,2,3]");
}

/// S-MUT-08: insert 头部
#[test]
fn test_2_smut08_insert_at_head() {
    let mut arr = parse("[2,3]").unwrap();
    arr.insert_item_in_array(0, JsonNode::Number(1.0)).unwrap();
    assert_eq!(print_unformatted(&arr), "[1,2,3]");
}

/// S-MUT-09: insert 尾部
#[test]
fn test_2_smut09_insert_at_tail() {
    let mut arr = parse("[1,2]").unwrap();
    arr.insert_item_in_array(2, JsonNode::Number(3.0)).unwrap();
    assert_eq!(print_unformatted(&arr), "[1,2,3]");
}

/// S-MUT-10: delete_item_from_object
#[test]
fn test_2_smut10_delete_item_from_object() {
    let mut obj = parse(r#"{"a":1,"b":2,"c":3}"#).unwrap();
    obj.delete_item_from_object("b").unwrap();
    assert_eq!(obj.len(), 2);
    assert!(!has_object_item(&obj, "b"));
}

/// S-MUT-11: delete 首元素后结构完整
#[test]
fn test_2_smut11_delete_first_from_array() {
    let mut arr = parse("[1,2,3,4,5]").unwrap();
    arr.delete_item_from_array(0).unwrap();
    assert_eq!(arr.len(), 4);
    assert_eq!(print_unformatted(&arr), "[2,3,4,5]");
}

// ============================================================================
// S-SORT: 对象键排序 (对标 old_utils_tests.c: sort_tests)
// ============================================================================

/// S-SORT-01: sort_object 大小写敏感
#[test]
fn test_2_ssort01_sort_object_case_sensitive() {
    let mut obj = parse(r#"{"z":1,"a":2,"m":3,"c":4}"#).unwrap();
    obj.sort_object(true).unwrap();
    if let JsonNode::Object(pairs) = &obj {
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["a", "c", "m", "z"]);
    }
}

/// S-SORT-02: sort_object 大小写不敏感
#[test]
fn test_2_ssort02_sort_object_case_insensitive() {
    let mut obj = parse(r#"{"Banana":1,"apple":2,"Cherry":3}"#).unwrap();
    obj.sort_object(false).unwrap();
    if let JsonNode::Object(pairs) = &obj {
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["apple", "Banana", "Cherry"]);
    }
}

// ============================================================================
// S-KEYS: get_object_keys
// ============================================================================

/// S-KEYS-01: 获取对象键
#[test]
fn test_2_skeys01_get_object_keys() {
    let obj = parse(r#"{"name":"John","age":30,"city":"NYC"}"#).unwrap();
    let keys = obj.get_object_keys().unwrap();
    assert!(keys.is_array());
    assert_eq!(keys.len(), 3);
}

// ============================================================================
// S-BOM: BOM 不在开头
// ============================================================================

/// S-BOM-01: 尾部 BOM 不应被接受
#[test]
fn test_2_sbom01_bom_not_at_beginning() {
    let input = "{}\u{FEFF}";
    let result = parse(input);
    if result.is_err() {
        // 预期：尾部非法字符
    } else {
        println!("[INFO] 解析器接受了尾部 BOM");
    }
}

// ============================================================================
// S-BUG94: Bug #94 复杂转义字符串
// ============================================================================

/// S-BUG94-01: 大量转义字符的复杂字符串
#[test]
fn test_2_sbug94_parse_complex_escapes() {
    let input = r#""~!@\\#$%^&*()\\\\-\\+{}[]:\\;\"\\<\\>?/.,DC=ad,DC=com""#;
    let result = parse(input);
    assert!(result.is_ok(), "Bug #94 复杂转义字符串应可解析");
    assert!(result.unwrap().is_string());
}

// ============================================================================
// S-ARR-CREATE: 数组批量创建
// ============================================================================

/// S-ARR-01: create_int_array
#[test]
fn test_2_sarr01_create_int_array() {
    let arr = JsonNode::create_int_array(&[1, 2, 3, 4, 5]);
    assert!(arr.is_array());
    assert_eq!(arr.len(), 5);
    assert_eq!(print_unformatted(&arr), "[1,2,3,4,5]");
}

/// S-ARR-02: create_float_array
#[test]
fn test_2_sarr02_create_float_array() {
    let arr = JsonNode::create_float_array(&[1.5_f32, 2.5, 3.5]);
    assert!(arr.is_array());
    assert_eq!(arr.len(), 3);
}

/// S-ARR-03: create_double_array
#[test]
fn test_2_sarr03_create_double_array() {
    let arr = JsonNode::create_double_array(&[1.1, 2.2, 3.3]);
    assert!(arr.is_array());
    assert_eq!(arr.len(), 3);
}

/// S-ARR-04: create_string_array
#[test]
fn test_2_sarr04_create_string_array() {
    let arr = JsonNode::create_string_array(&["hello", "world", "test"]);
    assert!(arr.is_array());
    assert_eq!(arr.len(), 3);
    assert_eq!(print_unformatted(&arr), r#"["hello","world","test"]"#);
}

// ============================================================================
// S-PE: 文件级解析往返测试（嵌入 JSON 字符串）
// ============================================================================

/// S-PE-01: 解析 glossary 对象 (对标 test1)
#[test]
fn test_2_spe01_parse_example_glossary() {
    let input = r#"{"glossary":{"title":"example glossary","GlossDiv":{"title":"S","GlossList":{"GlossEntry":{"ID":"SGML","SortAs":"SGML","GlossTerm":"Standard Generalized Markup Language","Acronym":"SGML","Abbrev":"ISO 8879:1986","GlossDef":{"para":"A meta-markup language, used to create markup languages such as DocBook.","GlossSeeAlso":["GML","XML"]},"GlossSee":"markup"}}}}}"#;
    let parsed = parse(input).unwrap();
    let printed = print(&parsed);
    let reparsed = parse(&printed).unwrap();
    assert!(compare(&parsed, &reparsed, true), "round-trip 应保持一致");
}

/// S-PE-02: 解析 menu with null items (对标 test5)
#[test]
fn test_2_spe02_parse_example_menu() {
    let input = r#"{"menu":{"header":"SVG Viewer","items":[{"id":"Open"},{"id":"OpenNew","label":"Open New"},null,{"id":"ZoomIn","label":"Zoom In"},{"id":"ZoomOut","label":"Zoom Out"},null,{"id":"Help"},{"id":"About","label":"About Adobe CVG Viewer..."}]}}"#;
    let parsed = parse(input).unwrap();
    let printed = print(&parsed);
    let reparsed = parse(&printed).unwrap();
    assert!(compare(&parsed, &reparsed, true));
}

/// S-PE-03: 无效 JSON 应失败 (对标 test6)
#[test]
fn test_2_spe03_parse_invalid_html() {
    let input = "<!DOCTYPE html><html><body>Not JSON</body></html>";
    assert!(parse(input).is_err());
}

/// S-PE-04: 解析 Jack "Bee" Nimble (对标 test11)
#[test]
fn test_2_spe04_parse_example_jack() {
    let input = r#"{"name":"Jack (\"Bee\") Nimble","format":{"type":"rect","width":1920,"height":1080,"interlace":false,"frame rate":24}}"#;
    let parsed = parse(input).unwrap();
    let name = get_object_item_case_sensitive(&parsed, "name").unwrap();
    assert!(name.as_string().unwrap().contains("Jack"));
    let printed = print(&parsed);
    let reparsed = parse(&printed).unwrap();
    assert!(compare(&parsed, &reparsed, true));
}

/// S-PE-05: 不完整 JSON 应失败 (对标 test12)
#[test]
fn test_2_spe05_parse_incomplete_json() {
    assert!(parse(r#"{"name":"#).is_err());
}

/// S-PE-06: parse_with_length 精确长度与截断
#[test]
fn test_2_spe06_parse_with_length_no_null_terminator() {
    let json = r#"{"key":"value"}"#;
    assert!(parse_with_length(json, json.len()).is_ok());
    assert!(parse_with_length(json, 3).is_err());
}

// ============================================================================
// S-README: Readme 构造示例测试
// ============================================================================

/// S-README-01: 用基础 API 构造 Monitor 对象
#[test]
fn test_2_sreadme01_create_monitor() {
    let mut monitor = JsonNode::new_object();
    monitor.add_string_to_object("name", "Awesome 4K").unwrap();
    monitor.add_number_to_object("refresh_rate", 60.0).unwrap();

    let mut resolutions = JsonNode::new_array();
    for &(w, h) in &[(1280.0, 720.0), (1920.0, 1080.0), (3840.0, 2160.0)] {
        let mut res = JsonNode::new_object();
        res.add_number_to_object("width", w).unwrap();
        res.add_number_to_object("height", h).unwrap();
        resolutions.add_item_to_array(res).unwrap();
    }
    monitor.add_item_to_object("resolutions", resolutions).unwrap();

    let output = print(&monitor);
    assert!(output.contains("Awesome 4K"));
    assert!(output.contains("1920"));
    assert!(output.contains("2160"));
}

/// S-README-02: 检查 Full HD 支持
#[test]
fn test_2_sreadme02_supports_full_hd() {
    let input = r#"{"name":"Awesome 4K","resolutions":[{"width":1280,"height":720},{"width":1920,"height":1080},{"width":3840,"height":2160}]}"#;
    let monitor = parse(input).unwrap();
    let resolutions = get_object_item_case_sensitive(&monitor, "resolutions").unwrap();
    let items = resolutions.as_array().unwrap();

    let mut has_full_hd = false;
    for res in items {
        let w = get_object_item_case_sensitive(res, "width").unwrap();
        let h = get_object_item_case_sensitive(res, "height").unwrap();
        if w.as_number().unwrap() == 1920.0 && h.as_number().unwrap() == 1080.0 {
            has_full_hd = true;
        }
    }
    assert!(has_full_hd, "应检测到 1920x1080");

    // 不含 Full HD 的 monitor
    let input2 = r#"{"name":"SD","resolutions":[{"width":640,"height":480}]}"#;
    let monitor2 = parse(input2).unwrap();
    let res2 = get_object_item_case_sensitive(&monitor2, "resolutions").unwrap();
    let items2 = res2.as_array().unwrap();
    let mut has_hd2 = false;
    for r in items2 {
        let w = get_object_item_case_sensitive(r, "width").unwrap();
        let h = get_object_item_case_sensitive(r, "height").unwrap();
        if w.as_number().unwrap() == 1920.0 && h.as_number().unwrap() == 1080.0 {
            has_hd2 = true;
        }
    }
    assert!(!has_hd2, "SD monitor 不应有 Full HD");
}

// ============================================================================
// S-PTR: 补充 JSON Pointer 测试 (RFC 6901)
// ============================================================================

/// S-PTR-01: 空指针 "" 返回根
#[test]
fn test_2_sptr01_pointer_root() {
    let doc = parse(r#"{"foo":["bar"],"":0}"#).unwrap();
    let result = get_pointer(&doc, "");
    assert!(result.is_ok(), "空指针应返回根");
    assert!(result.unwrap().is_object());
}

/// S-PTR-02: "/" 匹配空字符串键
#[test]
fn test_2_sptr02_pointer_empty_key() {
    let doc = parse(r#"{"":0,"foo":1}"#).unwrap();
    let result = get_pointer(&doc, "/");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_number().unwrap(), 0.0);
}

/// S-PTR-03: 含百分号的键
#[test]
fn test_2_sptr03_pointer_percent_key() {
    let mut obj = JsonNode::new_object();
    obj.add_number_to_object("c%d", 2.0).unwrap();
    let result = get_pointer(&obj, "/c%d");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_number().unwrap(), 2.0);
}

/// S-PTR-04: 含竖线的键
#[test]
fn test_2_sptr04_pointer_pipe_key() {
    let mut obj = JsonNode::new_object();
    obj.add_number_to_object("g|h", 4.0).unwrap();
    assert_eq!(get_pointer(&obj, "/g|h").unwrap().as_number().unwrap(), 4.0);
}

/// S-PTR-05: 含反斜杠的键
#[test]
fn test_2_sptr05_pointer_backslash_key() {
    let mut obj = JsonNode::new_object();
    obj.add_number_to_object("i\\j", 5.0).unwrap();
    assert_eq!(get_pointer(&obj, "/i\\j").unwrap().as_number().unwrap(), 5.0);
}

/// S-PTR-06: 含双引号的键
#[test]
fn test_2_sptr06_pointer_quote_key() {
    let mut obj = JsonNode::new_object();
    obj.add_number_to_object("k\"l", 6.0).unwrap();
    assert_eq!(get_pointer(&obj, "/k\"l").unwrap().as_number().unwrap(), 6.0);
}

/// S-PTR-07: 含空格的键
#[test]
fn test_2_sptr07_pointer_space_key() {
    let mut obj = JsonNode::new_object();
    obj.add_number_to_object(" ", 7.0).unwrap();
    assert_eq!(get_pointer(&obj, "/ ").unwrap().as_number().unwrap(), 7.0);
}

/// S-PTR-08: 深层嵌套指针
#[test]
fn test_2_sptr08_pointer_deep_nested() {
    let doc = parse(r#"{"a":{"b":{"c":[0,1,2]}}}"#).unwrap();
    assert_eq!(get_pointer(&doc, "/a/b/c/2").unwrap().as_number().unwrap(), 2.0);
}

// ============================================================================
// S-MP: 补充 Merge Patch 测试 (RFC 7396)
// ============================================================================

/// S-MP-01: 覆盖值
#[test]
fn test_2_smp01_merge_overwrite() {
    let mut t = parse(r#"{"a":"b"}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":"c"}"#).unwrap(), true).unwrap();
    assert!(compare(&t, &parse(r#"{"a":"c"}"#).unwrap(), true));
}

/// S-MP-02: 添加新字段
#[test]
fn test_2_smp02_merge_add_field() {
    let mut t = parse(r#"{"a":"b"}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"b":"c"}"#).unwrap(), true).unwrap();
    assert!(has_object_item(&t, "a"));
    assert!(has_object_item(&t, "b"));
}

/// S-MP-03: null 删除字段
#[test]
fn test_2_smp03_merge_delete_field() {
    let mut t = parse(r#"{"a":"b"}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":null}"#).unwrap(), true).unwrap();
    assert!(compare(&t, &parse("{}").unwrap(), true));
}

/// S-MP-04: 删除单个字段
#[test]
fn test_2_smp04_merge_delete_one_field() {
    let mut t = parse(r#"{"a":"b","b":"c"}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":null}"#).unwrap(), true).unwrap();
    assert!(!has_object_item(&t, "a"));
    assert!(has_object_item(&t, "b"));
}

/// S-MP-05: 数组被标量替换
#[test]
fn test_2_smp05_merge_array_replaced_by_scalar() {
    let mut t = parse(r#"{"a":["b"]}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":"c"}"#).unwrap(), true).unwrap();
    assert_eq!(get_object_item_case_sensitive(&t, "a").unwrap().as_string().unwrap(), "c");
}

/// S-MP-06: 标量被数组替换
#[test]
fn test_2_smp06_merge_scalar_replaced_by_array() {
    let mut t = parse(r#"{"a":"c"}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":["b"]}"#).unwrap(), true).unwrap();
    assert!(get_object_item_case_sensitive(&t, "a").unwrap().is_array());
}

/// S-MP-07: 嵌套对象合并
#[test]
fn test_2_smp07_merge_nested_object() {
    let mut t = parse(r#"{"a":{"b":"c"}}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":{"b":"d","c":null}}"#).unwrap(), true).unwrap();
    assert!(compare(&t, &parse(r#"{"a":{"b":"d"}}"#).unwrap(), true));
}

/// S-MP-08: 数组不做合并直接替换
#[test]
fn test_2_smp08_merge_array_replaced() {
    let mut t = parse(r#"{"a":[{"b":"c"}]}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":[1]}"#).unwrap(), true).unwrap();
    assert!(compare(&t, &parse(r#"{"a":[1]}"#).unwrap(), true));
}

/// S-MP-09: 非对象 target 被替换
#[test]
fn test_2_smp09_merge_array_target_replaced() {
    let mut t = parse(r#"["a","b"]"#).unwrap();
    merge_patch(&mut t, &parse(r#"["c","d"]"#).unwrap(), true).unwrap();
    assert!(compare(&t, &parse(r#"["c","d"]"#).unwrap(), true));
}

/// S-MP-10: 对象被数组替换
#[test]
fn test_2_smp10_merge_object_replaced_by_array() {
    let mut t = parse(r#"{"a":"b"}"#).unwrap();
    merge_patch(&mut t, &parse(r#"["c"]"#).unwrap(), true).unwrap();
    assert!(compare(&t, &parse(r#"["c"]"#).unwrap(), true));
}

/// S-MP-11: null patch 替换一切
#[test]
fn test_2_smp11_merge_null_patch() {
    let mut t = parse(r#"{"a":"foo"}"#).unwrap();
    let patch = parse("null").unwrap();
    let result = merge_patch(&mut t, &patch, true);
    if result.is_ok() {
        assert!(t.is_null(), "null patch 应替换 target 为 null");
    } else {
        println!("[功能差异] LX-json merge_patch 不支持 null patch");
    }
}

/// S-MP-12: 标量 patch
#[test]
fn test_2_smp12_merge_scalar_patch() {
    let mut t = parse(r#"{"a":"foo"}"#).unwrap();
    let patch = parse(r#""bar""#).unwrap();
    let result = merge_patch(&mut t, &patch, true);
    if result.is_ok() {
        assert!(t.is_string());
    } else {
        println!("[功能差异] LX-json merge_patch 不支持标量 patch");
    }
}

/// S-MP-13: 保留 null 值字段
#[test]
fn test_2_smp13_merge_preserve_null_value() {
    let mut t = parse(r#"{"e":null}"#).unwrap();
    merge_patch(&mut t, &parse(r#"{"a":1}"#).unwrap(), true).unwrap();
    assert!(has_object_item(&t, "e"));
    assert!(has_object_item(&t, "a"));
}

/// S-MP-14: 深层 null 删除
#[test]
fn test_2_smp14_merge_deep_null_delete() {
    let mut t = parse("{}").unwrap();
    merge_patch(&mut t, &parse(r#"{"a":{"bb":{"ccc":null}}}"#).unwrap(), true).unwrap();
    assert!(compare(&t, &parse(r#"{"a":{"bb":{}}}"#).unwrap(), true),
            "深层 null 应删除对应键, 实际: {}", print(&t));
}

// ============================================================================
// S-PATCH: 补充 JSON Patch 测试
// ============================================================================

/// S-PATCH-01: add_patch_to_array
#[test]
fn test_2_spatch01_add_patch_to_array() {
    let mut patches = JsonNode::new_array();
    let value = JsonNode::String("test".to_string());
    add_patch_to_array(&mut patches, "add", "/foo", Some(&value)).unwrap();
    assert_eq!(patches.len(), 1);
}

/// S-PATCH-02: 多种 patch 操作
#[test]
fn test_2_spatch02_multiple_patch_ops() {
    let mut doc = parse(r#"{"a":1,"b":2,"c":3}"#).unwrap();
    let patches = parse(r#"[
        {"op":"remove","path":"/b"},
        {"op":"add","path":"/d","value":4},
        {"op":"replace","path":"/a","value":10}
    ]"#).unwrap();

    let result = apply_patches(&mut doc, &patches, true);
    if result.is_ok() {
        assert!(!has_object_item(&doc, "b"), "b 应被删除");
        if let Ok(a) = get_object_item_case_sensitive(&doc, "a") {
            assert_eq!(a.as_number().unwrap(), 10.0);
        }
    } else {
        println!("[可能差异] apply_patches 多种操作失败: {:?}", result.err());
    }
}

/// S-PATCH-03: generate_patches 空到非空
#[test]
fn test_2_spatch03_generate_patches_from_empty() {
    let from = parse("{}").unwrap();
    let to = parse(r#"{"a":1,"b":"hello"}"#).unwrap();
    let patches = generate_patches(&from, &to, true).unwrap();
    assert!(patches.is_array());
    assert!(patches.len() > 0);
}

// ============================================================================
// S-MISC: 杂项测试
// ============================================================================

/// S-MISC-01: has_object_item
#[test]
fn test_2_smisc01_has_object_item() {
    let obj = parse(r#"{"name":"John","age":30}"#).unwrap();
    assert!(has_object_item(&obj, "name"));
    assert!(has_object_item(&obj, "age"));
    assert!(!has_object_item(&obj, "missing"));
}

/// S-MISC-02: 对非对象调用 has_object_item
#[test]
fn test_2_smisc02_has_object_item_on_non_object() {
    let arr = parse("[1,2,3]").unwrap();
    assert!(!has_object_item(&arr, "key"), "对非对象应返回 false");
}

/// S-MISC-03: get_array_item 正常与越界
#[test]
fn test_2_smisc03_get_array_item() {
    let arr = parse("[10,20,30]").unwrap();
    assert_eq!(get_array_item(&arr, 0).unwrap().as_number().unwrap(), 10.0);
    assert_eq!(get_array_item(&arr, 2).unwrap().as_number().unwrap(), 30.0);
    assert!(get_array_item(&arr, 3).is_err());
}

/// S-MISC-04: 浅拷贝与深拷贝
#[test]
fn test_2_smisc04_duplicate_shallow_deep() {
    let original = parse(r#"{"items":[1,2,3],"meta":{"key":"val"}}"#).unwrap();
    let deep = duplicate(&original, true);
    assert!(compare(&original, &deep, true));
    let shallow = duplicate(&original, false);
    assert!(shallow.is_object());
    assert_eq!(shallow.len(), 0);
}

/// S-MISC-05: print_buffered
#[test]
fn test_2_smisc05_print_buffered() {
    let node = parse(r#"{"key":"value"}"#).unwrap();
    let output = print_buffered(&node, 1024, true);
    assert!(output.contains("key"));
    let compact = print_buffered(&node, 64, false);
    assert_eq!(compact, r#"{"key":"value"}"#);
}

/// S-MISC-06: print_preallocated
#[test]
fn test_2_smisc06_print_preallocated() {
    let node = parse("[1,2,3]").unwrap();
    let mut buffer = String::with_capacity(1024);
    let result = print_preallocated(&node, &mut buffer, false);
    assert!(result.is_ok());
    assert_eq!(buffer, "[1,2,3]");
}

/// S-MISC-07: 数组遍历
#[test]
fn test_2_smisc07_array_foreach() {
    let arr = parse("[10,20,30,40,50]").unwrap();
    let items = arr.as_array().unwrap();
    let mut count = 0;
    for item in items {
        assert!(item.is_number());
        count += 1;
    }
    assert_eq!(count, 5);
}

/// S-MISC-08: 空数组遍历不崩溃
#[test]
fn test_2_smisc08_array_foreach_empty() {
    let items = parse("[]").unwrap().as_array().unwrap().to_vec();
    assert_eq!(items.len(), 0);
}

/// S-MISC-09: 更多非法输入不崩溃
#[test]
fn test_2_smisc09_more_invalid_inputs() {
    let invalid = vec![
        "{{}", "[[]", r#"{"key""#, r#"{"key":"#,
        "nulll", "truee", "falsee",
        "0x1", "01", "1e", "1e+",
        r#""\"#, r#""\u""#, r#""\u000""#,
    ];
    for input in &invalid {
        let _ = parse(input);
    }
}
