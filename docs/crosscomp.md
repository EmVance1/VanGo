# Cross-Compilation
If you're familiar with the Clang toolchain, you already know that these tools support cross-compilation out of the box via its LLVM backend. If you don't need these features or you're used to the clang cross workflow, then plain clang is a fine way to go, specifying the `--target` and `--sysroot` options directly via the toml `*-options` fields whenever necessary. However, one headache this can often cause is that clang does not bundle in the default libraries for the targets it compiles to, and these can be non-trivial to set up, depending on the OS you want to target. Luckily, the brilliant developers of zig have solved this problem for us.

As referenced earlier, vango supports the usage of zig as a C/C++ compiler (not for the zig language itself unfortunately). This works because zig includes clang as part of its ecosystem, however, the main benefit of using zig's wrappers vs plain clang, is that zig *does* ship with system libraries for many many platforms. This means that if you have zig on your system, no messing around with `sysroot`s is necessary. In fact, you do not need to so much as touch the target platform until you ship. All that's required is to specify the (zig style) target triple like so:
```toml
compiler-options = [ "-target", "<machine>-<os>-<abi>" ]
linker-options = [ "-target", "<machine>-<os>-<abi>" ]
```
and the correct binary will be generated.

In future, I hope to implement this bundling myself via the package manager (which I have yet to begin working on), as it does seem silly to require 3 different compilers just to build Hello World to an ELF file on windows, but for now this is a relatively simple solution to an unnecessarily overcomplicated problem. For more info, see article [Zig Makes Rust Cross-compilation Just Work](https://actually.fyi/posts/zig-makes-rust-cross-compilation-just-work/).

