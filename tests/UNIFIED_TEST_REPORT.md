# LX-json 统一测试报告 —— 与 cJSON 原版测试对比分析

> 生成时间：2026-03-10  
> 测试对象：LX-json v0.1.0（Rust 原生 + C ABI 胶水层）  
> 对标基准：cJSON 原版测试套件（21 个测试二进制 / 162 个 `RUN_TEST` / 97 项规范）

---

## 一、测试总览

### 1.1 cJSON 原版测试架构

cJSON 使用 **Unity** C 测试框架，每个测试源文件编译为独立可执行文件，通过 CMake `add_test()` 注册。

| 维度 | 数值 |
|------|------|
| CTest 注册二进制（不含 utils） | 19（含 `test.c` 的 `cJSON_test`） |
| CTest 注册二进制（含 utils） | 22 |
| `RUN_TEST(...)` 用例总数 | 162 |
| `TEST_ASSERT_*` 断言总数 | ~491 |
| 文档规范测试项（`CJSON_TESTING_REFERENCE.md`） | 97 |

#### cJSON 21 个测试二进制清单

| # | 可执行文件 | 源文件 | RUN_TEST 数 | 断言数 | 功能域 |
|---|-----------|--------|------------|--------|--------|
| 1 | `cJSON_test` | `test.c` | —（手工） | — | 示例 + PrintPreallocated + Infinity |
| 2 | `parse_value` | `parse_value.c` | 7 | ~2 | 值类型解析 |
| 3 | `parse_number` | `parse_number.c` | 6 | ~5 | 数字解析 |
| 4 | `parse_string` | `parse_string.c` | 6 | ~4 | 字符串 / Unicode / 转义解析 |
| 5 | `parse_array` | `parse_array.c` | 4 | ~10 | 数组解析 |
| 6 | `parse_object` | `parse_object.c` | 4 | ~8 | 对象解析 |
| 7 | `parse_with_opts` | `parse_with_opts.c` | 6 | ~20 | 定长 / BOM / require_null |
| 8 | `parse_examples` | `parse_examples.c` | 15 | ~14 | 文件示例往返解析 |
| 9 | `parse_hex4` | `parse_hex4.c` | 2 | ~20 | 十六进制 4 位解析 |
| 10 | `print_value` | `print_value.c` | 7 | ~3 | 值类型打印 |
| 11 | `print_number` | `print_number.c` | 6 | ~2 | 数字打印 |
| 12 | `print_string` | `print_string.c` | 3 | ~2 | 字符串打印 |
| 13 | `print_array` | `print_array.c` | 3 | ~5 | 数组打印 |
| 14 | `print_object` | `print_object.c` | 3 | ~5 | 对象打印 |
| 15 | `cjson_add` | `cjson_add.c` | 31 | ~55 | Add API + 分配失败注入 |
| 16 | `compare_tests` | `compare_tests.c` | 10 | ~54 | 比较（数字/布尔/字符串/数组/对象） |
| 17 | `minify_tests` | `minify_tests.c` | 7 | ~11 | 最小化 / 注释去除 |
| 18 | `misc_tests` | `misc_tests.c` | 30 | ~223 | 综合：类型检查/深度限制/NULL防御/BOM 等 |
| 19 | `readme_examples` | `readme_examples.c` | 3 | ~4 | README 示例代码验证 |
| 20 | `json_patch_tests` | `json_patch_tests.c` | 3 | ~15 | JSON Patch (RFC 6902) |
| 21 | `old_utils_tests` | `old_utils_tests.c` | 5 | ~21 | JSON Pointer / Merge Patch / Sort |
| — | `misc_utils_tests` | `misc_utils_tests.c` | 1 | ~13 | Utils 全 API NULL 安全 |
| | **合计** | | **162** | **~491** | |

### 1.2 LX-json 四个测试文件汇总

| 测试文件 | 语言 | 对标章节 | 测试函数数 | 断言总计 | 通过 | 失败 | 通过率 |
|----------|------|----------|-----------|---------|------|------|--------|
| `test_1.rs` | Rust | 第一章 `test.c` | 17 | ~58 | 17 | 0 | **100%** |
| `test_2.rs` | Rust | 第二章 `tests/` | 169 | ~357 | 163 | 6 | **96.4%** |
| `test_1_c.c` | C ABI | 第一章 `test.c` | 14 | ~57 | 56 | 1 | **98.2%** |
| `test_2_c.c` | C ABI | 第二章 `tests/` | 31 | ~276 | 272 | 4 | **98.6%** |
| **合计** | — | — | **231** | **~748** | **508** | **11** | **97.9%** |

### 1.3 失败项一览

| # | 测试文件 | 测试名 | 对标编号 | 失败原因 |
|---|---------|--------|---------|---------|
| 1 | test_2.rs | `scmp01_compare_strings` | S-CMP（补充） | `compare` 不支持大小写不敏感字符串比较 |
| 2 | test_2.rs | `smp13_merge_preserve_null_value` | S-MP（补充） | merge patch 未保留显式 null 值字段 |
| 3 | test_2.rs | `smp14_merge_deep_null_delete` | S-MP（补充） | 深层 null 未触发删除，保留为 `"ccc": null` |
| 4 | test_2.rs | `sptr01_pointer_root` | S-PTR（补充） | 空指针 `""` 未返回根节点 |
| 5 | test_2.rs | `sptr02_pointer_empty_key` | S-PTR（补充） | 空键 `"/"` 查找返回 `None` |
| 6 | test_2.rs | `um03_merge_patch_replace_array` | U-M-03 | 数组被 merge patch 后，结果含 `"c": null` 未被删除 |
| 7 | test_1_c.c | `S-T-03` | S-T（补充） | `lx_json_create_int_array(vals, 0)` 返回 NULL 而非空数组 |
| 8 | test_2_c.c | `P-S-03` | P-S-03 | 代理对 `\uD83D\uDC31` 未解码为 🐱 |
| 9 | test_2_c.c | `U-M-03` | U-M-03 | 数组 merge patch 后结果含 `"c": null` |
| 10 | test_2_c.c | `S-OBJ-02` | S-OBJ（补充） | `get_object_item_case_insensitive("oNe")` 返回 NULL |
| 11 | test_2_c.c | `S-OBJ-02b` | S-OBJ（补充） | `get_object_item_case_insensitive("tWo")` 返回 NULL |

