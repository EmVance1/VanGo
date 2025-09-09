#pragma once
#include <iostream>
#include <sstream>
#include <exception>
#include <vector>


namespace vango {

enum class FailType {
    VANGO_FAIL_TRUE,
    VANGO_FAIL_EQ,
    VANGO_FAIL_NE,
    VANGO_FAIL_NULL,
    VANGO_FAIL_NON_NULL,
    VANGO_FAIL_THROWS,
};

class AssertionFail : public std::exception {
public:
    std::string msg;
    FailType failtype;
    uint32_t failline;

public:
    AssertionFail(const std::string& _msg, FailType _failtype, uint32_t _failline)
        : msg(_msg), failtype(_failtype), failline(_failline)
    {}

    const char* what() const noexcept {
       return msg.c_str();
    }
};

typedef void(*TestFunc)();

}


#define VANGO_TEST_OUTPUT std::stringstream _vango_test_assert_output; _vango_test_assert_output
#define VANGO_TEST_THROW(type) throw ::vango::AssertionFail(_vango_test_assert_output.str(), ::vango::FailType::type, __LINE__)

#define vg_assert(a)           do { if (!(a))       { VANGO_TEST_OUTPUT << "assertion fail: expected 'true', received 'false'"; \
    VANGO_TEST_THROW(VANGO_FAIL_TRUE); } } while (0)

#define vg_assert_eq(a, b)     do { if ((a) != (b)) { VANGO_TEST_OUTPUT << "assertion fail: '" << a << "' != '" << b << "'"; \
    VANGO_TEST_THROW(VANGO_FAIL_EQ); } } while (0)

#define vg_assert_ne(a, b)     do { if ((a) == (b)) { VANGO_TEST_OUTPUT << "assertion fail: '" << a << "' == '" << b << "'"; \
    VANGO_TEST_THROW(VANGO_FAIL_NE); } } while (0)

#define vg_assert_null(a)      do { if (a)         { VANGO_TEST_OUTPUT << "assertion fail: expected 'nullptr', received valid pointer"; \
    VANGO_TEST_THROW(VANGO_FAIL_NULL); } } while (0)

#define vg_assert_non_null(a)  do { if (!(a))      { VANGO_TEST_OUTPUT << "assertion fail: expected valid pointer, received 'nullptr'"; \
    VANGO_TEST_THROW(VANGO_FAIL_NON_NULL); } } while (0)

#define vg_assert_throws(a, e) do { try { a; \
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it did not";              VANGO_TEST_THROW(VANGO_FAIL_THROWS); \
    } catch (const e&) {} catch (...) { \
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it threw something else"; VANGO_TEST_THROW(VANGO_FAIL_THROWS); \
    } } while (0)


#if defined(_MSC_VER)
#define VANGO_SECTION_TESTS __declspec(allocate("vango_tests"))
#pragma section("vango_tests", read, write)
#elif defined(__clang__) || defined(__GNUC__)
#define VANGO_SECTION_TESTS __attribute__((used, section("vango_tests")))
#else
#error Unsupported compiler
#endif

#define VANGO_REGISTER_TEST(fn) \ static TestFunc __test_##fn VANGO_SECTION_TESTS = fn

#define test(name) void name(); VANGO_REGISTER_TEST(name); void name()


#ifdef VANGO_TEST_ROOT

#define run_test(k, f) try { \
        f(); \
        std::cerr << "\033[32m[VanGo:  info] passed: '" << k << "'\033[m\n"; \
    } catch (const ::vango::AssertionFail& e) { \
        std::cerr << "\033[32m[VanGo:  info] \033[31mfailed: '" << k << "' on line " << e.failline << ": \033[m" << e.msg << "\n"; \
    }


#if defined(_MSC_VER)
extern "C" __declspec(selectany) vango::TestFunc* __start_vango_tests;
extern "C" __declspec(selectany) vango::TestFunc* __stop_vango_tests;
#elif defined(__clang__) || defined(__GNUC__)
extern vango::TestFunc __start_vango_tests[];
extern vango::TestFunc __stop_vango_tests[];
#else
#error Unsupported compiler
#endif


int main(int argc, char** argv) {
    for (vango::TestFunc* t = __start_vango_tests; t != __stop_vango_tests; t++) {
        (*t)();
    }
}

#endif
