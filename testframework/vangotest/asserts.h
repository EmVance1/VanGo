#pragma once
#include <iostream>
#include <sstream>
#include <exception>
#include <vector>


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

struct TestFuncArray {
    std::vector<const char*> names;
    std::vector<TestFunc> funcs;
};

TestFuncArray* init_testfunc(const char* name, TestFunc func, bool noassign);

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


#define test(name) void name(); ::vango::TestFuncArray* _##name##_runner = ::vango::init_testfunc( #name, name, false ); void name()
#define decl_test(name) void name()


#ifdef VANGO_TEST_ROOT

#define vg_run_test(k, f) try { \
        f(); \
        std::cerr << "\033[32m[VanGo:  info] passed: '" << k << "'\033[m\n"; \
    } catch (const ::vango::AssertionFail& e) { \
        std::cerr << "\033[32m[VanGo:  info] \033[31mfailed: '" << k << "' on line " << e.failline << ": \033[m" << e.msg << "\n"; \
    }

namespace vango {

TestFuncArray* init_testfunc(const char* name, TestFunc func, bool noassign) {
    static TestFuncArray testfuncarray;
    if (!noassign) {
        testfuncarray.names.push_back(name);
        testfuncarray.funcs.push_back(func);
    }
    return &testfuncarray;
}

}

int main(int argc, char** argv) {
    ::vango::TestFuncArray* arr = ::vango::init_testfunc(nullptr, nullptr, true);
    if (argc == 1) {
        for (size_t i = 0; i < arr->names.size(); i++) {
            vg_run_test(arr->names[i], arr->funcs[i]);
        }
    } else {
        for (int j = 1; j < argc; j++) {
            for (size_t i = 0; i < arr->names.size(); i++) {
                if (strcmp(arr->names[i], argv[j]) == 0) {
                    run_test(arr->names[i], arr->funcs[i]);
                }
            }
        }
    }
}

#endif

