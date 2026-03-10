/**
 * @file test_1_c.c
 * @brief LX-json C ABI tests corresponding to cJSON test.c (Chapter 1)
 *
 * Tests T-01 through T-08 from CJSON_TESTING_REFERENCE.md
 * Covers: constructing JSON via API, printing, PrintPreallocated, numeric boundaries
 *
 * Build (Windows, MSVC):
 *   cl /nologo /W3 tests\test_1_c.c /I include /link /LIBPATH:target\debug lx_json.dll.lib /OUT:tests\test_1_c.exe
 *
 * Build (Windows, GCC/MinGW):
 *   gcc -Wall -Wextra tests/test_1_c.c -I include -L target/debug -llx_json -o tests/test_1_c.exe
 *
 * Run:
 *   tests\test_1_c.exe
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "lx_json.h"

/* ========================================================================== */
/* Simple test framework                                                       */
/* ========================================================================== */

static int g_tests_run    = 0;
static int g_tests_passed = 0;
static int g_tests_failed = 0;

#define TEST_ASSERT(cond, msg) do { \
    g_tests_run++; \
    if (cond) { \
        g_tests_passed++; \
        printf("  [PASS] %s\n", msg); \
    } else { \
        g_tests_failed++; \
        printf("  [FAIL] %s (line %d)\n", msg, __LINE__); \
    } \
} while(0)

#define TEST_BEGIN(name) printf("\n=== %s ===\n", name)

/* ========================================================================== */
/* T-01: Construct Video object                                                */
/* ========================================================================== */

static void test_1_t01_create_video(void)
{
    TEST_BEGIN("T-01: Construct Video object");

    /* Build root object */
    LXJsonNode *root = lx_json_create_object();
    TEST_ASSERT(root != NULL, "create root object");

    /* name */
    int r = lx_json_add_string_to_object(root, "name", "Jack (\"Bee\") Nimble");
    TEST_ASSERT(r == 1, "add name string");

    /* format sub-object */
    LXJsonNode *fmt = lx_json_create_object();
    TEST_ASSERT(fmt != NULL, "create format object");

    lx_json_add_string_to_object(fmt, "type", "rect");
    lx_json_add_number_to_object(fmt, "width", 1920.0);
    lx_json_add_number_to_object(fmt, "height", 1080.0);
    lx_json_add_bool_to_object(fmt, "interlace", 0);
    lx_json_add_number_to_object(fmt, "frame rate", 24.0);

    r = lx_json_add_item_to_object(root, "format", fmt);
    TEST_ASSERT(r == 1, "add format sub-object");

    /* Print and verify */
    char *out = lx_json_print(root);
    TEST_ASSERT(out != NULL, "lx_json_print succeeded");
    if (out) {
        TEST_ASSERT(strlen(out) > 0, "printed string is non-empty");
        TEST_ASSERT(strstr(out, "Jack") != NULL, "output contains name");
        TEST_ASSERT(strstr(out, "format") != NULL, "output contains format key");
        TEST_ASSERT(strstr(out, "1920") != NULL, "output contains width");
        printf("  Output preview: %.200s...\n", out);
        lx_json_free_string(out);
    }

    lx_json_free(root);
}

/* ========================================================================== */
/* T-02: Construct string array [Sunday..Saturday]                             */
/* ========================================================================== */

