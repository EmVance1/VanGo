# Prebuilt Library Configuration
Manifests that begin with `[staticlib]` are specialized for prebuilt library linking and are expected to have 3 base declarations at the root:
```toml
[staticlib]
name = "foobar"
version = "x.y.z"
lang = "C++XX"
```
`lang` in this case declares compatibility. Dependency resolution will error on any library that requires a newer C++ standard than the project linking it. In the case of mixing C and C++, the builder assumes all C to be C++ compatible for ease of use, but the user must ensure that this is in fact the case (i.e. that header files use 'clean' C).

### Profiles
Libraries may have `profile.*` sections. Like their `[package]` counterparts (see [Build Configuration](builds.md)), all profile options (except `inherits`) may be specified globally as a default. Libraries support the following profile options:

- `include`: a string that declares where the library header files are.
- `libdir`: a string that declares where the library binaries are.
- `binaries`: a list of the binaries that the library provides. These are specified in name only (no file extension, no 'lib' prefix for .a files).
- `defines`: inserts preprocessor definitions into dependent projects.
- `inherits`: this field is exclusive to custom profile definitions, as they require a base of settings to build upon.

