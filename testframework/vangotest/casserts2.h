#ifndef VANGO_CASSERTS_H
#define VANGO_CASSERTS_H


struct VangoTestResult {
    char* msg;
    unsigned int failline;
};

#define VANGO_TEST_PARAMS struct VangoTestResult* _vango_test_result

typedef void(*VangoTestFuncImpl)(VANGO_TEST_PARAMS);

struct VangoTestFunc {
    const char* id;
    VangoTestFuncImpl fn;
};


#define vg_assert(a)          do { if (!(a))       { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expression evaluated to 'false'"; return; } } while (0)
#define vg_assert_eq(a, b)    do { if ((a) != (b)) { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expressions expected to be equal were not equal"; return; } } while (0)
#define vg_assert_ne(a, b)    do { if ((a) == (b)) { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expressions expected not to be equal were equal"; return; } } while (0)
#define vg_assert_null(a)     do { if ((a))        { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expected 'NULL', received other address"; return; } } while (0)
#define vg_assert_non_null(a) do { if (!(a))       { _vango_test_result->failline=__LINE__; \
    _vango_test_result->msg="assertion fail: expected valid pointer, received 'NULL'"; return; } } while (0)


#if defined(_MSC_VER)
#pragma section("vgtest$a", read, write)
#pragma section("vgtest$v", read, write)
#pragma section("vgtest$z", read, write)
#define VANGO_SECTION_TESTS __declspec(allocate("vgtest$v"))
#elif defined(__clang__) || defined(__GNUC__)
#define VANGO_SECTION_TESTS __attribute__((used, section("vgtest")))
#else
#error compiler does not support automated testing
#endif

#define vango_test(name) \
    void name(VANGO_TEST_PARAMS); \
    VANGO_SECTION_TESTS struct VangoTestFunc _vango_test_##name = { .fn=name, .id=#name }; \
    void name(VANGO_TEST_PARAMS)


#ifdef VANGO_TEST_ROOT

#if defined(_MSC_VER)
#ifdef VANGO_TEST_MEMORY_LEAKS
#define _CRTDBG_MAP_ALLOC
#include <crtdbg.h>
#endif
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

__declspec(allocate("vgtest$a")) struct VangoTestFunc _start_vgtest = {};
__declspec(allocate("vgtest$z")) struct VangoTestFunc _stop_vgtest = {};

int main(int argc, char** argv) {
#ifdef VANGO_TEST_MEMORY_LEAKS
    _CrtSetReportMode(_CRT_WARN, _CRTDBG_MODE_FILE);
    _CrtSetReportFile(_CRT_WARN, _CRTDBG_FILE_STDERR);
    _CrtSetDbgFlag(_CRTDBG_ALLOC_MEM_DF | _CRTDBG_LEAK_CHECK_DF);
#endif

    char** _vg_begin = (char**)(&_start_vgtest+1);
    char** _vg_end = (char**)&_stop_vgtest;

    int _vg_failures = 0;

    for (; _vg_begin < _vg_end; _vg_begin++) {
        if (*_vg_begin == 0) { continue; }
        struct VangoTestFunc* _vg_f = (struct VangoTestFunc*)_vg_begin;
        int _vg_run_this = argc == 1 ? 1 : 0;
        if (argc > 1) {
            for (int i = 1; i < argc; i++) {
                if (strcmp(_vg_f->id, argv[i]) == 0) {
                    _vg_run_this = 1;
                    break;
                }
            }
        }
        if (_vg_run_this != 0) {
            struct VangoTestResult _vango_test_result = { .msg = NULL, .failline=0 };
            (_vg_f->fn)(&_vango_test_result);
            if (_vango_test_result.msg == NULL) {
                fprintf(stderr, "\033[32m[VanGo:  info] passed '%s'\033[m\n", _vg_f->id);
            } else {
                fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed '%s' on line %u: \033[m%s\n",
                    _vg_f->id, _vango_test_result.failline, _vango_test_result.msg);
                _vg_failures++;
            }
        }
        _vg_begin++;
    }

    return _vg_failures;
}

#elif defined(__clang__) || defined(__GNUC__)

#include <stdio.h>
#include <string.h>

extern struct VangoTestFunc __start_vgtest[];
extern struct VangoTestFunc __stop_vgtest[];

int main(int argc, char** argv) {
    int _vg_failures = 0;

    for (struct VangoTestFunc* _vg_f = __start_vgtest; _vg_f != __stop_vgtest; ++_vg_f) {
        int _vg_run_this = argc == 1 ? 1 : 0;
        if (argc > 1) {
            for (int i = 1; i < argc; i++) {
                if (strcmp(_vg_f->id, argv[i]) == 0) {
                    _vg_run_this = 1;
                    break;
                }
            }
        }
        if (_vg_run_this != 0) {
            struct VangoTestResult _vango_test_result = { .msg = NULL, .failline=0 };
            (_vg_f->fn)(&_vango_test_result);
            if (_vango_test_result.msg == NULL) {
                fprintf(stderr, "\033[32m[VanGo:  info] passed '%s'\033[m\n", _vg_f->id);
            } else {
                fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed '%s' on line %u: \033[m%s\n",
                    _vg_f->id, _vango_test_result.failline, _vango_test_result.msg);
                _vg_failures++;
            }
        }
    }

    return _vg_failures;
}

#else
#error compiler does not support automated testing
#endif

#endif

#endif