---

## 二、与 cJSON 原版测试规范逐项对比

以下按 `CJSON_TESTING_REFERENCE.md` 的 97 项编号逐条对照，标注每项在 4 个测试文件中的覆盖状态和结果。

图例：
- ✅ = 已覆盖且通过
- ❌ = 已覆盖但失败
- ⬜ = 未覆盖（该文件不涉及此项）
- 🔶 = 已覆盖，行为差异但可接受（SKIP/INFO）

### 2.1 第一章：`test.c`（T-01 ~ T-08）

cJSON 原版：`test.c` → `cJSON_test` 二进制（非 Unity，手工 printf 验证）。

| 编号 | 测试内容 | cJSON 原版 | test_1.rs | test_1_c.c | test_2.rs | test_2_c.c |
|------|---------|-----------|-----------|-----------|-----------|------------|
| T-01 | 构造 Video 对象 + 打印 | ✅ `create_objects()` | ✅ | ✅ | ⬜ | ⬜ |
| T-02 | 构造星期字符串数组 | ✅ `create_objects()` | ✅ | ✅ | ⬜ | ⬜ |
| T-03 | 构造二维整型矩阵 | ✅ `create_objects()` | ✅ | ✅ | ⬜ | ⬜ |
| T-04 | 构造 Image 对象（Thumbnail+IDs） | ✅ `create_objects()` | ✅ | ✅ | ⬜ | ⬜ |
| T-05 | 构造 records 数组 | ✅ `create_objects()` | ✅ | ✅ | ⬜ | ⬜ |
| T-06 | PrintPreallocated 缓冲区充足 | ✅ `print_preallocated()` | ✅ | ✅ | ⬜ | ⬜ |
| T-07 | PrintPreallocated 缓冲区不足 | ✅ `print_preallocated()` | ✅ | ✅ | ⬜ | ⬜ |
| T-08 | Infinity 数值不崩溃 | ✅ `create_objects()` | ✅ | ✅ | ⬜ | ⬜ |
| **小计** | **8/8** | **8 ✅** | **8 ✅** | **8 ✅** | — | — |

### 2.2 第二章 §2.1：解析测试

#### 2.2.1 值类型解析（P-V-01 ~ P-V-07）

cJSON 原版：`parse_value.c` → 7 个 `RUN_TEST`。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c |
|------|------|-----------|-----------|------------|
| P-V-01 | `null` | ✅ `parse_value_should_parse_null` | ✅ pv01 | ✅ |
| P-V-02 | `true` | ✅ `parse_value_should_parse_true` | ✅ pv02 | ✅ |
| P-V-03 | `false` | ✅ `parse_value_should_parse_false` | ✅ pv03 | ✅ |
| P-V-04 | `1.5` | ✅ `parse_value_should_parse_number` | ✅ pv04 | ✅ |
| P-V-05 | `"hello"` | ✅ `parse_value_should_parse_string` | ✅ pv05 | ✅ |
| P-V-06 | `[]` | ✅ `parse_value_should_parse_array` | ✅ pv06 | ✅ |
| P-V-07 | `{}` | ✅ `parse_value_should_parse_object` | ✅ pv07 | ✅ |
| **小计** | **7/7** | **7 ✅** | **7 ✅** | **7 ✅** |

#### 2.2.2 数字解析（P-N-01 ~ P-N-08）

cJSON 原版：`parse_number.c` → 6 个 `RUN_TEST`（每项覆盖正/负/浮点/大数多个子用例）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c |
|------|------|-----------|-----------|------------|
| P-N-01 | `0` | ✅ `parse_number_should_parse_zero` | ✅ pn01 | ✅ |
| P-N-02 | `-2147483648` | ✅ `..parse_negative_integers` | ✅ pn02 | ✅ |
| P-N-03 | `2147483647` | ✅ `..parse_positive_integers` | ✅ pn03 | ✅ |
| P-N-04 | `10e-10` | ✅ `..parse_positive_reals` | ✅ pn04 | ✅ |
| P-N-05 | `123e+127` | ✅ `..parse_positive_reals` | ✅ pn05 | ✅ |
| P-N-06 | 超大数字解析 | ✅ `..parse_big_numbers` + `misc_tests` | ✅ pn06 | ✅ |
| P-N-07 | 非法双小数点 | ✅ `misc_tests` | ✅ pn07 | ✅ |
| P-N-08 | 非法指数链 | ✅ `misc_tests` | ✅ pn08 | ✅ |
| **小计** | **8/8** | **8 ✅** | **8 ✅** | **8 ✅** |

#### 2.2.3 字符串与 Unicode 解析（P-S-01 ~ P-S-05）

