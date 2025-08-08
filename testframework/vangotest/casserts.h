#ifndef CASSERTS_H
#define CASSERTS_H
#include <stdlib.h>


typedef struct VangoTestResult {
    size_t failtype;
    size_t failline;
    char* msg;
} VangoTestResult;


#define FAIL_TRUE     1
#define FAIL_EQ       2
#define FAIL_NE       3
#define FAIL_NULL     4
#define FAIL_NON_NULL 5


#define assert(a)          if (!a)     { _test_result->failtype=FAIL_TRUE,     _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expression expected to be 'true' was 'false'"; return; }
#define assert_eq(a, b)    if (a != b) { _test_result->failtype=FAIL_EQ,       _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expressions expected to be equal were not equal"; return; }
#define assert_ne(a, b)    if (a == b) { _test_result->failtype=FAIL_NE,       _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expressions expected not to be equal were equal"; return; }
#define assert_null(a)     if (a)      { _test_result->failtype=FAIL_NULL,     _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expected 'NULL', received other address"; return; }
#define assert_non_null(a) if (a)      { _test_result->failtype=FAIL_NON_NULL; _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expected valid pointer, received 'NULL'"; return; }


#define test(name) void name(VangoTestResult* _test_result)
#define decl_test(name) void name(VangoTestResult* _test_result)


#ifdef TEST_ROOT

#include <stdio.h>
#include <string.h>

#define test_register(name) _test_register_impl(argc, argv, #name, name)
#define test_main(tests) int main(int argc, char** argv) { tests }


void _test_register_impl(int argc, char** argv, const char* name, void(*f)(VangoTestResult*)) {
    if (argc == 1) {
        goto run_test;
    } else {
        for (int i = 1; i < argc; i++) {
            if (strcmp(argv[i], name) == 0) {
                goto run_test;
            }
        }
    }
    return;

run_test:
    VangoTestResult test_result = { .failtype=0, .failline=0, .msg=NULL };
    f(&test_result);
    if (test_result.failtype == 0) {
        fprintf(stderr, "\033[32m[VanGo:  info] passed '%s'\033[m\n", name);
    } else {
        fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed '%s' on line %llu: \033[m%s\n", name, test_result.failline, test_result.msg);
    }
}

#endif

#endif
