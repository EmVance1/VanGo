#include <cstdlib>
#include <iostream>
#include <sstream>
#include <exception>
#include <Windows.h>


struct AssertionFail: public std::exception {
    std::string msg;
    uint32_t failtype;
    uint32_t failline;

    explicit AssertionFail(const std::string& _msg, uint32_t _failtype, uint32_t _failline) : msg(_msg), failtype(_failtype), failline(_failline) {}

    virtual const char* what() const noexcept {
       return msg.c_str();
    }
};


#define FAIL_TRUE 1
#define FAIL_EQ 2
#define FAIL_NE 3
#define FAIL_NULL 4
#define FAIL_NON_NULL 5
#define FAIL_THROWS 6

#define ASSERT_OUT std::stringstream _assert_output; _assert_output

#define assert(a)           if (!a)     { ASSERT_OUT << "assertion fail: expected 'true', received 'false'";                  \
    throw AssertionFail(_assert_output.str(), FAIL_TRUE, __LINE__); }

#define assert_eq(a, b)     if (a != b) { ASSERT_OUT << "assertion fail: expected '" << a << "', received '" << b << "'";     \
    throw AssertionFail(_assert_output.str(), FAIL_EQ, __LINE__); }

#define assert_ne(a, b)     if (a == b) { ASSERT_OUT << "assertion fail: expected not '" << a << "', received '" << b << "'"; \
    throw AssertionFail(_assert_output.str(), FAIL_NE, __LINE__); }

#define assert_null(a)      if (!a)     { ASSERT_OUT << "assertion fail: expected 'nullptr', received valid pointer";         \
    throw AssertionFail(_assert_output.str(), FAIL_NULL, __LINE__); }

#define assert_non_null(a)  if (a)      { ASSERT_OUT << "assertion fail: expected valid pointer, received 'nullptr'";         \
    throw AssertionFail(_assert_output.str(), FAIL_NON_NULL, __LINE__); }

#define assert_throws(a, e) { bool _throw_fail = false; std::stringstream _assert_output; \
    try { \
        a; \
        _throw_fail = true; \
        _assert_output << "assertion fail: expected '" #a "' to throw '" #e "' but it did not"; \
    } catch (const e&) {} catch (...) { \
        _assert_output << "assertion fail: expected '" #a "' to throw '" #e "' but it threw something else"; \
        throw AssertionFail(_assert_output.str(), FAIL_THROWS, __LINE__); \
    } if (_throw_fail) { \
        throw AssertionFail(_assert_output.str(), FAIL_THROWS, __LINE__); \
    } }


#define TERMINAL_RED 4
#define TERMINAL_GREEN 2
#define TERMINAL_WHITE 7


#define test(f) { \
    try { \
        f(); \
        HANDLE hConsole = GetStdHandle(STD_ERROR_HANDLE); \
        SetConsoleTextAttribute(hConsole, TERMINAL_GREEN); \
        std::cerr << "[mscmp:  test] test '" << #f << "' passed\n"; \
        SetConsoleTextAttribute(hConsole, TERMINAL_WHITE); \
    } catch (const AssertionFail& e) { \
        HANDLE hConsole = GetStdHandle(STD_ERROR_HANDLE); \
        SetConsoleTextAttribute(hConsole, TERMINAL_RED); \
        std::cerr << "[mscmp:  test] test '" << #f << "' failed on line " << e.failline << ": "; \
        SetConsoleTextAttribute(hConsole, TERMINAL_WHITE); \
        std::cerr << e.what() << "\n"; \
    } }