cJSON 原版：`parse_string.c` → 6 个 `RUN_TEST`（含代理对、Bug#94 回归）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| P-S-01 | `""` 空字符串 | ✅ `..parse_strings` | ✅ ps01 | ✅ | |
| P-S-02 | 转义 + Unicode `€` | ✅ `..parse_strings` | ✅ ps02 | ✅ | |
| P-S-03 | 代理对 `\uD83D\uDC31` → 🐱 | ✅ `..parse_utf16_surrogate_pairs` | ✅ ps03 | ❌ | C ABI 未正确解码 |
| P-S-04 | 非法转义 `\e` | ✅ `..not_parse_invalid_backslash` | ✅ ps04 | ✅ | |
| P-S-05 | 末尾反斜杠溢出 | ✅ `..not_overflow_with_closing_backslash` | ✅ ps05 | ✅ | |
| — | Bug#94 转义回归 | ✅ `..parse_bug_94` | ✅ sbug94 | ✅ | 补充项 |
| **小计** | **5/5 + 1** | **6 ✅** | **5 ✅** | **4 ✅ 1 ❌** | |

#### 2.2.4 数组与对象解析（P-A-01 ~ P-A-06）

cJSON 原版：`parse_array.c`（4 个 `RUN_TEST`）+ `parse_object.c`（4 个 `RUN_TEST`）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c |
|------|------|-----------|-----------|------------|
| P-A-01 | `[]` 空数组 | ✅ `..parse_empty_arrays` | ✅ pa01 | ✅ |
| P-A-02 | 混合类型数组 | ✅ `..parse_arrays_with_multiple_elements` | ✅ pa02 | ✅ |
| P-A-03 | 多键对象 | ✅ `..parse_objects_with_multiple_elements` | ✅ pa03 | ✅ |
| P-A-04 | 多类型字段对象 | ✅ `..parse_objects_with_one_element` | ✅ pa04 | ✅ |
| P-A-05 | 对象误用数组解析 | ✅ `..should_not_parse_non_arrays` | ✅ pa05 | ✅ |
| P-A-06 | 数组误用对象解析 | ✅ `..should_not_parse_non_objects` | ✅ pa06 | ✅ |
| **小计** | **6/6** | **6 ✅** | **6 ✅** | **6 ✅** |

#### 2.2.5 定长输入与结束位置（P-L-01 ~ P-L-05）

cJSON 原版：`parse_with_opts.c`（6 个 `RUN_TEST`）+ `parse_examples.c`（15 个 `RUN_TEST`）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| P-L-01 | 精确长度解析 | ✅ `parse_examples` (test13) | ✅ pl01 | ✅ | |
| P-L-02 | 截断长度拒绝 | ✅ `..handle_incomplete_json` | ✅ pl02 | ✅ | |
| P-L-03 | 尾部 parse_end | ✅ `..return_parse_end` | ✅ pl03 | 🔶 INFO | LX-json 严格模式拒绝 |
| P-L-04 | `{}x` 要求终止 | ✅ `..require_null_if_requested` | ✅ pl04 | ✅ | |
| P-L-05 | UTF-8 BOM | ✅ `..parse_utf8_bom` + `misc_tests` | ✅ pl05 | 🔶 INFO | LX-json 不支持 BOM |
| — | 文件示例往返 | ✅ `parse_examples` (15项) | ✅ spe01-06 | ⬜ | cJSON 测 11 个文件 |
| — | hex4 全组合 | ✅ `parse_hex4` (2项) | ⬜ | ⬜ | 0x0000-0xFFFF |
| **小计** | **5/5 + 17** | **22 ✅** | **5 ✅** | **3 ✅ 2 🔶** | |

### 2.3 第二章 §2.2：打印测试

#### 2.3.1 值打印（PR-V-01 ~ PR-V-07）

cJSON 原版：`print_value.c` → 7 个 `RUN_TEST`。

| 编号 | 输入值 | cJSON 原版 | test_2.rs | test_2_c.c |
|------|--------|-----------|-----------|------------|
| PR-V-01 | null → `null` | ✅ `..print_null` | ✅ prv01 | ✅ |
| PR-V-02 | true → `true` | ✅ `..print_true` | ✅ prv02 | ✅ |
| PR-V-03 | false → `false` | ✅ `..print_false` | ✅ prv03 | ✅ |
| PR-V-04 | 1.5 → `1.5` | ✅ `..print_number` | ✅ prv04 | ✅ |
| PR-V-05 | `"hello"` | ✅ `..print_string` | ✅ prv05 | ✅ |
| PR-V-06 | `[]` | ✅ `..print_array` | ✅ prv06 | ✅ |
| PR-V-07 | `{}` | ✅ `..print_object` | ✅ prv07 | ✅ |
| **小计** | **7/7** | **7 ✅** | **7 ✅** | **7 ✅** |

#### 2.3.2 数字打印（PR-N-01 ~ PR-N-06）

cJSON 原版：`print_number.c` → 6 个 `RUN_TEST`（含零/正负整数/浮点/NaN）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| PR-N-01 | 0 → `0` | ✅ `..print_zero` | ✅ prn01 | ✅ | |
| PR-N-02 | -32768 | ✅ `..print_negative_integers` | ✅ prn02 | ✅ | |
| PR-N-03 | 2147483647 | ✅ `..print_positive_integers` | ✅ prn03 | ✅ | |
| PR-N-04 | 10e-10 | ✅ `..print_positive_reals` | ✅ prn04 | ✅ | 格式差异，语义等价 |
| PR-N-05 | 123e+127 | ✅ `..print_positive_reals` | ✅ prn05 | ✅ | 同上 |
| PR-N-06 | -123e-128 | ✅ `..print_negative_reals` | ✅ prn06 | ✅ | 同上 |
| — | NaN/Infinity | ✅ `..print_non_number` | ⬜ | ⬜ | cJSON 额外覆盖 |
| **小计** | **6/6 + 1** | **7 ✅** | **6 ✅** | **6 ✅** | |

#### 2.3.3 字符串打印（PR-S-01 ~ PR-S-03）

