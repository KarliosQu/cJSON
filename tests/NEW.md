# 测试补充变更记录

## 变更概述

本轮基于 `CJSON_TESTING_REFERENCE.md` 中对 cJSON 测试规范的分析，对 LX-json 的测试套件进行了全面补充。
补充目标：覆盖 cJSON 具备但 LX-json 原测试未触及的功能边界，通过测试失败检验 LX-json 相对 cJSON 的功能缺失。

## 变更统计

| 文件 | 变更前 | 变更后 | 新增 | 类型 |
|------|--------|--------|------|------|
| `tests/test_1.rs` | 11 `#[test]` / 44 `assert!` | 17 `#[test]` / 58 `assert!` | +6 测试 / +14 断言 | Rust |
| `tests/test_2.rs` | 95 `#[test]` / 168 `assert!` | 169 `#[test]` / 357 `assert!` | +74 测试 / +189 断言 | Rust |
| `tests/test_1_c.c` | 8 函数 / 47 `TEST_ASSERT` | 14 函数 / 60 `TEST_ASSERT` | +6 函数 / +13 断言 | C ABI |
| `tests/test_2_c.c` | 16 函数 / 156 `TEST_ASSERT` | 31 函数 / 264 `TEST_ASSERT` | +15 函数 / +108 断言 | C ABI |
| `tests/RUST_TESTING_DOC.md` | — | — | 更新 | 文档 |
| `tests/C_ABI_TESTING_DOC.md` | — | — | 更新 | 文档 |
| `tests/NEW.md` | — | — | 新建 | 文档 |

**总计**：新增 80 个 Rust 测试（+203 断言），21 个 C ABI 测试函数（+121 断言）。

## test_1.rs 新增测试（6 项）

| 编号 | 测试名 | 覆盖内容 |
|------|--------|---------|
| ST-01 | `test_1_st01_create_double_array` | `create_double_array` 批量创建双精度数组 |
| ST-02 | `test_1_st02_create_float_array` | `create_float_array` 批量创建浮点数组 |
| ST-03 | `test_1_st03_create_empty_int_array` | 空数组（count=0）创建 |
| ST-04 | `test_1_st04_int_array_with_negatives` | 含负数的整型数组创建 |
| ST-05 | `test_1_st05_print_buffered_unformatted` | `print_buffered` 紧凑模式输出 |
| ST-06 | `test_1_st06_deeply_nested_construction` | 深层嵌套对象构造（3层以上） |

## test_2.rs 新增测试（74 项）

### 按类别分组

| 类别 | 前缀 | 数量 | 覆盖内容 |
|------|------|------|---------|
| 类型检查 | S-TC | 1 | 综合类型谓词验证 |
| 对象项边界 | S-OBJ | 4 | 对数组调用 `get_object_item` 不崩溃、大小写查找 |
| 值访问器 | S-ACC | 2 | `get_string_value`/`get_number_value` 正常与错误类型 |
| 补充比较 | S-CMP | 5 | 字符串大小写比较、Raw/科学计数法/键序/数组顺序 |
| 变异操作 | S-MUT | 11 | `detach`/`replace`/`insert`/`delete` 全面测试 |
| 排序 | S-SORT | 2 | 对象键排序（大小写敏感/不敏感） |
| 键获取 | S-KEYS | 1 | `get_object_keys` 获取所有键名 |
| BOM 处理 | S-BOM | 1 | BOM 出现在非首字节位置 |
| Bug 回归 | S-BUG94 | 1 | 复杂转义序列解析 |
| 数组批量创建 | S-ARR | 4 | int/float/double/string 数组 |
| 示例解析 | S-PE | 6 | cJSON 官方测试输入文件解析 |
| README 示例 | S-README | 2 | Monitor 构造与 FullHD 查询 |
| JSON Pointer | S-PTR | 8 | 根指针、空键、特殊字符键、深层寻址 |
| Merge Patch | S-MP | 14 | RFC 7396 完整行为矩阵 |
| JSON Patch | S-PATCH | 3 | `add_patch_to_array`、多操作补丁、空对象 diff |
| 杂项 | S-MISC | 9 | `has_object_item`、深浅拷贝、遍历、无效输入等 |

