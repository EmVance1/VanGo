#pragma once
#include <iostream>
#include <sstream>
#include <exception>


namespace vango {

class AssertionFail : public std::exception {
public:
    std::string msg;
    uint32_t failline;

public:
    AssertionFail(const std::string& _msg, uint32_t _failline)
        : msg(_msg), failline(_failline)
    {}

    const char* what() const noexcept {
       return msg.c_str();
    }
};

typedef void(*TestFunc)();

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
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it did not";              VANGO_TEST_THROW(VANGO_FAIL_THROWS); \
    } catch (const e&) {} catch (...) { \
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it threw something else"; VANGO_TEST_THROW(VANGO_FAIL_THROWS); \
    } } while (0)


#if defined(_MSC_VER)
#pragma section("vgtest$a", read, write)
#pragma section("vgtest$v", read, write)
#pragma section("vgtest$z", read, write)
#define VANGO_SECTION_TESTS __declspec(allocate("vgtest$v"))
#elif defined(__clang__) || defined(__GNUC__)
#define VANGO_SECTION_TESTS __attribute__((used, section("vgtest")))
#else
#error Unsupported compiler
#endif

#define vango_test(name) void name(); __declspec(allocate("vgtest$v")) ::vango::TestFunc _vango_test_##name = name; void name()


#ifdef VANGO_TEST_ROOT

#define run_test(k, f) try { \
        (f)(); \
        std::cerr << "\033[32m[VanGo:  info] passed: '" << k << "'\033[m\n"; \
    } catch (const ::vango::AssertionFail& e) { \
        std::cerr << "\033[32m[VanGo:  info] \033[31mfailed: '" << k << "' on line " << e.failline << ": \033[m" << e.msg << "\n"; \
    }


#if defined(_MSC_VER)

__declspec(allocate("vgtest$a")) uint8_t _start_vango_tests = 0;
__declspec(allocate("vgtest$z")) uint8_t _stop_vango_tests = 0;

int main(int argc, char** argv) {
    ::vango::TestFunc* _vg_start = (vango::TestFunc*)(&_start_vango_tests);
    ::vango::TestFunc* _vg_stop = (vango::TestFunc*)(&_stop_vango_tests);

    while (_vg_start++ < _vg_stop) {
        if (*_vg_start == 0) {  continue; }
        run_test("abc", *_vg_start);
        _vg_start++;
    }
}

#elif defined(__clang__) || defined(__GNUC__)

extern vango::TestFunc __start_vgtest[];
extern vango::TestFunc __stop_vgtest[];
int main(int argc, char** argv) {
    for (vango::TestFunc* t = __start_vango_tests; t != __stop_vango_tests; t++) {
        (*t)();
    }
}

#else
#error Unsupported compiler
#endif

#endif
