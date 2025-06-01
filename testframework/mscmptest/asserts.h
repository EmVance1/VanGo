#include <cstdlib>
#include <iostream>
#include <sstream>
#include <exception>
#include <Windows.h>


struct AssertionFail: public std::exception {
    explicit AssertionFail(uint32_t _failtype, uint32_t _failline) : failtype(_failtype), failline(_failline) {}

    virtual const char* what() const noexcept {
       return "test failed";
    }

    uint32_t failtype;
    uint32_t failline;
};


#define FAIL_TRUE 1
#define FAIL_EQ 2
#define FAIL_NE 3
#define FAIL_NULL 4
#define FAIL_NON_NULL 5
#define FAIL_THROWS 6

#define assert(a)           if (!a)     { _test_assert_output << "assertion fail: expected 'true', received 'false'";                  \
    throw AssertionFail(FAIL_TRUE, __LINE__); }

#define assert_eq(a, b)     if (a != b) { _test_assert_output << "assertion fail: expected '" << a << "', received '" << b << "'";     \
    throw AssertionFail(FAIL_EQ, __LINE__); }

#define assert_ne(a, b)     if (a == b) { _test_assert_output << "assertion fail: expected not '" << a << "', received '" << b << "'"; \
    throw AssertionFail(FAIL_NE, __LINE__); }

#define assert_null(a)      if (!a)     { _test_assert_output << "assertion fail: expected 'nullptr', received valid pointer";         \
    throw AssertionFail(FAIL_NULL, __LINE__); }

#define assert_non_null(a)  if (a)      { _test_assert_output << "assertion fail: expected valid pointer, received 'nullptr'";         \
    throw AssertionFail(FAIL_NON_NULL, __LINE__); }

#define assert_throws(a, e) { bool _throw_fail = false; try { a; _throw_fail = true; \
    _test_assert_output << "assertion fail: expected '" #a "' to throw '" #e "' but it did not"; throw AssertionFail(FAIL_THROWS, __LINE__); } \
    catch (const e&) {} catch (...) { \
        _test_assert_output << "assertion fail: expected '" #a "' to throw '" #e "' but it threw something else"; throw AssertionFail(FAIL_THROWS, __LINE__); \
    } if (_throw_fail) { \
        throw AssertionFail(FAIL_THROWS, __LINE__); \
    } }


#define TERMINAL_RED 4
#define TERMINAL_GREEN 2


#define test(f) { \
    std::stringstream _test_assert_output; \
    try { \
        f(_test_assert_output); \
        std::cerr << "test '" << #f << "' passed\n"; \
    } catch (const AssertionFail& e) { \
        std::cerr << "test '" << #f << "' failed on line " << e.failline << ": " << _test_assert_output.str() << "\n"; \
    } }

#define TEST_PARAMS std::stringstream& _test_assert_output