## test_1_c.c 新增测试函数（6 项）

| 函数名 | 覆盖内容 |
|--------|---------|
| `test_1_st01_double_array` | `lx_json_create_double_array` |
| `test_1_st02_float_array` | `lx_json_create_float_array` |
| `test_1_st03_empty_int_array` | 空整型数组 |
| `test_1_st04_int_array_negatives` | 负数整型数组 |
| `test_1_st05_print_buffered_compact` | `lx_json_print_buffered` 紧凑模式 |
| `test_1_st06_deep_nested` | 深层嵌套对象 |

## test_2_c.c 新增测试函数（15 项）

| 函数名 | 覆盖内容 |
|--------|---------|
| `test_2_typecheck` | 所有 `lx_json_is_*` 类型谓词 |
| `test_2_get_object_item_edge` | 对数组调用 `get_object_item` 不崩溃 |
| `test_2_accessor` | `get_string_value`/`get_number_value` |
| `test_2_compare_extra` | 科学计数法/键序无关/数组顺序比较 |
| `test_2_mutate` | detach/replace/insert/delete |
| `test_2_sort` | 对象键排序 |
| `test_2_keys` | `lx_json_get_object_keys` |
| `test_2_create_arrays` | 4 种数组批量创建 |
| `test_2_pointer_extra` | JSON Pointer 特殊路径 |
| `test_2_merge_patch_extra` | RFC 7396 扩展场景 |
| `test_2_patch_extra` | JSON Patch 多操作 |
| `test_2_print_preallocated` | 预分配打印 |
| `test_2_bug94` | 转义序列回归 |
| `test_2_readme_monitor` | README 示例复现 |
| `test_2_misc` | 杂项（遍历、拷贝、strict 模式） |

## Rust 测试结果

### test_1.rs: 17/17 通过（100%）

所有 17 项测试（含 6 项新增）全部通过。

### test_2.rs: 163/169 通过（96.4%），6 项失败

| 失败测试 | 分类 | 原因 |
|---------|------|------|
| `test_2_um03_merge_patch_replace_array` | U-M-03 | merge_patch 对非对象 target 替换时未删除 null 键 |
| `test_2_scmp01_compare_strings` | S-CMP-01 | 字符串比较大小写不敏感，与 cJSON 行为不一致 |
| `test_2_sptr01_pointer_root` | S-PTR-01 | 空指针 `""` 未返回根节点 |
| `test_2_sptr02_pointer_empty_key` | S-PTR-02 | `"/"` 指针对空键查找不工作 |
| `test_2_smp13_merge_preserve_null_value` | S-MP-13 | merge_patch 删除了应保留的 null 值字段 |
| `test_2_smp14_merge_deep_null_delete` | S-MP-14 | 深层 null 键未被正确递归删除 |

## 发现的 LX-json 功能差异汇总

### 新发现的差异（本轮补充测试揭露）

| 编号 | 差异项 | cJSON 行为 | LX-json 行为 | 严重程度 |
|------|--------|-----------|-------------|---------|
| 1 | 字符串比较大小写 | `compare` 始终大小写敏感 | 大小写不敏感 | 中 |
| 2 | JSON Pointer 空路径 | `""` 返回根节点 | 返回 None | 中 |
| 3 | JSON Pointer 空键 | `"/"` 查找空字符串键 | 返回 None | 中 |
| 4 | Merge Patch null 保留 | target 中已有的 null 字段在无关 patch 时保留 | 可能被删除 | 中 |
| 5 | Merge Patch 深层 null | 嵌套 patch 中的 null 值递归删除 | 未递归删除 | 中 |

### 已知差异（原有测试已记录）

| 编号 | 差异项 | 来源 |
|------|--------|------|
| 1 | UTF-16 代理对不支持 | P-S-03 |
| 2 | UTF-8 BOM 不支持 | P-L-05 |
| 3 | 注释剥离不支持 | M-02, M-03 |
| 4 | 尾部字符严格拒绝 | P-L-03 |
| 5 | PrintPreallocated 自动扩容 | T-07 |
| 6 | Infinity 打印为 inf | T-08 |
| 7 | Merge Patch 非对象替换 | U-M-03 |