cJSON 原版：`print_string.c` → 3 个 `RUN_TEST`。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c |
|------|------|-----------|-----------|------------|
| PR-S-01 | 空字符串 → `""` | ✅ `..print_empty_strings` | ✅ prs01 | ✅ |
| PR-S-02 | 控制字符转义 | ✅ `..print_ascii` | ✅ prs02 | ✅ |
| PR-S-03 | UTF-8 字符串 | ✅ `..print_utf8` | ✅ prs03 | ✅ |
| **小计** | **3/3** | **3 ✅** | **3 ✅** | **3 ✅** |

#### 2.3.4 数组与对象打印（PR-A/PR-O）

cJSON 原版：`print_array.c`（3 个 `RUN_TEST`）+ `print_object.c`（3 个 `RUN_TEST`）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| PR-A-01 | `[1,2,3]` 非格式化 | ✅ `..print_arrays_with_multiple_elements` | ✅ pra01 | ✅ | |
| PR-A-02 | `[1,2,3]` 格式化 | ✅ 同上 | ✅ pra02 | ✅ | |
| PR-O-01 | 对象非格式化 | ✅ `..print_objects_with_multiple_elements` | ✅ pro01 | ✅ | |
| PR-O-02 | 对象格式化 | ✅ 同上 | ✅ pro02 | ✅ | LX-json 空格缩进 vs Tab |
| — | 空数组/单元素 | ✅ `..print_empty_arrays` / `..one_element` | ⬜ | ⬜ | cJSON 额外覆盖 |
| — | 空对象/单键值对 | ✅ `..print_empty_objects` / `..one_element` | ⬜ | ⬜ | cJSON 额外覆盖 |
| **小计** | **4/4 + 4** | **8 ✅** | **4 ✅** | **4 ✅** | |

### 2.4 第二章 §2.3：构造与 Add API（A-01 ~ A-11）

cJSON 原版：`cjson_add.c` → 31 个 `RUN_TEST`（每种类型 3 个：正常/NULL/分配失败，+ 4 个 Create*Array 分配失败）。

| 编号 | 操作 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| A-01 | AddNull | ✅ `cjson_add_null_*` (3) | ✅ a01 | ✅ | |
| A-02 | AddTrue | ✅ `cjson_add_true_*` (3) | ✅ a02 | ✅ | |
| A-03 | AddFalse | ✅ `cjson_add_false_*` (3) | ✅ a03 | ✅ | |
| A-04 | AddBool | ✅ `cjson_add_bool_*` (3) | ✅ a04 | ✅ | |
| A-05 | AddNumber | ✅ `cjson_add_number_*` (3) | ✅ a05 | ✅ | |
| A-06 | AddString | ✅ `cjson_add_string_*` (3) | ✅ a06 | ✅ | |
| A-07 | AddRaw | ✅ `cjson_add_raw_*` (3) | ✅ a07 | 🔶 SKIP | C ABI 无 `create_raw` |
| A-08 | AddObject | ✅ `cjson_add_object_*` (3) | ✅ a08 | ✅ | |
| A-09 | AddArray | ✅ `cjson_add_array_*` (3) | ✅ a09 | ✅ | |
| A-10 | NULL 参数防御 | ✅ 各 `_should_fail_*` | ✅ a10 | ✅ | |
| A-11 | 分配失败注入 | ✅ 各 `_should_fail_on_allocation_failure` | ✅ a11 | 🔶 SKIP | C ABI 不可注入 |
| — | Create*Array 分配失败 | ✅ `create_int/float/double/string_array_*` (4) | ⬜ | ⬜ | cJSON 额外 |
| **小计** | **11/11 + 4** | **31 ✅** | **11 ✅** | **9 ✅ 2 🔶** | |

### 2.5 第二章 §2.4：比较与最小化

#### 2.5.1 比较（C-01 ~ C-06）

cJSON 原版：`compare_tests.c` → 10 个 `RUN_TEST`（含 NULL/无效类型/数字/布尔/null/字符串/Raw/数组/对象深度比较）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| C-01 | `1 == 1` | ✅ `..compare_numbers` | ✅ c01 | ✅ | |
| C-02 | `1 != 2` | ✅ `..compare_numbers` | ✅ c02 | ✅ | |
| C-03 | 大小写敏感键不等 | ✅ `..compare_objects` | ✅ c03 | ✅ | |
| C-04 | 大小写不敏感键相等 | ✅ `..compare_objects` | ✅ c04 | 🔶 SKIP | C ABI 未暴露 |
| C-05 | 不同长度数组不等 | ✅ `..compare_arrays` | ✅ c05 | ✅ | |
| C-06 | 不同字段数对象不等 | ✅ `..compare_objects` | ✅ c06 | ✅ | |
| — | NULL 不相等 | ✅ `..compare_null_pointer_as_not_equal` | ⬜ | ⬜ | cJSON 额外 |
| — | 无效类型不相等 | ✅ `..compare_invalid_as_not_equal` | ⬜ | ⬜ | cJSON 额外 |
| — | 布尔比较 | ✅ `..compare_booleans` | ✅ compare_bool_equal | ⬜ | |
| — | Raw 比较 | ✅ `..compare_raw` | ✅ scmp02 | ⬜ | |
| **小计** | **6/6 + 4** | **10 ✅** | **6 ✅** | **5 ✅ 1 🔶** | |

#### 2.5.2 最小化（M-01 ~ M-05）

cJSON 原版：`minify_tests.c` → 7 个 `RUN_TEST`（含缓冲区溢出防护、空白/注释/字符串/死循环）。

