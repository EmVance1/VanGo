# VanGo - A C/C++ Build System for Cargo Lovers

This app is a build system designed with rusts cargo philosophy in mind. You can have a million options, but there is nothing wrong with sensible defaults. Use JSON to define a minimal build script, for example:
```json
{
    "project": "example",
    "lang": "C++20",
    "dependencies": []
}
```
The above configuration is already the minimum requirement. `./src` is assumed as the main source file directory (what the hell else are you putting there?) and added to the include path. `./bin` holds any incremental build files (usually object files).

The system supports most popular toolchains, specifically: GNU and Clang/LLVM on all platforms, as well as MSVC on windows. It does of course assume that you have all relevant compiler tools installed, as it is not in itself a compiler. For easier cross compilation, vango also supports zig as a target, which wraps clang. To read why this is useful, see chapter on [cross-compilation](#Cross-Compilation).

### Features supported so far
- New, Build, Run, Test, and Clean actions
- Specify header-only and binary libraries with a lib.json, supports custom profiles...
```json
{
    "library": "SFML",
    "lang" : "C++11",
    "include": "include",
    "profile": {
        ...
    }
},
```
- ...and plug and play at will in main project. Libraries can be placed in `./lib` or specified otherwise
```json
"dependencies": [ "SFML", "../SFUtils" ],
```
- Incremental building based on recent file changes
- Source code dependencies are automatically built recursively in the case of updates
- Preprocessor definitions
```json
"defines": [ "MACRO" ],
"defines": [ "VALUE=10" ],
```
- Precompiling a header was never easier
```json
"pch": "pch.h",
```
- Modify Debug and Release configurations, or add your own
```json
"profile": {
    "debug": { ... },
    "release": { ... },
    "minsizerel": { "inherits": "release", ... }
}
```
- Cross compilation via Clang/Zig

### It just works
Even without boilerplate generation, slap a `build.json` next to a `src` directory with a `main.cpp` in it and everything will just work. Was that so hard C++ standards commitee?

## How-to:
Some examples of invocations are as follows, but for a more complete list see the help action.

- `vango new     [--lib] [--c] <name>`
- `vango b[uild] [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>]`
- `vango r[un]   [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>] [-- args*]`
- `vango c[lean]`
- `vango help    [action]`

VanGo is opinionated for simplicity and makes some base assumptions and decisions:
- You have a valid build script in the project root (`build.json`)
- All of your source files are in the `src` directory, and all output files are generated in in `bin/{profile}/`.
- Your output binary is named the same as your project.
- To build as a library, you have a `lib.h` somewhere in your project.
- All platforms have a compiler toolchain they default to - MSVC on windows, GCC on linux, Clang on macos - this can be overridden using the -t switch on build, run, and test commands. The `-t=msvc` option is provided for completeness, despite the tool being unavailable on non-windows platforms. To change your system default toolchain, set the environment variable `VANGO_DEFAULT_TOOLCHAIN` to one of the four valid values.

### build.json
All `build.json` files are expected to have 3 base declarations at the root:
```json
{
    "project": "foobar"
    "lang": "C++XX",
    "dependencies": [ ... ],
    ...
}
```
- `project` is an arbitrary string that defines how your project is viewed in the builder. This is for example the name the builder will look for when resolving source dependencies (see later).

- `lang` takes any valid C or C++ standard, case insensitive.

- `dependencies` is the main workhorse of the build system. It takes 0 or more strings representing libraries also supported by VanGo. If no path to the library is specified, VanGo will search in '~/.vango/packages/'. A dependency must have a definition in its root directory. This may either be a `build.json` for source, or a `lib.json` for binary or header only libraries. Source libraries will be automatically built recursively by any project that includes them.

    There is currently basic support for git dependencies by specifying the full URL. The repo is cached in '~/.vango/packages/', and is otherwise treated just like any other dependency (must contain a build script, etc.).

    As it stands, there are plans for a very basic package manager, more a simple registry of URLs of popular libraries and corresponding build scripts, but this is a ways away for now.

- **Profiles**: to customize build profiles or define your own that inherites one of the builtins, you can define the `profile` object. All of the following options (except `inherits`) can be defined globally (same level as `project`) as a default, or inside a subobject of `profile`.

- Preprocessor definitions can be loaded through the optional `defines` array. By default, this array will contain `VANGO_DEBUG` or `VANGO_RELEASE` definitions, aswell as `VANGO_TEST` for test builds.

- If you want to precompile a header, just specify the header file relative to `src/` that you want precompiled as shown above (All source files will be assumed to use it).

- Source directory and (project) include directories are assumed to be `./src` and `[ ./src ]` respectively, however they can be overridden or appended to through the `src` and `include` options.

- If the project you are defining is going to be a library, you may want to add an `include-pub` field. This is a string that tells dependency resolution that this directory should be used as the public interface (as opposed to `src` by default).

- For finer control, the option is provided to pass compiler and linker flags directly, using the `compiler-options` and `linker-options` array fields. These are prepended to the arguments generated by vango. In the near future, this system is being phased out in favour of a toolchain agnostic variant.

- For a given project, you can make a platform specific build definition by naming the file 'win.build.json', 'lnx.build.json', or 'mac.build.json'. The same applies to the lib definitions in the following chapter.

- Custom profile definitions require a base of settings to build upon, which is declared with the `inherits` field.

### lib.json
A `lib.json` file specifies for prebuilt libraries how they should be correctly linked. It must contain:
```json
{
    "library": "foobar",
    "lang": "C++XX",
    ...
}
```
`lang` in this case declares compatibility. Dependency resolution will error on any library that requires a newer C++ standard than the project linking it. In the case of mixing C and C++, the builder assumes all C to be C++ compatible for ease of use, but the user must ensure that this is in fact the case (i.e. that header files use 'clean' C).

In addition, libraries may have a `profile` table. Like their `build.json` counterparts, all profile options (except `inherits`) may be specified globally as a default. Libraries support the following profile options:

- `include` is a string that declares where the libraries header files are.

- `libdir` is a string that declares where the library binaries are.

- The `binaries` is a list of the binaries that the library provides. These are specified in name only (no file extension, no 'lib' prefix for .a files).

- The `defines` array lists preprocessor definitions.

- Custom profile definitions require a base of settings to build upon, which is declared with the `inherits` field.

In the case of header only libraries, most of these can be ignored in favour of a globally default `include` field.

### Automated Testing
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

To forward declare a test, use the `decl_test(test_name)` macro.
In one file and one file only, the include statement must be preceded by the `VANGO_TEST_ROOT` definition. This ensures no ODR violations for implementation functions, and additionally in C++ enables some behind the scenes magic to perform automatic test detection and main function generation.
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
Given these prerequisites, tests can be run on a case by case basis by specifying their names on the command line,by specifying their names on the command line, or all at once by not specifying anything.


### Cross-Compilation
If you're familiar with the Clang/LLVM toolchain, you already know that these tools support cross-compilation out of the box. If you don't need these features or you're used to the clang cross workflow, then plain clang is a fine way to go, specifying the `--target` and `--sysroot` options directly via the `*-options` fields whenever necessary. However, one headache this can often cause is that clang does not bundle in the default libraries for the targets it compiles to, and these can be non-trivial to set up, depending on the OS you want to target. Luckily, the brilliant developers of zig have solved this problem for us.

As referenced earlier, vango supports the usage of zig as a compilation toolchain (not for the zig language itself unfortunately). This works because zig includes clang as part of its ecosystem, however, the main benefit of using zigs wrappers vs clang, is that zig *does* ship with system libraries for many many platforms. This means that if you have zig on your system, no messing around with `sysroot`s is necessary. In fact, you do not need to so much as touch the target platform until you ship. All that's required is to specify the target triple like so:
```json
    "compiler-options": [ "-target", "<machine>-<os>-<abi>" ],
    "linker-options": [ "-target", "<machine>-<os>-<abi>" ],
```
and the correct binary will be generated.