static void test_1_t02_string_array(void)
{
    TEST_BEGIN("T-02: String array [Sunday..Saturday]");

    const char *days[] = {
        "Sunday", "Monday", "Tuesday", "Wednesday",
        "Thursday", "Friday", "Saturday"
    };

    LXJsonNode *arr = lx_json_create_string_array(days, 7);
    TEST_ASSERT(arr != NULL, "create string array");
    TEST_ASSERT(lx_json_is_array(arr), "node is array");
    TEST_ASSERT(lx_json_get_array_size(arr) == 7, "array size is 7");

    /* Verify first and last element */
    LXJsonNode *first = lx_json_get_array_item(arr, 0);
    if (first) {
        char *s = lx_json_get_string_value(first);
        TEST_ASSERT(s != NULL && strcmp(s, "Sunday") == 0, "first element is Sunday");
        lx_json_free_string(s);
        lx_json_free(first);
    }

    LXJsonNode *last = lx_json_get_array_item(arr, 6);
    if (last) {
        char *s = lx_json_get_string_value(last);
        TEST_ASSERT(s != NULL && strcmp(s, "Saturday") == 0, "last element is Saturday");
        lx_json_free_string(s);
        lx_json_free(last);
    }

    /* Print */
    char *out = lx_json_print(arr);
    TEST_ASSERT(out != NULL, "print string array");
    if (out) {
        TEST_ASSERT(strstr(out, "Sunday") != NULL, "output contains Sunday");
        TEST_ASSERT(strstr(out, "Saturday") != NULL, "output contains Saturday");
        lx_json_free_string(out);
    }

    lx_json_free(arr);
}

/* ========================================================================== */
/* T-03: Construct 2D integer matrix array                                     */
/* ========================================================================== */

static void test_1_t03_matrix(void)
{
    TEST_BEGIN("T-03: 2D integer matrix");

    /* Create a 3x3 matrix: [[1,0,0],[0,1,0],[0,0,1]] (identity) */
    LXJsonNode *matrix = lx_json_create_array();
    TEST_ASSERT(matrix != NULL, "create matrix array");

    int64_t row0[] = {1, 0, 0};
    int64_t row1[] = {0, 1, 0};
    int64_t row2[] = {0, 0, 1};

    LXJsonNode *r0 = lx_json_create_int_array(row0, 3);
    LXJsonNode *r1 = lx_json_create_int_array(row1, 3);
    LXJsonNode *r2 = lx_json_create_int_array(row2, 3);

    lx_json_add_item_to_array(matrix, r0);
    lx_json_add_item_to_array(matrix, r1);
    lx_json_add_item_to_array(matrix, r2);

    TEST_ASSERT(lx_json_get_array_size(matrix) == 3, "matrix has 3 rows");

    /* Verify nested structure */
    LXJsonNode *check_row = lx_json_get_array_item(matrix, 0);
    if (check_row) {
        TEST_ASSERT(lx_json_is_array(check_row), "first row is array");
        TEST_ASSERT(lx_json_get_array_size(check_row) == 3, "first row has 3 cols");
        lx_json_free(check_row);
    }

    /* Print */
    char *out = lx_json_print(matrix);
    TEST_ASSERT(out != NULL, "print matrix");
    if (out) {
        TEST_ASSERT(strlen(out) > 0, "matrix output non-empty");
        printf("  Output preview: %.200s...\n", out);
        lx_json_free_string(out);
    }

    lx_json_free(matrix);
}

/* ========================================================================== */
/* T-04: Construct Image object (with Thumbnail, IDs)                          */
/* ========================================================================== */

