# VanGo Documentation

VanGo is opinionated for simplicity and makes some base assumptions and decisions. All of your source files are in the `src` directory, all test files in `test`, and all output files are generated in `bin/{profile}`. You must also have a valid `Vango.toml` in the project root. For a given project, you can make a platform specific build manifest by naming the file `win.Vango.toml`, `lnx.Vango.toml`, or `mac.Vango.toml`. A correct `Vango.toml` may begin with one of 2 sections - `[package]` and `[staticlib]` (see relevant chapters).

### Table of Contents
- [Toolchains](toolchains.md)
- [Build Configuration](builds.md)
- [Prebuilt Library Configuration](libraries.md)
- [Automated Testing](testing.md)

