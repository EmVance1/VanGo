#ifndef CASSERTS_H
#define CASSERTS_H

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>


typedef struct TestResult {
    size_t failtype;
    size_t failline;
    char* msg;
} TestResult;

const static TestResult TestOk = { .failtype=0, .failline=0, .msg=NULL };

#define FAIL_TRUE     1
#define FAIL_EQ       2
#define FAIL_NE       3
#define FAIL_NULL     4
#define FAIL_NON_NULL 5


#define assert(a)          if (!a)     { _test_result->failtype=FAIL_TRUE,     _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expression expected to be 'true' was 'false'"; }
#define assert_eq(a, b)    if (a != b) { _test_result->failtype=FAIL_EQ,       _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expressions expected to be equal were not equal"; }
#define assert_ne(a, b)    if (a == b) { _test_result->failtype=FAIL_NE,       _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expressions expected not to be equal were equal"; }
#define assert_null(a)     if (a)      { _test_result->failtype=FAIL_NULL,     _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expected 'NULL', received other address"; }
#define assert_non_null(a) if (a)      { _test_result->failtype=FAIL_NON_NULL; _test_result->failline=__LINE__; \
    _test_result->msg="assertion fail: expected valid pointer, received 'NULL'"; }


void _test_register_impl(int argc, char** argv, const char* name, void(*f)(TestResult*));

#define test(name) void name(TestResult* _test_result)
#define test_register(name) _test_register_impl(argc, argv, #name, name)
#define test_main(tests) int main(int argc, char** argv) { tests }

#ifdef TEST_ROOT

void _test_register_impl(int argc, char** argv, const char* name, void(*f)(TestResult*)) {
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
    TestResult test_result = TestOk;
    f(&test_result);
    if (test_result.failtype == 0) {
        fprintf(stderr, "\033[32m[mscmp:  info] passed '%s'\033[m\n", name);
    } else {
        fprintf(stderr, "\033[32m[mscmp:  info] \033[31mfailed '%s' on line %llu: \033[m%s\n", name, test_result.failline, test_result.msg);
    }
}

#endif

#endif
