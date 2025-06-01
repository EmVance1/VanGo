#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>


typedef struct TestResult {
    size_t err;
    size_t line;
    char* msg;
} TestResult;

const static TestResult TestOk = { .err=0, .line=0, .msg=NULL };


#define FAIL_TRUE     1
#define FAIL_EQ       2
#define FAIL_NE       3
#define FAIL_NULL     4
#define FAIL_NON_NULL 5


#define assert(a)          if (!a)     { return (TestResult){ .err=FAIL_TRUE,     .line=__LINE__, \
    .msg="assertion fail: expression expected to be 'true' was 'false'" }; }
#define assert_eq(a, b)    if (a != b) { return (TestResult){ .err=FAIL_EQ,       .line=__LINE__, \
    .msg="assertion fail: expressions expected to be equal were not equal" }; }
#define assert_ne(a, b)    if (a == b) { return (TestResult){ .err=FAIL_NE,       .line=__LINE__, \
    .msg="assertion fail: expressions expected not to be equal were equal" }; }
#define assert_null(a)     if (a)      { return (TestResult){ .err=FAIL_NULL,     .line=__LINE__, \
    .msg="assertion fail: expected 'NULL', received valid pointer" }; }
#define assert_non_null(a) if (a)      { return (TestResult){ .err=FAIL_NON_NULL, .line=__LINE__, \
    .msg="assertion fail: expected valid pointer, received 'NULL'" }; }


#define test(f) { \
    const TestResult res = f(); \
    if (res.err == 0) { \
        fprintf(stderr, "\033[32mtest '" #f "' passed\n\033[m"); \
    } else { \
        fprintf(stderr, "\033[31mtest '" #f "' failed on line %llu: %s\n\033[m", res.line, res.msg); \
    } }