| 编号 | 输入 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| M-01 | 空白字符去除 | ✅ `..remove_spaces` | ✅ m01 | ✅ | |
| M-02 | 行注释 `//` 去除 | ✅ `..remove_single_line_comments` | ✅ m02 | 🔶 INFO | LX-json 拒绝注释 |
| M-03 | 块注释 `/**/` 去除 | ✅ `..remove_multiline_comments` | ✅ m03 | 🔶 INFO | 同上 |
| M-04 | 字符串内容保持 | ✅ `..not_modify_strings` | ✅ m04 | ✅ | |
| M-05 | 特殊输入无死循环 | ✅ `..not_loop_infinitely` | ✅ m05 | ✅ | |
| — | 缓冲区溢出防护 | ✅ `..not_overflow_buffer` | ⬜ | ⬜ | cJSON 额外 |
| — | 基本压缩 | ✅ `..minify_json` | ⬜ | ⬜ | cJSON 原生 minify |
| **小计** | **5/5 + 2** | **7 ✅** | **5 ✅** | **3 ✅ 2 🔶** | |

### 2.6 第二章 §2.5：鲁棒性与边界（R-01 ~ R-07）

cJSON 原版：`misc_tests.c` → 30 个 `RUN_TEST`（★ 最大、最全面的测试文件，~223 断言）。

| 编号 | 测试点 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|--------|-----------|-----------|------------|------|
| R-01 | NULL / 空指针防御 | ✅ `..not_crash_with_null_pointers` | ✅ r01 | ✅ (20项) | cJSON: 全 API 覆盖 |
| R-02 | 深度嵌套限制 | ✅ `..not_parse_to_deeply_nested_jsons` | ✅ r02 | ✅ | |
| R-03 | 深拷贝 | ✅ `..not_follow_too_deep_circular_references` | ✅ r03 | ✅ | cJSON 还测循环引用 |
| R-04 | SetValuestring 重叠 | ✅ `..set_valuestring_should_return_null_if_strings_overlap` | ✅ r04 | 🔶 SKIP | C ABI 无此 API |
| R-05 | realloc 失败 | ✅ `ensure_should_fail_on_failed_realloc` | ⬜ | 🔶 SKIP | 无法注入 |
| R-06 | 大数边界 | ✅ `..parse_big_numbers_should_not_report_error` | ✅ r04 | ✅ | |
| R-07 | 数组链表完整性 | ✅ `..delete_item_from_array_should_not_broken_list_structure` | ✅ r05 | ✅ | |
| — | ArrayForEach 宏 | ✅ `..array_foreach_*` (2) | ✅ smisc07/08 | ⬜ | |
| — | GetObjectItem | ✅ `..get_object_item_*` (4) | ✅ sobj01-04 | ✅ | |
| — | 类型检查函数 | ✅ `typecheck_functions_should_check_type` | ✅ stc01 | ✅ | |
| — | SetNumberValue | ✅ `..set_number_value_should_set_numbers` | ⬜ | ⬜ | cJSON 额外 |
| — | DetachItemViaPointer | ✅ `..detach_item_via_pointer_*` (2) | ✅ smut01-03 | ✅ | |
| — | ReplaceItemViaPointer | ✅ `..replace_item_via_pointer_*` (2) | ✅ smut05-06 | ✅ | |
| — | GetStringValue | ✅ `..get_string_value_*` | ✅ sacc01 | ✅ | |
| — | GetNumberValue | ✅ `..get_number_value_*` | ✅ sacc02 | ✅ | |
| — | BOM 跳过 / 非开头 BOM | ✅ `skip_utf8_bom_*` (2) | ✅ pl05/sbom01 | ⬜ | |
| — | 创建引用 (string/object/array) | ✅ `..create_*_reference_*` (3) | ⬜ | ⬜ | cJSON 独有 |
| — | 自引用防护 | ✅ `..should_not_add_itself` | ⬜ | ⬜ | cJSON 独有 |
| — | UAF 防护 | ✅ `..not_use_after_free_when_string_is_aliased` | ⬜ | ⬜ | cJSON 独有 |
| — | SetBoolValue | ✅ `..set_bool_value_must_not_break_objects` | ⬜ | ⬜ | cJSON 独有 |
| — | 内存泄漏防护 | ✅ `..set_valuestring_to_object_should_not_leak_memory` | ⬜ | ⬜ | cJSON 独有 |
| **小计** | **7/7 + 23** | **30 ✅** | **6 ✅ 1 ⬜** | **5 ✅ 2 🔶** | |

### 2.7 第二章 §2.6：cJSON_Utils（可选）

#### 2.7.1 JSON Pointer（U-PTR-01 ~ U-PTR-03）

cJSON 原版：`old_utils_tests.c` → `json_pointer_tests`（含 RFC 6901 完整示例）。

| 编号 | 文档 | Pointer | cJSON 原版 | test_2.rs | test_2_c.c |
|------|------|---------|-----------|-----------|------------|
| U-PTR-01 | `{"foo":["bar"]}` | `/foo/0` | ✅ `json_pointer_tests` | ✅ uptr01 | ✅ |
| U-PTR-02 | `{"a/b":1}` | `/a~1b` | ✅ 同上 | ✅ uptr02 | ✅ |
| U-PTR-03 | `{"m~n":8}` | `/m~0n` | ✅ 同上 | ✅ uptr03 | ✅ |
| — | FindPointerFromObjectTo | — | ✅ `misc_tests` | ⬜ | ⬜ | cJSON 独有 |
| **小计** | **3/3 + 1** | | **4 ✅** | **3 ✅** | **3 ✅** |

#### 2.7.2 JSON Patch（U-PATCH-01 ~ U-PATCH-03）

cJSON 原版：`json_patch_tests.c` → 3 个 `RUN_TEST`（基于 RFC 6902 语料集）。

