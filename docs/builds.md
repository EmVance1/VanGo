# Build Configuration
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
- `version` takes a sem-ver number. At time of writing, this has no effect, but is worth maintaining nonetheless for clarity and for when versioned packages are implemented.
- `lang` takes any valid C or C++ standard, case insensitive.
- `kind` is for declaring whether your project builds to an executable or a library. The default value here is `"app"`, though it can be written explicitly. `staticlib` will produce a symbol archive file for your toolchain (.a, .lib, etc.). In contrast to other kinds, the behaviour of `sharedlib` varies widely per platform, *regardless of toolchain*. On linux, it creates a .so file, a .dylib on mac, while on windows it will produce a '.dll' binary and (by default) a static *import* library for automatic symbol loading. The macro `VANGO_EXPORT_SHARED` is also defined when building a DLL file, for all your `__declspec` needs.

    **Note**: At time of writing, DLLs must be manually moved to the dependent projects working directory for correct linkage.

- `interface`: at times you may want to implement a library using one standard, but provide interfaces for use in another earlier standard, or even C. To partially bypass the compatibility checker, you can declare the `interface` array, which lists all standards your headers are compatible with. Elements of `interface` use the same format as `lang`.

### Dependencies
The `dependencies` section is the main workhorse of the build system. Within it, you can list 0 or more named objects representing libraries also supported by VanGo. If no path to the library is specified, VanGo will search in '~/.vango/packages/'. A dependency that is not header-only must have a toml file in its root directory . Source libraries will be automatically built recursively by any project that includes them. Currently supported ways of specifying dependencies are as follows:
```toml
[dependencies]
MyLib     = { path="../MyLib" } # source, local, contains build toml-config
SFML      = { path="../SFML" }  # binary, local, contains static lib toml-config
SFUtils   = { git="https://github.com/EmVance1/ShimmyNav.git" } # source, remote, contains build toml-config
stb_image = { headers="lib/stb_image" } # headers, local, contains no config
Ws2       = { system="Ws2_32", target="windows" } # system libraries
```
Support for git dependencies is currently very basic. The repo is cached in '~/.vango/packages/', and is otherwise treated just like any other dependency (must contain a build script, etc.). For libraries that arent native to Vango, the ability to write automated build recipes (e.g. CMake invocation + toml injection) is coming soon.

As it stands, there are plans for a very basic package manager, more a simple registry of URLs of popular libraries and corresponding build recipes, but this is a ways away for now.

### Profiles
To customize build profiles or define your own that inherites one of the builtins, you can define the `profile.*` sections. All of the following options (except `inherits`) can be defined globally (under `[package]`) as a default, or under `[profile.debug]`, `[profile.release]`, or any `[profile.mycustomprofile]`.

- `defines`: additional preprocessor definitions. By default, this array will contain `VANGO_DEBUG` or `VANGO_RELEASE` definitions, aswell as `VANGO_TEST` for test builds, and `VANGO_EXPORT_SHARED` for dll builds.
- `pch`: if you want to precompile a header, just specify the header file relative to `src/` that you want precompiled as shown above (All source files will be assumed to use it).
- Source directory and (internal) include directories are assumed to be `./src` and `[ ./src ]` respectively, however they can be overridden or extended through the `src` and `include` options respectively.
- `include-pub`: if the project you are defining is going to be a library, you may want to add this field. This is a string that tells dependency resolution that this directory should be used as the public interface (as opposed to `src` by default).
- **settings**: the following are broad toolchain agnostic settings that translate to various compiler and linker options.
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
    * `pthread`: (GNU only) enable pthreads (`true|false`)

- For finer control, the option is provided to pass compiler and linker flags directly, using the `compiler-options` and `linker-options` array fields. These are prepended as-is to the arguments generated by vango. Options you put here must of course be tailored to your platform.
- `inherits`: this field is exclusive to custom profile definitions, as they require a base of settings to build upon.

**Important note**: all toolchain specific implementations of the options listed above may come with caveats not listed here. Arguments from different compilers will never be a perfect match. If you expect to be switching between toolchains often, a list of all implementations, aswell as profile defaults can be viewed in `toolchains/`, for further reading into platform specific quirks.

