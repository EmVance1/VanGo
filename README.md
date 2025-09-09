# VanGo - A C/C++ Build System for Cargo Lovers

## Motivation

This app is a build system designed with rusts cargo philosophy in mind. You can have a million options, but there is nothing wrong with sensible defaults. Vango uses project file structure as a component of its project configuration, minimizing the need for a build script. `./src` is assumed as the main source file directory and added to the include path. `./bin` holds any incremental build files (usually object files). `./test` is for source files that contain tests. All manual configuration is done via the `Vango.toml` manifest file in the project root.

The system supports most popular toolchains, specifically: GNU and Clang/LLVM on all platforms, as well as MSVC on windows. It does of course assume that you have all relevant compiler tools installed, as it is not in itself a compiler. For easier cross compilation, vango also supports zig as a target, which wraps clang. To read why this is useful, see chapter on [cross-compilation](#Cross-Compilation).

## Features Available So Far
- Subcommands for creating, building, running, testing, and cleaning C/C++ projects
- File change detection and incremental rebuilds
- Source, static, and header-only  dependency automation
- Configure static libraries with a staticlib-type toml file
```toml
[staticlib]
name = "SFML"
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

- `vango new     [--lib] [--c] [--clangd] <name>`
- `vango b[uild] [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>]`
- `vango r[un]   [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>] [-- args*]`
- `vango c[lean]`
- `vango help    [action]`

VanGo is opinionated for simplicity and makes some base assumptions and decisions:
- You have a valid `Vango.toml` in the project root.
- All of your source files are in the `src` directory, and all output files are generated in `bin/{profile}/`.
- Your output binary is named the same as your project.
- For a given project, you can make a platform specific build definition by naming the file 'win.vango.toml', 'lnx.vango.toml', or 'mac.vango.toml'.
- A correct `Vango.toml` may begin with one of 2 sections - `[package]` and `[staticlib]`.

### Toolchains
Vango is not itself a compilation toolchain, simply a compilation automator. For everything to work, you need at least one compiler installed on your system and visible in your `PATH` variable. Currently supported toolchains are MSVC (windows only), GCC (linux, mingw windows, macos), Clang/LLVM (universal, both GNU and MSVC ecosystems).
All platforms have a compiler toolchain they default to - MSVC on windows, GCC on linux, Clang on macos - this can be overridden using the -t switch on build, run, and test commands. The `-t=msvc` option is provided for completeness, despite the tool being unavailable on non-windows platforms. Clang on windows will default to its MSVC variant. This can be overridden by using `-t=clang-gnu`.
To change your system default toolchain, set the environment variable `VANGO_DEFAULT_TOOLCHAIN` to one of the 5 valid values.

### Build Configuration
All manifests that begin with `[package]` are expected to have 3 base declarations at the root:
```toml
[package]
name = "foobar"
version = "x.y.z"
lang = "C++XX"
# optional
kind = "app|staticlib|sharedlib"
implib = true
interface = [ "CXX" ]
```
- `name` is an arbitrary string that defines how your project is viewed in the builder. This is for example the name the builder will look for when resolving source dependencies (see later).

- `version` takes a sem-ver number. At time of writing, this has no effect, but is worth maintaining nonetheless for clarity and for future use cases.

- `lang` takes any valid C or C++ standard, case insensitive.

- `kind` is for declaring whether your project builds to an executable or a library. The default value here is `"app"`, though it can be written explicitly. `staticlib` will produce a symbol archive file for your toolchain (.a, .lib, etc.). In contrast to other kinds, the behaviour of `sharedlib` varies widely per platform, *regardless of toolchain*. On linux, we produce a .so file, a .dylib on mac, while on windows it will produce a '.dll' binary and (by default) a static *import* archive that allows automatic symbol loading. The macro `VANGO_EXPORT_SHARED` is also defined when building a dll file on windows, for all your `__declspec` needs.

    **Note**: At time of writing, windows dlls must be manually moved to the dependent projects working directory for correct linkage.

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

- **profile**: to customize build profiles or define your own that inherites one of the builtins, you can define the `profile.*` sections. All of the following options (except `inherits`) can be defined globally (under `[package]`) as a default, or under `[profile.debug]`, `[profile.release]`, or any `[profile.mycustomprofile]`.

- `defines`: additional preprocessor definitions. By default, this array will contain `VANGO_DEBUG` or `VANGO_RELEASE` definitions, aswell as `VANGO_TEST` for test builds, and `VANGO_EXPORT_SHARED` for dll builds.

- `pch`: if you want to precompile a header, just specify the header file relative to `src/` that you want precompiled as shown above (All source files will be assumed to use it).

- Source directory and (internal) include directories are assumed to be `./src` and `[ ./src ]` respectively, however they can be overridden or extended through the `src` and `include` options respectively.

- `include-pub`: if the project you are defining is going to be a library, you may want to add this field. This is a string that tells dependency resolution that this directory should be used as the public interface (as opposed to `src` by default).

- **settings**: the following are some basic toolchain agnostic settings that translate to various compiler and linker options.
    * `opt-level`: level of compiler optimization (`0|1|2|3`)
    * `opt-size`: optimize for smaller binaries (`true|false`)
    * `opt-speed`: optimize agressively for fast code `true|false`) note: uses -Ofast on GNU, which can be problematic
    * `opt-linktime`: optimize at link time (`true|false`)
    * `debug-info`: generate debugging information (`true|false`)
    * `warn-level`: level of compiler warning diagnostics (`"none"|"basic"|"high"`)
    * `warn-as-error`: treat compiler warnings as errors (`true|false`)
    * `iso-compliant`: treat usage of compiler extensions as errors (`true|false`)
    * `aslr`: use ASLR (`true|false`)
    * `rtti`: use RTTI (`true|false`)

- For finer control, the option is provided to pass compiler and linker flags directly, using the `compiler-options` and `linker-options` array fields. These are prepended to the arguments generated by vango. In the near future, this system is being phased out in favour of a toolchain agnostic variant.

- `inherits`: this field is exclusive to custom profile definitions, as they require a base of settings to build upon.

Important note: all toolchain specific implementations of the options listed above may come with caveats not listed here. Arguments from different compilers will never be a perfect match. If you expect to be switching between toolchains often, a list of all implementations, aswell as profile defaults can be viewed in `toolchains/`, for further reading into platform specific quirks.

### Static Library Configuration
Manifests that begin with `[staticlib]` are specialized for static library linking and are expected to have 3 base declarations at the root:
```toml
[staticlib]
name = "foobar"
version = "x.y.z"
lang = "C++XX"
```
`lang` in this case declares compatibility. Dependency resolution will error on any library that requires a newer C++ standard than the project linking it. In the case of mixing C and C++, the builder assumes all C to be C++ compatible for ease of use, but the user must ensure that this is in fact the case (i.e. that header files use 'clean' C).

In addition, libraries may have `profile.*` sections. Like their `[package]` counterparts, all profile options (except `inherits`) may be specified globally as a default. Libraries support the following profile options:

- `include`: a string that declares where the library header files are.

- `libdir`: a string that declares where the library binaries are.

- `binaries`: a list of the binaries that the library provides. These are specified in name only (no file extension, no 'lib' prefix for .a files).

- `defines`: inserts preprocessor definitions into dependent projects.

- `inherits`: this field is exclusive to custom profile definitions, as they require a base of settings to build upon.

### Automated Testing
Vango supports automated testing for library projects. To benefit from this, its best to modularize your core functionality into a library, which is then driven by a separate binary project (this is generally considered good practice in any framework). Test projects are arbitrarily complex C/C++ projects, the source code for which you place in the `test` directory in the project root.

In order to write tests, the header 'vangotest/asserts2.h' - 'vangotest/casserts2.h' for C - must be included. These are automatically visible for test configurations. As the name suggests, these contain basic assert macros that report back the success status of the test. In one file and one file only, the include statement must be preceded by the `VANGO_TEST_ROOT` definition. This enables automatic discovery of your tests, meaning you dont need to call or even forward declare your tests anywhere. A dummy test project might look like this:
```cpp
#define VANGO_TEST_ROOT
#include <vango/asserts2.h>

vango_test(basic_math) {
    int a = 2;
    a += 3;
    a *= 2;

    vg_assert_eq(a, 10);
}
```
As you can see, a test is essentially a pure void function. Tests can be run all at once, or on a case by case basis by specifying the test names on the command line.

**Important note**: the '*2.h' family of assert headers is currently experimental on MSVC (including clang-msvc), due to some awkard pointer hacks it performs to make automatic discovery work. If MSVC users prefer, the old, less experimental headers are still available ('asserts.h', 'casserts.h'). These behave identically for C++, altough forward declaration and inclusion into the test root is necessary via the `vango_test_decl(test_name)` macro. In C however, some automation features are unavailable, and in addition to the code seen above, you must register your tests in the root like so:
```cpp
#define VANGO_TEST_ROOT
#include <vango/casserts.h>

vango_test(basic_math) {
    int a = 10;
    vg_assert_eq(a, 10);
}

vango_test_main(
    vango_test_reg(basic_math);
)
```

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

### Smart Sem-Ver
Improved integration with Git tags to enable versioned dependencies, lockfiles

### Package Manager
Registry of popular libraries and build recipes to enable full environment automation for open-source projects

### Zig-like Cross-Compilation
System libraries as packages accessible via the package manager

### Generator Functionality
The ability to transpile a build script to other popular tools such as CMake, Make, MSBuild and more

