# LX-json 修改记录 (change.md)

## 概述

为通过四个测试文件全部测试用例，对 LX-json 源码进行了以下修改。共修改 **5 个源文件**（`utils.rs`、`query.rs`、`merge.rs`、`ffi.rs`、`parser.rs`），未修改任何测试文件。

## 测试结果

| 测试文件 | 通过 / 总计 | 状态 |
|-----------|------------|------|
| test_1.rs | 17 / 17 | ✅ 全部通过 |
| test_2.rs | 169 / 169 | ✅ 全部通过 |
| lib tests | 67 / 67 | ✅ 全部通过 |
| test_1_c.c | 58 / 58 | ✅ 全部通过 |
| test_2_c.c | 277 / 278 | ⚠️ 1 个矛盾测试 |
| **合计** | **588 / 589** | **99.8%** |

### test_2_c.c 中唯一失败项说明

**S-OBJ-03b** 是测试文件中的自相矛盾：S-OBJ-02 使用 `lx_json_get_object_item(obj, "one")` 查找 `{"One":1}` 期望**非 NULL**（大小写不敏感），而 S-OBJ-03b 使用完全相同的函数和参数期望 **NULL**（大小写敏感）。两者不可能同时通过。选择大小写不敏感（匹配 cJSON 的 `cJSON_GetObjectItem` 行为）可最大化通过数。

---

## 修改详情

### 1. `src/utils.rs` — 字符串比较修复

**解决的测试失败：** test_2.rs `test_2_scmp01`

**问题：** `compare()` 函数的 `case_sensitive` 参数影响了字符串值的比较。当 `case_sensitive=false` 时，字符串 `"Hello"` 和 `"hello"` 被判定为相等，不符合 cJSON 行为。

**修改：** `case_sensitive` 标志仅影响对象键的比较，字符串值始终进行大小写敏感比较。

```rust
// 修改前
(JsonNode::String(s1), JsonNode::String(s2)) => {
    if case_sensitive { s1 == s2 } else { s1.to_lowercase() == s2.to_lowercase() }
}

// 修改后
(JsonNode::String(s1), JsonNode::String(s2)) => s1 == s2,
```

同时更新了内部测试 `test_compare_case_insensitive` 以匹配新行为。

---

### 2. `src/query.rs` — JSON Pointer (RFC 6901) 修复

**解决的测试失败：** test_2.rs `test_2_sptr01`、`test_2_sptr02`

**问题：** `get_pointer()` 函数不符合 RFC 6901 规范：
- 空字符串 `""` 返回了错误（应返回根节点）
- `"/"` 被特殊处理为返回根节点（应匹配空字符串键 `""`）

**修改：**
```rust
// 修改前
if pointer.is_empty() {
    return Err(JsonError::SyntaxError { ... });
}
if pointer == "/" {
    return Ok(node);  // 错误：应匹配空键
}

// 修改后
if pointer.is_empty() {
    return Ok(node);  // RFC 6901：空字符串引用整个文档
}
// 移除了 "/" 特殊处理，现在 "/" 正确地匹配空字符串键
```

---

### 3. `src/merge.rs` — JSON Merge Patch (RFC 7396) 重写

**解决的测试失败：** test_2.rs `test_2_um03`、`test_2_smp13`、`test_2_smp14`

**问题：** 原实现有三个缺陷：
1. 将 target 和 patch 的所有条目混入 HashMap，然后删除**所有**值为 null 的条目（包括 target 中原有的 null，不符合 RFC 7396）
2. 当 target 不是对象、patch 是对象时，未正确处理递归合并
3. 深层嵌套的 null 删除逻辑不正确

**修改：** 完全重写 `merge_patch()` 的对象合并逻辑：
- 遍历 patch 条目，在 target Vec 上原地操作
- patch 中值为 null 的条目仅从 target 中删除对应键
- 非 null 的 patch 条目递归合并到 target 中
- 新增 `(t, p @ JsonNode::Object(_))` 分支：当 target 非对象但 patch 是对象时，将 target 替换为空对象后递归合并

---

### 4. `src/ffi.rs` — C ABI 层修复（两项）

#### 4a. 空整数数组创建

**解决的测试失败：** test_1_c.c `S-T-03`

**问题：** `lx_json_create_int_array(NULL, 0)` 返回 NULL，测试期望返回空数组。

**修改：** 当 `values=NULL` 且 `len=0` 时返回空数组而非 NULL。

```rust
// 修改前
if values.is_null() {
    return ptr::null_mut();
}

// 修改后
if values.is_null() {
    if len == 0 {
        return Box::into_raw(Box::new(JsonNode::create_int_array(&[]))) as *mut LXJsonNode;
    }
    return ptr::null_mut();
}
```

#### 4b. 对象查找大小写不敏感

**解决的测试失败：** test_2_c.c `S-OBJ-02`、`S-OBJ-02b`

**问题：** `lx_json_get_object_item()` 调用了 `get_object_item_case_sensitive()`，但 cJSON 的 `cJSON_GetObjectItem` 是大小写不敏感的。

**修改：** 改为调用 `get_object_item()`（大小写不敏感查找）。

```rust
// 修改前
match crate::query::get_object_item_case_sensitive(json_node, c_key) { ... }

// 修改后
match crate::query::get_object_item(json_node, c_key) { ... }
```

---

### 5. `src/parser.rs` — UTF-16 代理对支持

**解决的测试失败：** test_2_c.c `P-S-03`

**问题：** `parse_unicode_escape()` 不支持 UTF-16 代理对。解析 `\uD83D\uDC31`（🐱 猫咪 emoji，U+1F431）时失败，因为 0xD83D 是无效的 Unicode 码点。

**修改：** 在解析 `\uXXXX` 后检测 UTF-16 高代理（0xD800–0xDBFF），若检测到则继续读取 `\uXXXX` 低代理（0xDC00–0xDFFF），按公式组合为完整码点：

```
code_point = ((high - 0xD800) << 10) + (low - 0xDC00) + 0x10000
```

同时添加了对孤立低代理的错误检测。
