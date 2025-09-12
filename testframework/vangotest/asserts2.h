#pragma once
#include <sstream>
#include <exception>


namespace vango {

class AssertionFail : public std::exception {
public:
    std::string msg;
    unsigned int failline;

public:
    AssertionFail(const std::string& _msg, unsigned int _failline)
        : msg(_msg), failline(_failline)
    {}

    const char* what() const noexcept {
       return msg.c_str();
    }
};

typedef void(*TestFuncImpl)();

struct TestFunc {
    const char* id;
    TestFuncImpl fn;
};

}


#define VANGO_TEST_OUTPUT std::stringstream _vango_test_assert_output; _vango_test_assert_output
#define VANGO_TEST_THROW() throw ::vango::AssertionFail(_vango_test_assert_output.str(), __LINE__)

#define vg_assert(a)           do { if (!(a))       { VANGO_TEST_OUTPUT << "assertion fail: expected 'true', received 'false'"; \
    VANGO_TEST_THROW(); } } while (0)

#define vg_assert_eq(a, b)     do { if ((a) != (b)) { VANGO_TEST_OUTPUT << "assertion fail: '" << a << "' != '" << b << "'"; \
    VANGO_TEST_THROW(); } } while (0)

#define vg_assert_ne(a, b)     do { if ((a) == (b)) { VANGO_TEST_OUTPUT << "assertion fail: '" << a << "' == '" << b << "'"; \
    VANGO_TEST_THROW(); } } while (0)

#define vg_assert_null(a)      do { if (a)         { VANGO_TEST_OUTPUT << "assertion fail: expected 'nullptr', received valid pointer"; \
    VANGO_TEST_THROW(); } } while (0)

#define vg_assert_non_null(a)  do { if (!(a))      { VANGO_TEST_OUTPUT << "assertion fail: expected valid pointer, received 'nullptr'"; \
    VANGO_TEST_THROW(); } } while (0)

#define vg_assert_throws(a, e) do { try { a; \
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it did not";              VANGO_TEST_THROW(); \
    } catch (const e&) {} catch (...) { \
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it threw something else"; VANGO_TEST_THROW(); \
    } } while (0)


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
    void name(); \
    VANGO_SECTION_TESTS ::vango::TestFunc _vango_test_##name = { #name, &name }; \
    void name()


#ifdef VANGO_TEST_ROOT

#include <cstdio>
#include <cstring>

#if defined(_MSC_VER)

__declspec(allocate("vgtest$a")) ::vango::TestFunc _start_vgtest = {};
__declspec(allocate("vgtest$z")) ::vango::TestFunc _stop_vgtest = {};

int main(int argc, char** argv) {
    char** _vg_begin = (char**)(&_start_vgtest+1);
    char** _vg_end = (char**)&_stop_vgtest;

    int _vg_failures = 0;

    for (; _vg_begin < _vg_end; _vg_begin++) {
        if (*_vg_begin == 0) { continue; }
        vango::TestFunc* _vg_f = (vango::TestFunc*)_vg_begin;
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
            try {
                (_vg_f->fn)();
                fprintf(stderr, "\033[32m[VanGo:  info] passed: '%s'\033[m\n", _vg_f->id);
            } catch (const ::vango::AssertionFail& e) {
                fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed: '%s' on line %u: \033[m%s\n", _vg_f->id, e.failline, e.msg.c_str());
                _vg_failures++;
            }
            _vg_begin++;
        }
    }

    return _vg_failures;
}

#elif defined(__clang__) || defined(__GNUC__)

extern ::vango::TestFunc __start_vgtest[];
extern ::vango::TestFunc __stop_vgtest[];

int main(int argc, char** argv) {
    int _vg_failures = 0;

    for (::vango::TestFunc* _vg_f = __start_vgtest; _vg_f != __stop_vgtest; ++_vg_f) {
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
            try {
                (_vg_f->fn)();
                fprintf(stderr, "\033[32m[VanGo:  info] passed: '%s'\033[m\n", _vg_f->id);
            } catch (const ::vango::AssertionFail& e) {
                fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed: '%s' on line %u: \033[m%s\n", _vg_f->id, e.failline, e.msg.c_str());
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
