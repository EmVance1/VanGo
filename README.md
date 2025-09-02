# VanGo - A C/C++ Build System for Cargo Lovers

## Motivation

This app is a build system designed with rusts cargo philosophy in mind. You can have a million options, but there is nothing wrong with sensible defaults. Vango uses project file structure as a component of its project configuration, minimizing the need for a build script. `./src` is assumed as the main source file directory and added to the include path. `./bin` holds any incremental build files (usually object files). `./test` is for source files that contain tests. All manual configuration is done via the `Vango.toml` manifest file in the project root.

The system supports most popular toolchains, specifically: GNU and Clang/LLVM on all platforms, as well as MSVC on windows. It does of course assume that you have all relevant compiler tools installed, as it is not in itself a compiler. For easier cross compilation, vango also supports zig as a target, which wraps clang. To read why this is useful, see chapter on [cross-compilation](#Cross-Compilation).

## Features Available So Far
- Subcommands for creating, building, running, testing, and cleaning C/C++ projects
- File change detection and incremental rebuilds
- Source, static, and header-only  dependency automation
- Configure static libraries with a library-type toml file
```toml
[library]
package = "SFML"
version = "3.0.1"
lang = "C++17"
include = "include"
...
```
- Cross-platform precompiled headers
```toml
pch = "pch.h"
```
- Complete per-profile freedom of configuration
```toml
[profile.debug]
defines = [ "MACRO", "VALUE=10" ]

[profile.myprofile]
include = [ "src", "../some/other/headers" ]
```
- Cross compilation via Clang/Zig

**Conclusion**: It just works. Even without boilerplate generation via `vango new`, slap a `Vango.toml` next to a `src` directory with a `main.cpp` in it and everything will just work. Was that so hard everybody?

## How To:
Some examples of invocations are as follows, but for a more complete list see the help action.

- `vango new     [--lib] [--c] <name>`
- `vango b[uild] [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>]`
- `vango r[un]   [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>] [-- args*]`
- `vango c[lean]`
- `vango help    [action]`

VanGo is opinionated for simplicity and makes some base assumptions and decisions:
- You have a valid `Vango.toml` in the project root.
- All of your source files are in the `src` directory, and all output files are generated in `bin/{profile}/`.
- Your output binary is named the same as your project.
- All platforms have a compiler toolchain they default to - MSVC on windows, GCC on linux, Clang on macos - this can be overridden using the -t switch on build, run, and test commands. The `-t=msvc` option is provided for completeness, despite the tool being unavailable on non-windows platforms. To change your system default toolchain, set the environment variable `VANGO_DEFAULT_TOOLCHAIN` to one of the four valid values.

For a given project, you can make a platform specific build definition by naming the file 'win.vango.toml', 'lnx.vango.toml', or 'mac.vango.toml'.

A correct `Vango.toml` may begin with one of 2 sections - `[build]` and `[library]`.

### Build Configuration
All manifests that begin with `[build]` are expected to have 3 base declarations at the root:
```toml
[build]
package = "foobar"
version = "x.y.z"
lang = "C++XX"
# optional
kind = "app|staticlib|sharedlib"
interface = [ "CXX" ]
```
- `package` is an arbitrary string that defines how your project is viewed in the builder. This is for example the name the builder will look for when resolving source dependencies (see later).

- `version` takes a sem-ver number. At time of writing, this has no effect, but is worth maintaining nonetheless for clarity and for future use cases.

- `lang` takes any valid C or C++ standard, case insensitive.

- `kind` is for declaring whether your project builds to an executable or a library. The default value here is `"app"`, though it can be written explicitly. `staticlib` will produce a symbol archive file for your platform (.a, .lib, etc.). `sharedlib` as you might expect builds shared libraries, although these can very widely per platform. On linux this produces a .so file, while on windows it will produce a '.dll' binary and a '.lib' *import* library that is required for symbol loading. The macro `VANGO_EXPORT_SHARED` is also defined when building a dll file on windows, for all your `__declspec` needs.

**note**: Currently fully automatic linking for shared libraries is not quite functional on windows due to the way loading dlls works.

- `interface`: at times you may want to implement a library using one standard, but provide interfaces for use in another earlier standard, or even C. To partially bypass the compatibility checker, you can declare the `interface` array, which lists all standards your headers are compatible with. Elements of `interface` use the same format as `lang`.

- **dependencies**: The `dependencies` section is the main workhorse of the build system. Within it, you can list 0 or more named objects representing libraries also supported by VanGo. If no path to the library is specified, VanGo will search in '~/.vango/packages/'. A dependency that is not header-only must have a toml file in its root directory . Source libraries will be automatically built recursively by any project that includes them. Currently supported ways of specifying dependencies are as follows:
```toml
[dependencies]
MyLib     = { path="../MyLib" } # source, local, contains build toml-config
SFML      = { path="../SFML" }  # binary, local, contains static lib toml-config
SFUtils   = { git="https://github.com/EmVance1/ShimmyNav.git" } # source, remote, contains build toml-config
stb_image = { headers="lib/stb_image" } # headers, local, contains no config
```
Support for git dependencies is currently very basic. The repo is cached in '~/.vango/packages/', and is otherwise treated just like any other dependency (must contain a build script, etc.). For libraries that arent native to Vango, the ability to write automated build recipes (e.g. CMake invocation + toml injection) is coming soon.

As it stands, there are plans for a very basic package manager, more a simple registry of URLs of popular libraries and corresponding build recipes, but this is a ways away for now.

- **profile**: to customize build profiles or define your own that inherites one of the builtins, you can define the `profile.*` sections. All of the following options (except `inherits`) can be defined globally (under `[build]`) as a default, or under `[profile.debug]`, `[profile.release]`, or any `[profile.mycustomprofile]`.

- `defines`: additional preprocessor definitions. By default, this array will contain `VANGO_DEBUG` or `VANGO_RELEASE` definitions, aswell as `VANGO_TEST` for test builds.

- `pch`: if you want to precompile a header, just specify the header file relative to `src/` that you want precompiled as shown above (All source files will be assumed to use it).

- Source directory and (internal) include directories are assumed to be `./src` and `[ ./src ]` respectively, however they can be overridden or extended through the `src` and `include` options respectively.

- `include-pub`: if the project you are defining is going to be a library, you may want to add this field. This is a string that tells dependency resolution that this directory should be used as the public interface (as opposed to `src` by default).

- For finer control, the option is provided to pass compiler and linker flags directly, using the `compiler-options` and `linker-options` array fields. These are prepended to the arguments generated by vango. In the near future, this system is being phased out in favour of a toolchain agnostic variant.

- `inherits`: this field is exclusive to custom profile definitions, as they require a base of settings to build upon.

### Static Library Configuration
Manifests that begin with `[library]` are specialized for static library linking and are expected to have 3 base declarations at the root:
```toml
[library]
package = "foobar"
version = "x.y.z"
lang = "C++XX"
```
`lang` in this case declares compatibility. Dependency resolution will error on any library that requires a newer C++ standard than the project linking it. In the case of mixing C and C++, the builder assumes all C to be C++ compatible for ease of use, but the user must ensure that this is in fact the case (i.e. that header files use 'clean' C).

In addition, libraries may have `profile.*` sections. Like their `[build]` counterparts, all profile options (except `inherits`) may be specified globally as a default. Libraries support the following profile options:

- `include`: a string that declares where the library header files are.

- `libdir`: a string that declares where the library binaries are.

- `binaries`: a list of the binaries that the library provides. These are specified in name only (no file extension, no 'lib' prefix for .a files).

- `defines`: inserts preprocessor definitions into dependent projects.

- `inherits`: this field is exclusive to custom profile definitions, as they require a base of settings to build upon.

### Automated Testing
Testing is made easy by assuming all tests are in a `test` directory in the project root. A test project is a C/C++ project of arbitrary complexity, and may look like the following:
```cpp
#define VANGO_TEST_ROOT
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
#define VANGO_TEST_ROOT
#include <vango/casserts.h>

test(basic_math) {
    int a = 10;
    assert_eq(a, 10);
}

test_main(
    test_register(basic_math);
)
```
Given these prerequisites, tests can be run on a case by case basis by specifying their names on the command line (see `vango help test`), or all at once by not specifying anything.


### Cross-Compilation
If you're familiar with the Clang toolchain, you already know that these tools support cross-compilation out of the box via its LLVM backend. If you don't need these features or you're used to the clang cross workflow, then plain clang is a fine way to go, specifying the `--target` and `--sysroot` options directly via the toml `*-options` fields whenever necessary. However, one headache this can often cause is that clang does not bundle in the default libraries for the targets it compiles to, and these can be non-trivial to set up, depending on the OS you want to target. Luckily, the brilliant developers of zig have solved this problem for us.

As referenced earlier, vango supports the usage of zig as a C/C++ compiler (not for the zig language itself unfortunately). This works because zig includes clang as part of its ecosystem, however, the main benefit of using zig's wrappers vs plain clang, is that zig *does* ship with system libraries for many many platforms. This means that if you have zig on your system, no messing around with `sysroot`s is necessary. In fact, you do not need to so much as touch the target platform until you ship. All that's required is to specify the (zig style) target triple like so:
```toml
compiler-options = [ "-target", "<machine>-<os>-<abi>" ]
linker-options = [ "-target", "<machine>-<os>-<abi>" ]
```
and the correct binary will be generated.

In future, I hope to implement this bundling myself via the package manager (which I have yet to begin working on), as it does seem silly to require 3 different compilers just to build Hello World to an ELF file on windows, but for now this is a relatively simple solution to an unnecessarily overcomplicated problem. For more info, see article [Zig Makes Rust Cross-compilation Just Work](https://actually.fyi/posts/zig-makes-rust-cross-compilation-just-work/).

## Planned Features
### Feature Flags
Conditional compilation based on requested features

### Platform Agnostic TOML Manifest
Fully platform agnostic compiler and linker options for improved cross-platform and cross-compilation functionality

### Smart Sem-Ver
Improved integration with Git tags to enable versioned dependencies, lockfiles

### Package Manager
Registry of popular libraries and build recipes to enable full environment automation for open-source projects

### Zig-like Cross-Compilation
System libraries as packages accessible via the package manager

### Generator Functionality
The ability to transpile a build script to other popular tools such as CMake, Make, MSBuild and more