static void test_1_t04_image(void)
{
    TEST_BEGIN("T-04: Image object with Thumbnail and IDs");

    LXJsonNode *root = lx_json_create_object();

    /* Image metadata */
    lx_json_add_number_to_object(root, "Width", 800.0);
    lx_json_add_number_to_object(root, "Height", 600.0);
    lx_json_add_string_to_object(root, "Title", "View from 15th Floor");
    lx_json_add_bool_to_object(root, "Animated", 0);

    /* Thumbnail sub-object */
    LXJsonNode *thumb = lx_json_create_object();
    lx_json_add_string_to_object(thumb, "Url", "http://www.example.com/image/481989943");
    lx_json_add_number_to_object(thumb, "Height", 125.0);
    lx_json_add_number_to_object(thumb, "Width", 100.0);
    lx_json_add_item_to_object(root, "Thumbnail", thumb);

    /* IDs array */
    int64_t ids[] = {116, 943, 234, 38793};
    LXJsonNode *id_arr = lx_json_create_int_array(ids, 4);
    lx_json_add_item_to_object(root, "IDs", id_arr);

    /* Verify fields */
    TEST_ASSERT(lx_json_has_object_item(root, "Width"), "has Width");
    TEST_ASSERT(lx_json_has_object_item(root, "Thumbnail"), "has Thumbnail");
    TEST_ASSERT(lx_json_has_object_item(root, "IDs"), "has IDs");

    /* Verify Thumbnail sub-object */
    LXJsonNode *th = lx_json_get_object_item(root, "Thumbnail");
    if (th) {
        TEST_ASSERT(lx_json_is_object(th), "Thumbnail is object");
        TEST_ASSERT(lx_json_has_object_item(th, "Url"), "Thumbnail has Url");
        lx_json_free(th);
    }

    /* Print */
    char *out = lx_json_print(root);
    TEST_ASSERT(out != NULL, "print Image object");
    if (out) {
        TEST_ASSERT(strstr(out, "Thumbnail") != NULL, "output has Thumbnail");
        TEST_ASSERT(strstr(out, "IDs") != NULL, "output has IDs");
        printf("  Output preview: %.300s...\n", out);
        lx_json_free_string(out);
    }

    lx_json_free(root);
}

/* ========================================================================== */
/* T-05: Construct records array (object array)                                */
/* ========================================================================== */

static void test_1_t05_records(void)
{
    TEST_BEGIN("T-05: Records array (object array)");

    LXJsonNode *records = lx_json_create_array();

    /* Record 1 */
    LXJsonNode *r1 = lx_json_create_object();
    lx_json_add_string_to_object(r1, "name", "Alice");
    lx_json_add_number_to_object(r1, "age", 30.0);
    lx_json_add_string_to_object(r1, "city", "Beijing");
    lx_json_add_item_to_array(records, r1);

    /* Record 2 */
    LXJsonNode *r2 = lx_json_create_object();
    lx_json_add_string_to_object(r2, "name", "Bob");
    lx_json_add_number_to_object(r2, "age", 25.0);
    lx_json_add_string_to_object(r2, "city", "Shanghai");
    lx_json_add_item_to_array(records, r2);

    /* Record 3 */
    LXJsonNode *r3 = lx_json_create_object();
    lx_json_add_string_to_object(r3, "name", "Charlie");
    lx_json_add_number_to_object(r3, "age", 35.0);
    lx_json_add_string_to_object(r3, "city", "Guangzhou");
    lx_json_add_item_to_array(records, r3);

    TEST_ASSERT(lx_json_get_array_size(records) == 3, "records has 3 items");

    /* Verify each record */
    LXJsonNode *item0 = lx_json_get_array_item(records, 0);
    if (item0) {
        TEST_ASSERT(lx_json_is_object(item0), "record 0 is object");
        TEST_ASSERT(lx_json_has_object_item(item0, "name"), "record 0 has name");
        TEST_ASSERT(lx_json_has_object_item(item0, "age"), "record 0 has age");
        TEST_ASSERT(lx_json_has_object_item(item0, "city"), "record 0 has city");
        lx_json_free(item0);
    }

    /* Print */
    char *out = lx_json_print(records);
    TEST_ASSERT(out != NULL, "print records");
    if (out) {
        TEST_ASSERT(strstr(out, "Alice") != NULL, "output has Alice");
        TEST_ASSERT(strstr(out, "Bob") != NULL, "output has Bob");
        TEST_ASSERT(strstr(out, "Charlie") != NULL, "output has Charlie");
        lx_json_free_string(out);
    }

    lx_json_free(records);
}

/* ========================================================================== */
/* T-06: PrintPreallocated with sufficient buffer                              */
/* ========================================================================== */

