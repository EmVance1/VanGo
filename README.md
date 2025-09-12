# VanGo - A C/C++ Build System for Cargo Lovers

A build system designed with rusts cargo philosophy in mind. You can have a million options, but there is nothing wrong with sensible defaults. Vango uses project file structure as a component of its project configuration, minimizing the need for a build script. `./src` is the source file directory and added to the include path. `./bin` holds any incremental build files (usually object files). `./test` is for source files that contain tests. All manual configuration is done via the `Vango.toml` manifest file in the project root.

The system supports most popular toolchains, specifically: GNU and Clang/LLVM on all platforms, as well as MSVC on windows. It does of course assume that you have all relevant compiler tools installed, as it is not in itself a compiler. For easier cross compilation, vango also supports zig as a target, which wraps clang. To read why this is useful, see the docs chapter on [cross-compilation](docs/toolchains.md).

For a more in-depth explanation, see the [documentation](docs/README.md).

## Features Available So Far
- Subcommands for creating, building, running, testing, and cleaning C/C++ projects. Some usage examples are as follows, but for a more complete list see the help action.
    * `vango new     [--lib] [--c] [--clangd] <name>`
    * `vango b[uild] [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>]`
    * `vango r[un]   [-r|--release] [-t|--toolchain=<msvc|gnu|clang|zig>] [-- args*]`
    * `vango c[lean]`
    * `vango help    [action]`

- File change detection and incremental rebuilds
- Source, prebuilt, and header-only dependency automation
- Cross-platform abstraction of shared library builds, precompiled headers, system dependencies and more
- Complete per-profile freedom of configuration
```toml
[profile.debug]
defines = [ "MACRO", "VALUE=10" ]

[profile.myprofile]
include = [ "src", "../some/other/headers" ]
```
- Configure prebuilt libraries with a staticlib-type toml file
```toml
[staticlib]
name = "SFML"
version = "3.0.1"
lang = "C++17"
include = "include"
...
```
- Cross compilation via Clang/Zig

**Conclusion**: It just works. Even without boilerplate generation via `vango new`, slap a `Vango.toml` next to a `src` directory with a `main.cpp` in it and everything will just work. Was that so hard everybody?

## Caveat on MacOS
Although I have tried my best to dilligently research MacOS workflows and write correct code to the best of my theoretical knowledge, I am a solo developer with no direct access to an Apple computer. Because of this, I have no way of knowing *for sure* what the real world behaviour of Vango will be on such platforms specifically. Those of you using this on MacOS, do so at your own risk, and know I welcome people to test what I cannot.

## Future Plans
- **Feature Flags:** Conditional compilation based on requested features
- **Smart Sem-Ver:** Improved integration with Git tags to enable versioned dependencies, lockfiles
- **Package Manager:** Registry of popular libraries and build recipes to enable full environment automation for open-source projects
- **Zig-like Cross-Compilation:** System libraries as packages accessible via the package manager
- **Generator Functionality:** The ability to transpile a build script to other popular tools such as CMake, Make, MSBuild and more

