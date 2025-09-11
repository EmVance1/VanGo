# Toolchains
Vango is not itself a compilation toolchain, simply a compilation automator. For everything to work, you need at least one compiler installed on your system and visible in your `PATH` variable. Currently supported toolchains are MSVC (windows only), GCC (linux, mingw windows, macos), Clang/LLVM (universal, both GNU and MSVC ecosystems).

All platforms have a compiler toolchain they default to - MSVC on windows, GCC on linux, Clang on macos - this can be overridden using the -t switch on build, run, and test commands. The `-t=msvc` option is provided for completeness, despite the tool being unavailable on non-windows platforms. Clang on windows will default to its MSVC variant. This can be overridden by using `-t=clang-gnu`.

To change your system default toolchain, set the environment variable `VANGO_DEFAULT_TOOLCHAIN` to one of the 5 valid values.

