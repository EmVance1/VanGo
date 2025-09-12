#ifndef VANGO_CASSERTS_H
#define VANGO_CASSERTS_H


struct VangoTestResult {
    char* msg;
    unsigned int failline;
};


#define vg_assert(a)          do { if (!(a))       { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expression expected to be 'true' was 'false'"; return; } } while (0)
#define vg_assert_eq(a, b)    do { if ((a) != (b)) { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expressions expected to be equal were not equal"; return; } } while (0)
#define vg_assert_ne(a, b)    do { if ((a) == (b)) { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expressions expected not to be equal were equal"; return; } } while (0)
#define vg_assert_null(a)     do { if ((a))        { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expected 'NULL', received other address"; return; } } while (0)
#define vg_assert_non_null(a) do { if (!(a))       { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expected valid pointer, received 'NULL'"; return; } } while (0)


#define vango_test(name) void name(struct VangoTestResult* _vango_test_result)
#define vango_test_decl(name) void name(struct VangoTestResult* _vango_test_result)


#ifdef VANGO_TEST_ROOT

#include <stdio.h>
#include <string.h>

#define vango_test_reg(name) _vango_test_register_impl(argc, argv, &_vg_failures, #name, name)
#define vango_test_main(tests) int main(int argc, char** argv) { int _vg_failures = 0; tests; return _vg_failures; }


static void _vango_test_register_impl(int argc, char** argv, int* _vg_failures, const char* name, void(*f)(struct VangoTestResult*)) {
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
    struct VangoTestResult test_result = { .failline=0, .msg=NULL };
    f(&test_result);
    if (test_result.msg == NULL) {
        fprintf(stderr, "\033[32m[VanGo:  info] passed '%s'\033[m\n", name);
    } else {
        fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed '%s' on line %u: \033[m%s\n", name, test_result.failline, test_result.msg);
        (*_vg_failures)++;
    }
}

#endif

#endif
