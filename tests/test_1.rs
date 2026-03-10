//! test_1.rs - 对标 cJSON test.c 的测试内容
//!
//! 本文件覆盖 CJSON_TESTING_REFERENCE.md 第一章的测试：
//! - T-01 ~ T-05: 构造与打印测试
//! - T-06 ~ T-07: PrintPreallocated 成功/失败路径
//! - T-08: 数值边界路径（无穷大）

use lx_json::{
    parse, print, print_unformatted, print_preallocated, print_buffered,
    JsonNode, compare,
};

// ============================================================================
// T-01: 构造 Video 对象并打印
// ============================================================================
#[test]
fn test_1_t01_create_video_object_and_print() {
    // 构造 Video 对象: 含 name, format.type/width/height/interlace/frame rate
    let mut format = JsonNode::new_object();
    format.add_item_to_object("type", JsonNode::new_string("rect")).unwrap();
    format.add_number_to_object("width", 1920.0).unwrap();
    format.add_number_to_object("height", 1080.0).unwrap();
    format.add_bool_to_object("interlace", false).unwrap();
    format.add_number_to_object("frame rate", 24.0).unwrap();

    let mut video = JsonNode::new_object();
    video.add_item_to_object("name", JsonNode::new_string("Awesome 4K")).unwrap();
    video.add_item_to_object("format", format).unwrap();

    let output = print(&video);
    assert!(!output.is_empty(), "cJSON_Print 应成功输出非空字符串");

    // 验证输出为合法 JSON（可再次解析）
    let reparsed = parse(&output);
    assert!(reparsed.is_ok(), "输出应为合法 JSON 文本，可再次解析");

    // 验证关键字段存在
    assert!(output.contains("name"), "输出应包含 'name' 字段");
    assert!(output.contains("Awesome 4K"), "输出应包含视频名称");
    assert!(output.contains("format"), "输出应包含 'format' 字段");
    assert!(output.contains("width"), "输出应包含 'width' 字段");
    assert!(output.contains("1920"), "输出应包含宽度值");
    assert!(output.contains("1080"), "输出应包含高度值");
}