static void test_1_t06_preallocated_success(void)
{
    TEST_BEGIN("T-06: PrintPreallocated buffer sufficient");

    LXJsonNode *root = lx_json_create_object();
    lx_json_add_string_to_object(root, "key", "value");
    lx_json_add_number_to_object(root, "num", 42.0);

    /* First get normal print to know size */
    char *normal = lx_json_print(root);
    TEST_ASSERT(normal != NULL, "normal print succeeded");

    if (normal) {
        size_t needed = strlen(normal);

        /* Allocate sufficient buffer: strlen + 5 extra bytes */
        size_t buf_len = needed + 5;
        char *buffer = (char *)calloc(buf_len + 1, 1);
        size_t required_length = 0;

        int result = lx_json_print_preallocated(root, buffer, buf_len + 1, 1, &required_length);
        TEST_ASSERT(result == 1, "preallocated print returns success");
        TEST_ASSERT(strlen(buffer) > 0, "buffer has content");
        printf("  Normal output length: %zu, Buffer size: %zu\n", needed, buf_len);
        printf("  Buffer content: %.100s...\n", buffer);

        free(buffer);
        lx_json_free_string(normal);
    }

    lx_json_free(root);
}

/* ========================================================================== */
/* T-07: PrintPreallocated with insufficient buffer                            */
/* ========================================================================== */

static void test_1_t07_preallocated_fail(void)
{
    TEST_BEGIN("T-07: PrintPreallocated buffer insufficient");

    LXJsonNode *root = lx_json_create_object();
    lx_json_add_string_to_object(root, "key", "value");
    lx_json_add_number_to_object(root, "num", 42.0);

    /* Use a very small buffer that cannot hold the output */
    char buffer[4] = {0};
    size_t required_length = 0;

    int result = lx_json_print_preallocated(root, buffer, sizeof(buffer), 1, &required_length);
    TEST_ASSERT(result == 0, "preallocated print returns failure for small buffer");
    printf("  Required length reported: %zu, Buffer size: %zu\n",
           required_length, sizeof(buffer));

    lx_json_free(root);
}

/* ========================================================================== */
/* T-08: Number = 1.0/0.0 (Infinity)                                          */
/* ========================================================================== */

static void test_1_t08_infinity_number(void)
{
    TEST_BEGIN("T-08: Number = 1.0/0.0 (Infinity)");

    LXJsonNode *root = lx_json_create_object();
    /* MSVC does not allow compile-time 1.0/0.0; use HUGE_VAL instead */
    double inf_val = HUGE_VAL;

    lx_json_add_number_to_object(root, "number", inf_val);

    /* The key requirement: no crash */
    char *out = lx_json_print(root);
    if (out) {
        printf("  Infinity output: %s\n", out);
        TEST_ASSERT(1, "print with infinity did not crash");
        lx_json_free_string(out);
    } else {
        /* NULL return is also acceptable - as long as no crash */
        TEST_ASSERT(1, "print with infinity returned NULL (acceptable)");
    }

    lx_json_free(root);
}

/* ========================================================================== */
/* Supplementary tests                                                         */
/* ========================================================================== */

/* S-T-01: create_double_array */
static void test_1_st01_double_array(void)
{
    TEST_BEGIN("S-T-01: create_double_array");

    double vals[] = {1.1, 2.2, 3.3, 4.4};
    LXJsonNode *arr = lx_json_create_double_array(vals, 4);
    TEST_ASSERT(arr != NULL && lx_json_is_array(arr), "double array created");
    TEST_ASSERT(lx_json_get_array_size(arr) == 4, "size is 4");
    if (arr) {
        char *s = lx_json_print_buffered(arr, 256, 0);
        TEST_ASSERT(s != NULL, "print double array");
        if (s) {
            /* Can reparse */
            LXJsonNode *rp = lx_json_parse(s);
            TEST_ASSERT(rp != NULL && lx_json_compare(arr, rp) != 0, "round-trip");
            if (rp) lx_json_free(rp);
            lx_json_free_string(s);
        }
        lx_json_free(arr);
    }
}