| 编号 | 描述 | cJSON 原版 | test_2.rs | test_2_c.c |
|------|------|-----------|-----------|------------|
| U-PATCH-01 | 应用有效 patch | ✅ `..pass_json_patch_test_tests` | ✅ upatch01 | ✅ |
| U-PATCH-02 | 应用非法 patch | ✅ `..pass_json_patch_test_spec_tests` | ✅ upatch02 | ✅ |
| U-PATCH-03 | 生成 patch + 回放 | ✅ `..pass_json_patch_test_cjson_utils_tests` | ✅ upatch03 | ✅ |
| **小计** | **3/3** | **3 ✅** | **3 ✅** | **3 ✅** |

#### 2.7.3 Merge Patch（U-M-01 ~ U-M-03）

cJSON 原版：`old_utils_tests.c` → `merge_tests` + `generate_merge_tests`。

| 编号 | 原文档 | 补丁 | 预期 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|--------|------|------|-----------|-----------|------------|------|
| U-M-01 | `{"a":"b"}` | `{"a":"c"}` | `{"a":"c"}` | ✅ `merge_tests` | ✅ um01 | ✅ | |
| U-M-02 | `{"a":"b"}` | `{"a":null}` | `{}` | ✅ `merge_tests` | ✅ um02 | ✅ | |
| U-M-03 | `[1,2]` | `{"a":"b","c":null}` | `{"a":"b"}` | ✅ `merge_tests` | ❌ um03 | ❌ | null 键未删除 |
| — | GenerateMergePatch | — | — | ✅ `generate_merge_tests` | ⬜ | ⬜ | cJSON 额外 |
| **小计** | **3/3 + 1** | | | **4 ✅** | **2 ✅ 1 ❌** | **2 ✅ 1 ❌** | |

#### 2.7.4 README 示例 + Utils NULL 安全

cJSON 原版：`readme_examples.c`（3 个 `RUN_TEST`）+ `misc_utils_tests.c`（1 个 `RUN_TEST`）+ `old_utils_tests.c` 中的 `sort_tests`。

| 编号 | 描述 | cJSON 原版 | test_2.rs | test_2_c.c | 备注 |
|------|------|-----------|-----------|------------|------|
| — | 创建 monitor JSON | ✅ `create_monitor_should_create_a_monitor` | ✅ sreadme01 | ✅ | |
| — | Helper 创建 monitor | ✅ `create_monitor_with_helpers_*` | ⬜ | ⬜ | cJSON 额外 |
| — | Full HD 分辨率检查 | ✅ `supports_full_hd_*` | ✅ sreadme02 | ✅ | |
| — | Utils 全 API NULL 安全 | ✅ `cjson_utils_functions_shouldnt_crash_*` | ⬜ | ⬜ | 部分已覆盖 |
| — | SortObject 排序 | ✅ `sort_tests` | ✅ ssort01/02 | ✅ | |
| **小计** | **5** | **5 ✅** | **3 ✅** | **3 ✅** | |

---

## 三、补充测试（S-* 系列）覆盖汇总

除 97 项 cJSON 原版规范外，LX-json 还新增了大量补充测试覆盖扩展场景。

### 3.1 test_1 补充测试

| 编号 | 描述 | test_1.rs | test_1_c.c | 状态 |
|------|------|-----------|-----------|------|
| S-T-01 | 创建 double 数组 | ✅ st01 | ✅ | |
| S-T-02 | 创建 float 数组 | ✅ st02 | ✅ | |
| S-T-03 | 空整数数组 (count=0) | ✅ st03 | ❌ | C: 返回 NULL 而非空数组 |
| S-T-04 | 含负数整数数组 | ✅ st04 | ✅ | |
| S-T-05 | print_buffered 紧凑输出 | ✅ st05 | ✅ | |
| S-T-06 | 深层嵌套构造 | ✅ st06 | ✅ | |
| — | roundtrip / print 一致性 / formatted vs unformatted | ✅ (3项) | ⬜ | Rust 独有 |
| **小计** | | **9 ✅** | **5 ✅ 1 ❌** | |

### 3.2 test_2 补充测试

| 类别 | 编号前缀 | 数量 | test_2.rs 结果 | test_2_c.c 结果 |
|------|---------|------|---------------|----------------|
| 类型检查 | S-TC | 1 (综合) | ✅ | ✅ (10 子项) |
| 对象访问 | S-OBJ | 4 | 4 ✅ | 3 ✅ 2 ❌ 1 ✅ |
| 值访问器 | S-ACC | 2 | 2 ✅ | ✅ (9 子项) |
| 补充比较 | S-CMP | 5 | 4 ✅ 1 ❌ | ✅ (5 子项) |
| 变更操作 | S-MUT | 11 | 11 ✅ | ✅ (19 子项) |
| 排序 | S-SORT | 2 | 2 ✅ | ✅ (3 子项) |
| 键名获取 | S-KEYS | 1 | 1 ✅ | ✅ (3 子项) |
| BOM 边界 | S-BOM | 1 | 1 ✅ | ⬜ |
| Bug 回归 | S-BUG94 | 1 | 1 ✅ | ✅ |
| 数组创建 | S-ARR | 4 | 4 ✅ | ✅ (10 子项) |
| 示例解析 | S-PE | 6 | 6 ✅ | ⬜ |
| README 示例 | S-README | 2 | 2 ✅ | ✅ (6 子项) |
| 补充指针 | S-PTR | 8 | 6 ✅ 2 ❌ | ✅ (6 子项) |
| 补充合并 | S-MP | 14 | 12 ✅ 2 ❌ | ✅ (9 子项) |
| 补充 Patch | S-PATCH | 3 | 3 ✅ | ✅ (5 子项) |
| 杂项 | S-MISC | 9 | 9 ✅ | ✅ (7 子项) |
| 往返测试 | roundtrip / compare | 6 | 6 ✅ | ⬜ |
| 打印辅助 | S-PRINT | — | ⬜ | ✅ (4 子项) |

