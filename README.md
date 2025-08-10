# VanGo - A C/C++ Build System for Cargo Lovers

This app is a build system designed with rusts cargo philosophy in mind. You can have a million options, but there is nothing wrong with sensible defaults. Use JSON to define a minimal build script, for example:
```json
{
    "project": "example",
    "cpp": "C++20",
    "dependencies": []
}
```
The above configuration is already the minimum requirement. `./src` is assumed as the main source file directory (what the hell else are you putting there?) and added to the include path. `./bin` holds any incremental build files (usually object files).

The system supports most popular toolchains, specifically: MSVC and MinGW on windows, GNU on Linux, and Clang/LLVM on MacOS. It does of course assume that you have all relevant compiler tools installed. It is not in itself a compiler.

### Features supported so far
- New, Build, Run, Test, and Clean actions
- Specify header-only and binary libraries with a lib.json, supports multiple configurations...
```json
{
    "library": "SFML",
    "minstd" : "C++11",
    "include": "include/",
    "configs": {
        ...
    }
},
```
- ...and plug and play at will in main project. Libraries can be placed in `./lib` or specified otherwise
```json
"dependencies": [ "SFML:static", "../SFUtils" ],
```
- Incremental building based on recent file changes
- Source code dependencies are automatically built recursively in the case of updates
- Preprocessor definitions
```json
"defines": [ "MACRO" ],
"defines": [ "VALUE=10" ],
```
- Precompiled headers were never easier
```json
"pch": "pch.h",
```
- Executable/Library output deduction using main.cpp/lib.h entry points
- Debug and Release configurations (work in progress)
```json
"SETTING.debug": { ... },
"SETTING.release": { ... },
```

### It just works
Slap a `build.json` next to a `src` directory with a `main.cpp` in it and everything will just work. Was that so hard C++ standards commitee?

## How-to:
The build system is invoked like so:

- `vango n[ew]   [-lib] [-c] name`
- `vango b[uild] [-r[elease]] [-mingw]`
- `vango r[un]   [-r[elease]] [-mingw] [-- args*]`
- `vango t[est]  [-r[elease]] [-mingw] [tests*]`
- `vango c[lean]`

VanGo is opinionated for simplicity and makes some base assumptions: you have a valid build script in the project root (`build.json`), all of your source files are in the `src` directory, and it will place all output files in `bin/{config}/`. Your output executable is named the same as your project. In the `run` action, all extraneous arguments are passed to the invoked executable.

All platforms have a compiler toolchain they default to, that being MSVC on windows. To use MinGW GCC instead, you can just pass `-mingw` to the build, run or test commands.

### How-to: build.json
All `build.json` files are expected to have 3 base declarations at the root:
```json
{
    "project": "foobar"
    "cpp": "C++XX",
    "dependencies": [ ... ],
    ...
}
```
`project` is an arbitrary string that defines how your project is viewed in the builder. This is for example the name the builder will look for when resolving source dependencies (see later). `cpp` takes any valid C++ standard, prefixed by `"C++"` (case insensitive). It also takes `"CXX"` if you want to build pure C projects.


`dependences` is the main workhorse of the build system. It takes 0 or more strings representing libraries also supported by VanGo. If no path to the library is specified, VanGo will search in `./lib`. The dependency string also supports an optional version, separated by a ':' (see chapter on library version definitions) as in `SFML:static`. A dependency must have a definition in its root directory. This may either be a `build.json` for source, or a `lib.json` for binary or header only libraries. Source libraries will be automatically built recursively by any project that includes them.

There is currently basic support for git dependencies by specifying the full URL. The repo is cached in '~/.vango/packages/', and is otherwise treated just like any other dependency (must contain a build script, etc.).

As it stands, there are plans for a very basic package manager, more a simple registry of URLs of popular libraries and corresponding build scripts, but this is a ways away for now.


Preprocessor definitions can be loaded through the optional `defines` array. By default, this array will contain `DEBUG` or `RELEASE` definitions, aswell as `TEST` for test builds.

If you want to precompile a header, just specify the header file at the root of `src/` that you want precompiled as shown above (All source files will be assumed to use it).

Source directory and (project) include directories are assumed to be `./src` and `[ ./src ]` respectively, however they can be overridden or appended to through the `srcdir` and `incdirs` options.

If the project you are defining is going to be a library, you may want to add an `include-public` field. This is a string that tells dependency resolution that this directory should be used as the public interface (as opposed to `src` by default).

For finer control, the option is provided to pass compiler and linker flags directly, using the `compiler-options` and `linker-options` array fields.

For a given project, you can make a platform specific build definition by naming the file "win.build.json", "linux.build.json", or "macos.build.json". The same applies to the lib definitions in the following chapter.

### How-to: lib.json
A `lib.json` file specifies for prebuilt libraries how they should be correctly linked. It must contain:
```json
{
    "library": "foobar",
    "minstd": "C++XX",
    "include": "include/"
    ..
}
```
`minstd` declares compatibility. Dependency resolution will error on any library that requires a newer C++ standard than the project linking it. In the case of mixing C and C++, the builder assumes all C to be C++ compatible for ease of use, but the user must ensure that this is in fact the case (i.e. no designated initializers in header files etc.).

In addition, all libraries must have one of the following (but not both):
```json
    "all": { ... },
    "configs": { "name": { ... }, ... },
```

Configs represent different ways of linking a given library, for example if a library supports both static and dynamic linking. It is defined by 3 required fields
```json
{
    "links": [ ... ],
    "binary.debug": "foo/",
    "binary.release": "bar/"
}
```
as well as an optional field for version specific preprocessor flags
```json
    "defines": [ "MACRO" ]
```
The `all` field represents a standard configuration if versions are not necessary for a project.

### How-to: automated testing
Testing is made easy by assuming all tests are in a `test/` directory in the project root. A test project is a C/C++ project of arbitrary complexity, and may look like the following:
```cpp
#define TEST_ROOT
#include <vango/asserts.h>

test(basic_math) {
    int a = 2;
    a += 3;
    a *= 2;

    assert_eq(a, 10);
}
```
In order to write tests, the header 'vangotest/asserts.h' or 'vangotest/casserts.h' must be included. The files are automatically in the include path for test configurations. As the name suggests, these contain basic assert macros that report back the success status of the test, however some things are of note:

To forward declare a test, use the macro `decl_test(test_name)`.
In one file and one file only, the include statement must be preceded by the `TEST_ROOT` definition. This ensures no ODR violations for implementation functions, and additionally in C++ enables some behind the scenes magic to perform automatic test detection and main function generation.
In C however, some automation features are unavailable, and in addition to the code seen above, you must register your tests like so:
```cpp
#define TEST_ROOT
#include <vango/casserts.h>

test(basic_math) {
    int a = 10;
    assert_eq(a, 10);
}

test_main(
    test_register(basic_math);
)
```
Given these prerequisites, tests can be run on a case by case basis or all at once by not specifying specific tests.