/* S-T-02: create_float_array */
static void test_1_st02_float_array(void)
{
    TEST_BEGIN("S-T-02: create_float_array");

    float vals[] = {1.5f, 2.5f, 3.5f};
    LXJsonNode *arr = lx_json_create_float_array(vals, 3);
    TEST_ASSERT(arr != NULL && lx_json_is_array(arr), "float array created");
    TEST_ASSERT(lx_json_get_array_size(arr) == 3, "size is 3");
    if (arr) lx_json_free(arr);
}

/* S-T-03: empty int_array */
static void test_1_st03_empty_int_array(void)
{
    TEST_BEGIN("S-T-03: empty int_array");

    LXJsonNode *arr = lx_json_create_int_array(NULL, 0);
    TEST_ASSERT(arr != NULL && lx_json_is_array(arr), "empty int array created");
    TEST_ASSERT(lx_json_get_array_size(arr) == 0, "size is 0");
    if (arr) {
        char *s = lx_json_print_buffered(arr, 64, 0);
        TEST_ASSERT(s != NULL && strcmp(s, "[]") == 0, "prints as []");
        lx_json_free_string(s);
        lx_json_free(arr);
    }
}

/* S-T-04: negatives in int_array */
static void test_1_st04_int_array_negatives(void)
{
    TEST_BEGIN("S-T-04: int_array with negatives");

    int64_t vals[] = {-100, 0, 100, -2147483648LL, 2147483647};
    LXJsonNode *arr = lx_json_create_int_array(vals, 5);
    TEST_ASSERT(arr != NULL && lx_json_get_array_size(arr) == 5, "int array size 5");
    if (arr) lx_json_free(arr);
}

/* S-T-05: print_buffered compact */
static void test_1_st05_print_buffered_compact(void)
{
    TEST_BEGIN("S-T-05: print_buffered compact");

    LXJsonNode *obj = lx_json_create_object();
    lx_json_add_string_to_object(obj, "key", "value");
    char *s = lx_json_print_buffered(obj, 64, 0);
    TEST_ASSERT(s != NULL && strcmp(s, "{\"key\":\"value\"}") == 0, "compact output");
    lx_json_free_string(s);
    lx_json_free(obj);
}

/* S-T-06: deeply nested construction */
static void test_1_st06_deep_nested(void)
{
    TEST_BEGIN("S-T-06: deeply nested construction");

    LXJsonNode *inner = lx_json_create_object();
    lx_json_add_number_to_object(inner, "deep", 42.0);

    LXJsonNode *middle = lx_json_create_object();
    lx_json_add_item_to_object(middle, "inner", inner);

    LXJsonNode *outer = lx_json_create_object();
    lx_json_add_item_to_object(outer, "middle", middle);

    char *s = lx_json_print(outer);
    TEST_ASSERT(s != NULL, "print nested");
    if (s) {
        LXJsonNode *rp = lx_json_parse(s);
        TEST_ASSERT(rp != NULL && lx_json_compare(outer, rp) != 0, "round-trip");
        if (rp) lx_json_free(rp);
        lx_json_free_string(s);
    }
    lx_json_free(outer);
}

/* ========================================================================== */
/* Main                                                                        */
/* ========================================================================== */

int main(void)
{
    printf("============================================\n");
    printf("LX-json C ABI Test Suite: test_1_c\n");
    printf("(Corresponds to cJSON test.c, Chapter 1)\n");
    printf("============================================\n");

    test_1_t01_create_video();
    test_1_t02_string_array();
    test_1_t03_matrix();
    test_1_t04_image();
    test_1_t05_records();
    test_1_t06_preallocated_success();
    test_1_t07_preallocated_fail();
    test_1_t08_infinity_number();

    /* Supplementary */
    test_1_st01_double_array();
    test_1_st02_float_array();
    test_1_st03_empty_int_array();
    test_1_st04_int_array_negatives();
    test_1_st05_print_buffered_compact();
    test_1_st06_deep_nested();

    printf("\n============================================\n");
    printf("Results: %d total, %d passed, %d failed\n",
           g_tests_run, g_tests_passed, g_tests_failed);
    printf("============================================\n");

    return g_tests_failed > 0 ? 1 : 0;
}
