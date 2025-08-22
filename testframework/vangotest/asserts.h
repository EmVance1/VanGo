#pragma once
#include <cstdlib>
#include <iostream>
#include <sstream>
#include <exception>


class AssertionFail : public std::exception {
public:
    std::string msg;
    uint32_t failtype;
    uint32_t failline;

public:
    AssertionFail(const std::string& _msg, uint32_t _failtype, uint32_t _failline)
        : msg(_msg), failtype(_failtype), failline(_failline)
    {}

    const char* what() const noexcept {
       return msg.c_str();
    }
};


#define VANGO_TEST_OUTPUT std::stringstream _test_assert_output; _test_assert_output
#define VANGO_TEST_THROW(type) throw AssertionFail(_test_assert_output.str(), type, __LINE__)

#define FAIL_TRUE 1
#define FAIL_EQ 2
#define FAIL_NE 3
#define FAIL_NULL 4
#define FAIL_NON_NULL 5
#define FAIL_THROWS 6

#define assert(a)           if (!(a))       { VANGO_TEST_OUTPUT << "assertion fail: expected 'true', received 'false'";                  \
    VANGO_TEST_THROW(FAIL_TRUE); }

#define assert_eq(a, b)     if ((a) != (b)) { VANGO_TEST_OUTPUT << "assertion fail: expected '" << a << "', received '" << b << "'";     \
    VANGO_TEST_THROW(FAIL_EQ); }

#define assert_ne(a, b)     if ((a) == (b)) { VANGO_TEST_OUTPUT << "assertion fail: expected not '" << a << "', received '" << b << "'"; \
    VANGO_TEST_THROW(FAIL_NE); }

#define assert_null(a)      if (!(a))       { VANGO_TEST_OUTPUT << "assertion fail: expected 'nullptr', received valid pointer";         \
    VANGO_TEST_THROW(FAIL_NULL); }

#define assert_non_null(a)  if (a)          { VANGO_TEST_OUTPUT << "assertion fail: expected valid pointer, received 'nullptr'";         \
    VANGO_TEST_THROW(FAIL_NON_NULL); }

#define assert_throws(a, e) { try { a; \
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it did not";              VANGO_TEST_THROW(FAIL_THROWS); \
    } catch (const e&) {} catch (...) { \
        VANGO_TEST_OUTPUT << "assertion fail: expected '" #a "' to throw '" #e "' but it threw something else"; VANGO_TEST_THROW(FAIL_THROWS); \
    } }


typedef void(*TestFunc)();

typedef struct TestFuncArray {
    const char** names;
    TestFunc* funcs;
    size_t size;
    size_t cap;
} TestFuncArray;


#define test(name) void name(); TestFuncArray* _##name##_runner = init_testfunc( #name, name, false ); void name()
#define decl_test(name) void name()


TestFuncArray* init_testfunc(const char* name, TestFunc func, bool noassign);


#ifdef VANGO_TEST_ROOT

#define run_test(k, f) try { \
        f(); \
        std::cerr << "\033[32m[VanGo:  info] passed: '" << k << "'\033[m\n"; \
    } catch (const AssertionFail& e) { \
        std::cerr << "\033[32m[VanGo:  info] \033[31mfailed: '" << k << "' on line " << e.failline << ": \033[m" << e.msg << "\n"; \
    }

TestFuncArray init_testfuncarray(size_t size) {
    return TestFuncArray{
        (const char**)malloc(size * sizeof(char*)),
        (TestFunc*)malloc(size * sizeof(TestFunc*)),
        0,
        size
    };
}

TestFuncArray* init_testfunc(const char* name, TestFunc func, bool noassign) {
    static TestFuncArray testfuncarray = init_testfuncarray(128);
    if (!noassign) {
        testfuncarray.names[testfuncarray.size] = name;
        testfuncarray.funcs[testfuncarray.size] = func;
        testfuncarray.size++;
    }
    return &testfuncarray;
}

int main(int argc, char** argv) {
    TestFuncArray* arr = init_testfunc(nullptr, nullptr, true);
    if (argc == 1) {
        for (size_t i = 0; i < arr->size; i++) {
            run_test(arr->names[i], arr->funcs[i]);
        }
    } else {
        for (int j = 1; j < argc; j++) {
            for (size_t i = 0; i < arr->size; i++) {
                if (strcmp(arr->names[i], argv[j]) == 0) {
                    run_test(arr->names[i], arr->funcs[i]);
                }
            }
        }
    }
}

#endif