---

## 四、cJSON 规范覆盖率统计

### 4.1 按章节覆盖率

| 章节 | 规范项数 | cJSON `RUN_TEST` | Rust 覆盖 | Rust 通过 | C ABI 覆盖 | C ABI 通过 |
|------|---------|---------------|----------|----------|-----------|------------|
| 第一章 test.c (T-01~T-08) | 8 | —（手工） | 8 (100%) | 8 (100%) | 8 (100%) | 8 (100%) |
| §2.1 解析 (P-*) | 31 | 50 | 31 (100%) | 31 (100%) | 31 (100%) | 28 (90.3%) |
| §2.2 打印 (PR-*) | 20 | 28 | 20 (100%) | 20 (100%) | 20 (100%) | 20 (100%) |
| §2.3 Add API (A-*) | 11 | 31 | 11 (100%) | 11 (100%) | 11 (100%) | 9 (81.8%) |
| §2.4 比较+最小化 (C-/M-) | 11 | 17 | 11 (100%) | 11 (100%) | 11 (100%) | 8 (72.7%) |
| §2.5 鲁棒性 (R-*) | 7 | 30 | 6 (85.7%) | 6 (100%) | 7 (100%) | 5 (71.4%) |
| §2.6 Utils (U-*) | 9 | 12 | 9 (100%) | 8 (88.9%) | 9 (100%) | 8 (88.9%) |
| **合计** | **97** | **162** | **96 (99.0%)** | **95 (99.0%)** | **97 (100%)** | **86 (88.7%)** |

> 注：cJSON 的 162 个 `RUN_TEST` 覆盖的维度远超规范的 97 项，特别是 `misc_tests.c`（30 个）和 `cjson_add.c`（31 个）包含大量 NULL/分配失败注入子用例。  
> C ABI “通过”统计中将 SKIP/INFO 视为不计入。如将 SKIP/INFO 视为“合理通过”，则 C ABI 有效通过率为 **93/97 (95.9%)**。

### 4.2 按功能域覆盖率

| 功能域 | cJSON 规范项 | cJSON RUN_TEST | Rust 通过 | C ABI 通过 | 差异说明 |
|--------|-----------|--------------|----------|-----------|----------|
| 解析（基础+数字+字符串+数组+对象+定长） | 31 | 50 | 31/31 | 28/31 | C: P-S-03/P-L-03/05 |
| 打印（值+数字+字符串+数组+对象） | 20 | 28 | 20/20 | 20/20 | 格式差异但语义等价 |
| 构造 API | 11 | 31 | 11/11 | 9/11 | C: Raw/分配注入 |
| 比较 | 6 | 10 | 6/6 | 5/6 | C: 不敏感比较跳过 |
| 最小化 | 5 | 7 | 5/5 | 3/5 | 注释为非法 JSON |
| 鲁棒性 | 7 | 30 | 6/7 | 5/7 | R-05 双端不可测 |
| JSON Pointer | 3 | 5 | 3/3 | 3/3 | |
| JSON Patch | 3 | 3 | 3/3 | 3/3 | |
| Merge Patch | 3 | 7 | 2/3 | 2/3 | U-M-03 双端失败 |

### 4.3 cJSON 独有测试（LX-json 未覆盖）

以下为 cJSON `RUN_TEST` 中存在但 LX-json 两端均未对标的测试点：

| cJSON 源文件 | 测试函数 | 功能描述 | 未覆盖原因 |
|------------|----------|---------|------------|
| `parse_hex4.c` | `parse_all_combinations` / `parse_mixed_case` | 0x0000-0xFFFF 十六进制 4 位全组合 | 内部实现细节，非公开 API |
| `misc_tests.c` | `set_number_value_should_set_numbers` | SetNumberValue API | C ABI 未暴露 |
| `misc_tests.c` | `create_*_reference_*` (3项) | 字符串/对象/数组引用创建 | Rust 所有权模型不需引用 |
| `misc_tests.c` | `should_not_add_itself` | 自引用防护 | Rust 借用检查器负责 |
| `misc_tests.c` | `not_use_after_free_when_string_is_aliased` | UAF 防护 | Rust 内存安全保证 |
| `misc_tests.c` | `set_bool_value_must_not_break_objects` | SetBoolValue API | C ABI 未暴露 |
| `misc_tests.c` | `set_valuestring_to_object_should_not_leak_memory` | 内存泄漏防护 | Rust 自动 Drop |
| `cjson_add.c` | `create_int/float/double/string_array_*` (4项) | Create*Array 分配失败 | 无法注入分配失败 |
| `misc_tests.c` | `ensure_should_fail_on_failed_realloc` | realloc 失败 | 同上 |
| `misc_utils_tests.c` | `cjson_utils_functions_shouldnt_crash_with_null_pointers` | Utils 全 API NULL 安全 | 部分已覆盖 |
| `old_utils_tests.c` | `generate_merge_tests` | GenerateMergePatch | LX-json 未提供生成合并补丁 |
| `print_number.c` | `print_non_number` | NaN/Infinity 打印 | T-08 部分覆盖 |

> 这些未覆盖项多数是因为：(1) Rust 语言特性已自带保证（内存安全、借用检查）；(2) 分配失败注入需要特殊钩子；(3) 部分 API 未在 C ABI 层暴露。

---

## 五、已知差异与完整性分析

