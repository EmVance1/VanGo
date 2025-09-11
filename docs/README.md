# VanGo Documentation

VanGo is opinionated for simplicity and makes some base assumptions and decisions:
- You have a valid `Vango.toml` in the project root.
- All of your source files are in the `src` directory, and all output files are generated in `bin/{profile}/`.
- Your output binary is named the same as your project.
- For a given project, you can make a platform specific build definition by naming the file 'win.vango.toml', 'lnx.vango.toml', or 'mac.vango.toml'.
- A correct `Vango.toml` may begin with one of 2 sections - `[package]` and `[staticlib]`.

### Table of Contents
- [Toolchains](toolchains.md)
- [Build Configuration](builds.md)
- [Static Library Configuration](libraries.md)
- [Automated Testing](testing.md)
- [Cross-Compilation](crosscomp.md)