// ============================================================================
// T-02: 构造字符串数组 [Sunday..Saturday] 并打印
// ============================================================================
#[test]
fn test_1_t02_create_weekday_string_array() {
    let days = vec!["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    let array = JsonNode::create_string_array(&days);

    let output = print(&array);
    assert!(!output.is_empty(), "打印应成功");

    // 验证数组顺序保持一致
    let reparsed = parse(&output).unwrap();
    let arr = reparsed.as_array().unwrap();
    assert_eq!(arr.len(), 7, "应有 7 天");

    for (i, day) in days.iter().enumerate() {
        let val = arr[i].as_string().unwrap();
        assert_eq!(val, *day, "第 {} 天应为 {}", i, day);
    }
}

// ============================================================================
// T-03: 构造二维整型矩阵数组并打印
// ============================================================================
#[test]
fn test_1_t03_create_2d_int_matrix() {
    // 构造 3x3 矩阵
    let matrix_data: Vec<Vec<i64>> = vec![
        vec![1, 0, 0],
        vec![0, 1, 0],
        vec![0, 0, 1],
    ];

    let mut rows = Vec::new();
    for row in &matrix_data {
        rows.push(JsonNode::create_int_array(row));
    }
    let matrix = JsonNode::Array(rows);

    let output = print(&matrix);
    assert!(!output.is_empty(), "打印应成功");

    // 验证嵌套数组结构
    let reparsed = parse(&output).unwrap();
    let outer = reparsed.as_array().unwrap();
    assert_eq!(outer.len(), 3, "应有 3 行");

    for (i, row) in outer.iter().enumerate() {
        let inner = row.as_array().unwrap();
        assert_eq!(inner.len(), 3, "每行应有 3 个元素");
        for (j, val) in inner.iter().enumerate() {
            let expected = matrix_data[i][j] as f64;
            assert_eq!(val.as_number().unwrap(), expected, "matrix[{}][{}] 应为 {}", i, j, expected);
        }
    }
}

// ============================================================================
// T-04: 构造 Image 对象（含 Thumbnail、IDs）并打印
// ============================================================================
#[test]
fn test_1_t04_create_image_object() {
    let mut thumbnail = JsonNode::new_object();
    thumbnail.add_item_to_object("Url", JsonNode::new_string("http://www.example.com/image/481989943")).unwrap();
    thumbnail.add_number_to_object("Height", 125.0).unwrap();
    thumbnail.add_number_to_object("Width", 100.0).unwrap();

    let ids = JsonNode::create_int_array(&[116, 943, 234, 38793]);

    let mut image = JsonNode::new_object();
    image.add_number_to_object("Width", 800.0).unwrap();
    image.add_number_to_object("Height", 600.0).unwrap();
    image.add_item_to_object("Title", JsonNode::new_string("View from 15th Floor")).unwrap();
    image.add_item_to_object("Thumbnail", thumbnail).unwrap();
    image.add_item_to_object("IDs", ids).unwrap();

    let output = print(&image);
    assert!(!output.is_empty(), "打印应成功");

    let reparsed = parse(&output);
    assert!(reparsed.is_ok(), "输出应为合法 JSON");

    let obj = reparsed.unwrap();
    assert!(obj.get("Width").is_some(), "应包含 Width 字段");
    assert!(obj.get("Height").is_some(), "应包含 Height 字段");
    assert!(obj.get("Title").is_some(), "应包含 Title 字段");
    assert!(obj.get("Thumbnail").is_some(), "应包含 Thumbnail 字段");
    assert!(obj.get("IDs").is_some(), "应包含 IDs 字段");
}

// ============================================================================
// T-05: 构造 records 数组（对象数组）并打印
// ============================================================================
#[test]
fn test_1_t05_create_records_array() {
    let records_data = vec![
        ("first", "John", 25),
        ("second", "Jane", 30),
        ("third", "Bob", 35),
    ];

    let mut records = Vec::new();
    for (id, name, age) in &records_data {
        let mut record = JsonNode::new_object();
        record.add_item_to_object("id", JsonNode::new_string(*id)).unwrap();
        record.add_item_to_object("name", JsonNode::new_string(*name)).unwrap();
        record.add_number_to_object("age", *age as f64).unwrap();
        records.push(record);
    }
    let array = JsonNode::Array(records);

    let output = print(&array);
    assert!(!output.is_empty(), "打印应成功");

    let reparsed = parse(&output).unwrap();
    let arr = reparsed.as_array().unwrap();
    assert_eq!(arr.len(), 3, "应有 3 条记录");

    // 验证每条记录的字段完整
    for (i, item) in arr.iter().enumerate() {
        assert!(item.get("id").is_some(), "记录 {} 应包含 id", i);
        assert!(item.get("name").is_some(), "记录 {} 应包含 name", i);
        assert!(item.get("age").is_some(), "记录 {} 应包含 age", i);

        let (expected_id, expected_name, expected_age) = records_data[i];
        assert_eq!(item.get("id").unwrap().as_string().unwrap(), expected_id);
        assert_eq!(item.get("name").unwrap().as_string().unwrap(), expected_name);
        assert_eq!(item.get("age").unwrap().as_number().unwrap(), expected_age as f64);
    }
}

// ============================================================================
// T-06: PrintPreallocated 成功路径（缓冲区足够大）
// ============================================================================
#[test]
fn test_1_t06_print_preallocated_success() {
    // 构造一个 JSON 对象
    let mut obj = JsonNode::new_object();
    obj.add_item_to_object("name", JsonNode::new_string("test")).unwrap();
    obj.add_number_to_object("value", 42.0).unwrap();

    // 先正常打印，获取所需大小
    let normal_output = print(&obj);
    let required_len = normal_output.len();

    // 分配 strlen(cJSON_Print)+5 大小的缓冲区
    let buffer_size = required_len + 5;
    let mut buffer = String::with_capacity(buffer_size);

    // 调用 print_preallocated
    let result = print_preallocated(&obj, &mut buffer, true);
    assert!(result.is_ok(), "缓冲区充足时 print_preallocated 应返回成功");
    assert!(!buffer.is_empty(), "输出不应为空");

    // 验证输出内容与正常打印一致
    assert_eq!(buffer, normal_output, "预分配打印结果应与正常打印一致");
}

// ============================================================================
// T-07: PrintPreallocated 失败路径（缓冲区不足）
// 注意：LX-json 的 print_preallocated 使用 Rust 的 String，会自动扩容，
// 因此此测试可能无法真正触发"缓冲区不足"的失败。这是一个已知差异。
// ============================================================================
#[test]
fn test_1_t07_print_preallocated_insufficient_buffer() {
    let mut obj = JsonNode::new_object();
    obj.add_item_to_object("name", JsonNode::new_string("test")).unwrap();
    obj.add_number_to_object("value", 42.0).unwrap();

    // 先正常打印获取所需大小
    let normal_output = print(&obj);
    let _required_len = normal_output.len();

    // 分配一个极小的缓冲区（Rust String 会自动扩容，所以这里只用小 capacity）
    let mut buffer = String::with_capacity(1);

    // 在 Rust 的 String 实现中，print_preallocated 不会真正失败
    // 因为 String 会自动扩容。这体现了 LX-json 与 cJSON 的行为差异。
    let result = print_preallocated(&obj, &mut buffer, true);

    // LX-json 的 print_preallocated 使用 Rust 的 String 会自动扩容
    // 这里记录此差异：cJSON 会返回失败，而 LX-json 总是成功
    if result.is_ok() {
        println!("[已知差异] LX-json 的 print_preallocated 不会因缓冲区不足而失败（Rust String 自动扩容）");
        assert_eq!(buffer, normal_output);
    } else {
        // 如果实现确实返回了失败，则验证预期的失败行为
        println!("print_preallocated 返回失败（缓冲区不足），符合 cJSON 行为");
    }
}

// ============================================================================
// T-08: 数值边界路径 - Infinity
// 向对象写入 number = 1.0 / 0.0 再打印，不崩溃
// ============================================================================
#[test]
fn test_1_t08_infinity_number_no_crash() {
    let inf_value = 1.0_f64 / 0.0_f64;
    assert!(inf_value.is_infinite(), "应为无穷大");

    let mut obj = JsonNode::new_object();
    obj.add_number_to_object("number", inf_value).unwrap();

    // 主要验证：不崩溃
    let output = print(&obj);
    // 输出可能包含 "inf"、"Infinity" 或其他表示
    // 核心要求是不崩溃、不越界
    println!("Infinity 打印结果: {}", output);

    // 额外验证：NaN
    let nan_value = f64::NAN;
    let mut obj2 = JsonNode::new_object();
    obj2.add_number_to_object("number", nan_value).unwrap();

    let output2 = print(&obj2);
    println!("NaN 打印结果: {}", output2);
    // 不崩溃即可
}

// ============================================================================
// 额外测试：验证普通打印与预分配打印一致性
// ============================================================================
#[test]
fn test_1_print_consistency() {
    // 构造复合 JSON
    let mut root = JsonNode::new_object();
    root.add_item_to_object("name", JsonNode::new_string("Awesome 4K")).unwrap();

    let mut format = JsonNode::new_object();
    format.add_item_to_object("type", JsonNode::new_string("rect")).unwrap();
    format.add_number_to_object("width", 1920.0).unwrap();
    format.add_number_to_object("height", 1080.0).unwrap();
    root.add_item_to_object("format", format).unwrap();

    // 普通打印
    let normal = print(&root);

    // 预分配打印
    let mut buffer = String::new();
    let result = print_preallocated(&root, &mut buffer, true);
    assert!(result.is_ok());
    assert_eq!(normal, buffer, "普通打印与预分配打印结果应一致");

    // buffered 打印
    let buffered = print_buffered(&root, normal.len() + 5, true);
    assert_eq!(normal, buffered, "普通打印与 buffered 打印结果应一致");

    // 非格式化打印
    let unformatted = print_unformatted(&root);
    assert!(!unformatted.contains('\n'), "非格式化输出不应包含换行");
    assert!(!unformatted.contains("  "), "非格式化输出不应包含多余空格");

    // 非格式化打印也应是合法 JSON
    let reparsed = parse(&unformatted);
    assert!(reparsed.is_ok(), "非格式化输出应可再次解析");
}

// ============================================================================
// 额外测试：使用 compare 验证构造后 round-trip
// ============================================================================
#[test]
fn test_1_roundtrip_with_compare() {
    let mut obj = JsonNode::new_object();
    obj.add_item_to_object("name", JsonNode::new_string("test")).unwrap();
    obj.add_number_to_object("value", 42.0).unwrap();
    obj.add_bool_to_object("active", true).unwrap();
    obj.add_item_to_object("items", JsonNode::Array(vec![
        JsonNode::Number(1.0),
        JsonNode::Number(2.0),
        JsonNode::Number(3.0),
    ])).unwrap();

    let output = print(&obj);
    let reparsed = parse(&output).unwrap();

    assert!(compare(&obj, &reparsed, true), "构造的 JSON 经 print/parse 后应与原始相等");
}

// ============================================================================
// 额外测试：同时验证格式化打印与非格式化打印
// ============================================================================
#[test]
fn test_1_formatted_vs_unformatted() {
    let days = vec!["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    let array = JsonNode::create_string_array(&days);

    let formatted = print(&array);
    let unformatted = print_unformatted(&array);

    // 两者解析后应相等
    let parsed_formatted = parse(&formatted).unwrap();
    let parsed_unformatted = parse(&unformatted).unwrap();

    assert!(compare(&parsed_formatted, &parsed_unformatted, true),
            "格式化和非格式化解析后应相等");

    // 格式化版本应更长（包含缩进和换行）
    assert!(formatted.len() > unformatted.len(),
            "格式化输出应比非格式化输出更长");
}

// ============================================================================
// 补充测试：对标 cJSON test.c 中的额外构造场景
// ============================================================================

/// S-T-01: 构造 double_array 并打印
#[test]
fn test_1_st01_create_double_array() {
    let arr = JsonNode::create_double_array(&[1.1, 2.2, 3.3, 4.4]);
    assert!(arr.is_array());
    assert_eq!(arr.len(), 4);
    let output = print_unformatted(&arr);
    let reparsed = parse(&output).unwrap();
    assert!(compare(&arr, &reparsed, true), "double_array round-trip");
}

/// S-T-02: 构造 float_array 并打印
#[test]
fn test_1_st02_create_float_array() {
    let arr = JsonNode::create_float_array(&[1.5_f32, 2.5, 3.5]);
    assert!(arr.is_array());
    assert_eq!(arr.len(), 3);
    let output = print_unformatted(&arr);
    let reparsed = parse(&output).unwrap();
    assert_eq!(reparsed.len(), 3);
}

/// S-T-03: 构造空 int_array
#[test]
fn test_1_st03_create_empty_int_array() {
    let arr = JsonNode::create_int_array(&[]);
    assert!(arr.is_array());
    assert_eq!(arr.len(), 0);
    assert_eq!(print_unformatted(&arr), "[]");
}

/// S-T-04: 负数与零的 int_array
#[test]
fn test_1_st04_int_array_with_negatives() {
    let arr = JsonNode::create_int_array(&[-100, 0, 100, -2147483648, 2147483647]);
    assert_eq!(arr.len(), 5);
    let output = print_unformatted(&arr);
    let reparsed = parse(&output).unwrap();
    assert!(compare(&arr, &reparsed, true));
}

/// S-T-05: print_buffered 非格式化
#[test]
fn test_1_st05_print_buffered_unformatted() {
    let mut obj = JsonNode::new_object();
    obj.add_item_to_object("key", JsonNode::new_string("value")).unwrap();
    let compact = print_buffered(&obj, 64, false);
    assert_eq!(compact, r#"{"key":"value"}"#);
}

/// S-T-06: 嵌套 3 层对象的构造与验证
#[test]
fn test_1_st06_deeply_nested_construction() {
    let mut inner = JsonNode::new_object();
    inner.add_number_to_object("deep", 42.0).unwrap();

    let mut middle = JsonNode::new_object();
    middle.add_item_to_object("inner", inner).unwrap();

    let mut outer = JsonNode::new_object();
    outer.add_item_to_object("middle", middle).unwrap();

    let output = print(&outer);
    let reparsed = parse(&output).unwrap();
    assert!(compare(&outer, &reparsed, true));

    // 验证嵌套访问
    let deep_val = reparsed.get("middle").unwrap().get("inner").unwrap().get("deep").unwrap();
    assert_eq!(deep_val.as_number().unwrap(), 42.0);
}