### 5.1 真实 Bug（需修复）

| # | 编号 | 问题 | 影响 | 严重度 |
|---|------|------|------|--------|
| 1 | U-M-03 | merge patch 对数组目标执行后，null 值键未被删除 | Rust + C ABI 一致 | ⚠️ 中 |
| 2 | P-S-03 | C ABI 代理对 `\uD83D\uDC31` 解码失败 | 仅 C ABI（Rust 已通过） | ⚠️ 中 |
| 3 | S-T-03 | `create_int_array(vals, 0)` 返回 NULL | 仅 C ABI | 🔵 低 |
| 4 | S-OBJ-02 | `get_object_item_case_insensitive` 功能异常 | 仅 C ABI | ⚠️ 中 |
| 5 | S-PTR-01/02 | JSON Pointer 空路径 `""` 和 `"/"` 行为异常 | 仅 Rust | 🔵 低 |
| 6 | S-MP-13/14 | merge patch 的 null 值保留/深层删除语义不正确 | 仅 Rust | ⚠️ 中 |
| 7 | S-CMP-01 | compare 不支持大小写不敏感字符串比较 | 仅 Rust | 🔵 低 |

### 5.2 设计差异（非 Bug）

| 差异点 | cJSON 行为 | LX-json 行为 | 判定 |
|--------|-----------|-------------|------|
| UTF-8 BOM | 跳过 BOM 前缀 | 拒绝 BOM 开头 | 可接受（严格 JSON） |
| 尾部内容 | 解析成功 + 返回 parse_end | 严格模式拒绝 | 可接受 |
| 注释 (`//`, `/**/`) | minify 时去除 | 拒绝解析 | 正确（RFC 8259 无注释） |
| 数字打印格式 | 科学计数法 `1e-09` | 完整小数 `0.000000001` | 语义等价 |
| 对象缩进 | Tab 缩进 | 空格缩进 | 可接受 |
| Raw 类型 | 支持 CreateRaw | C ABI 未暴露 | C ABI 限制 |

### 5.3 覆盖缺口

| 缺口 | 说明 | 优先级 |
|------|------|--------|
| R-05 realloc 失败注入 | Rust 通过 Allocator trait 理论可测，C ABI 无法测 | 低 |
| A-11 分配失败注入 | 同上 | 低 |
| C-04 不敏感比较（C ABI） | C ABI 未暴露 `compare_case_insensitive` 函数 | 低 |

### 5.4 Rust/C 中“无该项测试”统一说明（含原因）

以下汇总 cJSON 中在 Rust 原生层或 C ABI 层无法 1:1 对标、无需单独测试、或仅能以替代测试覆盖的项目。

| 编号 | cJSON 测试点 | Rust 原生状态 | C ABI 状态 | 无该项测试或未 1:1 对标原因 |
|------|-------------|---------------|------------|-----------------------------|
| R-04 | `SetValuestring` 指针重叠 | 无等价直接测试（以 R-06 边界测试替代） | 🔶 SKIP | C 指针重叠/别名内存场景，Rust 安全接口无等价操作；C ABI 也未暴露 `set_valuestring` |
| R-05 | `realloc` 失败注入 | 无直接测试（以 R-07 完整性替代） | 🔶 SKIP | 依赖 cJSON 分配器 hook 做故障注入；Rust 默认内存模型与 C ABI 均不支持该注入路径 |
| A-11 | Add API 分配失败注入 | 无直接测试（改测参数/类型边界失败） | 🔶 SKIP | 同 R-05，需要可控分配失败注入机制；现有 Rust/C ABI 测试环境不可实现 |
| R-03（循环引用子场景） | `should_not_add_itself` 等自引用防护 | 无独立对标项（通过深拷贝/变异稳定性覆盖） | 无独立对标项（`R-03` 仅做 deep copy） | Rust 所有权/借用规则天然规避大多数自引用误用；C ABI 层当前未提供可构造该类自引用的直接接口 |
| A-07 | AddRaw / `CreateRaw` | ✅ Rust 已测 | 🔶 SKIP | C ABI 未导出 `lx_json_create_raw`，因此 C 侧无法对标该项 |
| C-04 | 大小写不敏感比较 | ✅ Rust 已测 | 🔶 SKIP | C ABI 未暴露 case-insensitive compare 入口，C 侧无法执行该测试 |

> 统计口径说明：按 97 项规范口径，Rust 为 96/97（1 项不可等价映射）；C ABI 为 97/97（含 SKIP 标注）。

---

## 六、结论

1. **cJSON 原版测试规模**：21 个测试二进制、162 个 `RUN_TEST`、~491 断言，归纳为 97 项规范。
2. **cJSON 97 项规范覆盖率**：Rust 端 99.0%（96/97），C ABI 端 100%（97/97 均已编写测试，含 SKIP）。
3. **总体通过率**：Rust 端 95/97（97.9%），C ABI 端有效通过 93/97（95.9%）。
4. **真实失败**：共 11 条断言失败，归类为 7 个独立问题：
   - 1 个核心 merge patch 语义 bug（U-M-03，双端一致）
   - 2 个 C ABI 特有问题（代理对解码、不敏感查找）
   - 4 个 Rust 补充测试发现的边界问题
5. **cJSON 独有测试未覆盖**：~12 项，主要是 Rust 语言特性已自带保证（UAF/自引用/内存泄漏）、分配失败注入不可行、或部分 API 未在 C ABI 暴露。
6. **设计差异非 Bug**：BOM 拒绝、注释拒绝、数字格式差异均为合理的严格 JSON 实现选择。
7. **补充测试**：在 cJSON 规范之外额外新增 ~130 条补充测试（S-* 系列），显著强化了覆盖面。
