# Automated Testing
Vango supports automated testing for library projects. To benefit from this, its best to modularize your core functionality into a library, which is then driven by a separate binary project (this is generally considered good practice in any framework). Test projects are arbitrarily complex C/C++ projects, the source code for which you place in the `test` directory in the project root.

In order to write tests, the header `vangotest/asserts2.h`- `vangotest/casserts2.h` for C - must be included. These are automatically visible for test configurations. As the name suggests, these contain basic assert macros that report back the success status of the test. In one file and one file only, the include statement must be preceded by the `VANGO_TEST_ROOT` definition. This enables automatic discovery of your tests, meaning you dont need to call or even forward declare your tests anywhere. A dummy test project might look like this:
```cpp
#define VANGO_TEST_ROOT
#include <vangotest/asserts2.h>

vango_test(basic_math) {
    int a = 2;
    a += 3;
    a *= 2;

    vg_assert_eq(a, 10);
}
```
As you can see, a test is essentially a pure void function. Tests can be run all at once, or on a case by case basis by specifying the test names on the command line.

### Important Note
The '*2.h' family of assert headers is currently experimental on MSVC (including clang-msvc), due to some awkard pointer hacks it performs to make automatic discovery work. If MSVC users prefer, the old, more stable headers are still available ('asserts.h', 'casserts.h'). These behave identically for C++, albeit with some ordering quirks. In C however, some automation features are unavailable, and in addition to the code seen above, you must forward declare and include your tests into the test root, and register them like so:
```c
// basic.h ====================
#ifndef BASIC_H
#define BASIC_H
#include <vangotest/casserts.h>

vango_test_decl(basic_math)

#endif

// basic.c ====================
#include "basic.h"

vango_test(basic_math) {
    int a = 10;
    vg_assert_eq(a, 10);
}

// test.c =====================
#define VANGO_TEST_ROOT
#include <vangotest/casserts.h>
#include "basic.h"

vango_test_main(
    vango_test_reg(basic_math);
)
```
