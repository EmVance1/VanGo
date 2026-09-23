#pragma once
#include <sstream>
#include <exception>
#include <vector>


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


#define vango_test(name) void name(); ::vango::TestFuncArray* _##name##_runner = ::vango::init_testfunc( #name, name, false ); void name()
#define vango_test_decl(name) void name()


#ifdef VANGO_TEST_ROOT

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
    int _vg_failures = 0;

    if (argc == 1) {
        for (size_t i = 0; i < arr->names.size(); i++) {
            try {
                (arr->funcs[i])();
                fprintf(stderr, "\033[32m[VanGo:  info] passed: '%s'\033[m\n", arr->names[i]);
            } catch (const ::vango::AssertionFail& e) {
                fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed: '%s' on line %d: \033[m%s\n", arr->names[i], e.failline, e.msg.c_str());
                _vg_failures++;
            } catch (const ::std::exception& e) {
                fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed: '%s' threw: \033[m%s\n", arr->names[i], e.what());
                _vg_failures++;
            }
        }
    } else {
        for (int j = 1; j < argc; j++) {
            for (size_t i = 0; i < arr->names.size(); i++) {
                if (strcmp(arr->names[i], argv[j]) == 0) {
                    try {
                        (arr->funcs[i])();
                        fprintf(stderr, "\033[32m[VanGo:  info] passed: '%s'\033[m\n", arr->names[i]);
                    } catch (const ::vango::AssertionFail& e) {
                        fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed: '%s' on line %d: \033[m%s\n", arr->names[i], e.failline, e.msg.c_str());
                        _vg_failures++;
                    } catch (const ::std::exception& e) {
                        fprintf(stderr, "\033[32m[VanGo:  info] \033[31mfailed: '%s' threw: \033[m%s\n", arr->names[i], e.what());
                        _vg_failures++;
                    }
                }
            }
        }
    }

    return _vg_failures;
}

#endif

