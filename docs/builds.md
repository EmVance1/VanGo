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
interface = "CXX"
```
- `name` is an arbitrary string that defines how your project is viewed in the builder. This is for example the name the builder will look for when resolving source dependencies (see later).
- `version` takes a sem-ver number. At time of writing, this has no effect, but is worth maintaining nonetheless for clarity and for when versioned packages are implemented.
- `lang` takes any valid C or C++ ISO standard, case insensitive. GNU standards not yet supported. Aside from compiler settings, if the `interface` field is not defined, `lang` also declares a libraries minimum compatibility (see [Library Configuration](libraries.md)).
- `kind` is for declaring whether your project builds to an executable (`app`, default) or a library. `staticlib` will produce a symbol archive file for your toolchain (.a, .lib). In contrast to other kinds, the behaviour of `sharedlib` varies widely per platform, *regardless of toolchain*. On linux, it creates a .so file, a .dylib on mac, while on windows it will produce a '.dll' binary and (by default) a static *import* library for automatic symbol loading. The macro `VANGO_EXPORT_SHARED` is also defined when building a DLL file, for all your `__declspec` needs.

    **Note**: At time of writing, DLLs must be manually moved to the dependent projects working directory for correct linkage.

- `interface`: at times you may want to implement a library using one standard, but provide an interface for use in another earlier standard, or in C. To partially bypass the compatibility checker, you can declare the `interface` field, which sets the earliest standard your library is compatible with. `interface` uses the same format as `lang`.

### Dependencies
The `dependencies` section is the main workhorse of the build system. Within it, you can list 0 or more named objects representing libraries also supported by VanGo. A dependency that is not header-only must have a toml file in its root directory. Source libraries will be automatically built recursively by any project that includes them. Currently supported ways of specifying dependencies are as follows:
```toml
[dependencies]
MyLib     = { path="../MyLib" } # source, local, contains [package] (build) toml-config
SFML      = { path="../SFML" }  # binary, local, contains [staticlib] (prebuilt) toml-config
SFUtils   = { git="https://github.com/EmVance1/ShimmyNav.git" } # source, remote, contains [package] toml-config
stb_image = { headers="lib/stb_image" } # headers, local, contains no config
Ws2       = { system="Ws2_32", target="windows" } # system binaries require no config
```
**Note**: if you are building a *static* library, it is important to remember that no dependencies are bundled into the binary you build - they still need to be linked into the final executable. For example, if you are building a wrapper library for the Winsock2 API, the executable consuming it must list said library **and** `Ws2_32.lib` in its dependencies (this is not the case for *shared* libraries, as they are created via the linker). Despite this, static library projects should always declare all dependencies, both for user clarity, and because tests need to inherit them (tests are effectively dependent executables).

Support for git dependencies is currently very basic. The repo is cached (and searched for) in `~/.vango/packages/`, and is otherwise treated just like any other dependency (must contain a build script, etc.). For libraries that arent native to Vango, the ability to write automated build recipes (e.g. CMake invocation + toml injection) is coming soon.

### Profiles
To customize build profiles or define your own that inherites one of the builtins, you can define the `profile.*` sections. All of the following options (except `inherits`) can be defined globally (under `[package]`) as a default, or under `[profile.debug]`, `[profile.release]`, or any `[profile.mycustomprofile]`.

- `defines`: additional preprocessor definitions. This option always **extends** whatever defaults you have set, as opposed to overwriting them. See a list of builtin preprocessor definitions below.
- `include` is an array of strings to add to your (private) include directories, which by default contains only `src` (and `include` in libraries). This option always **extends** whatever defaults you have set, as opposed to overwriting them. Most of the time you can leave this field blank and rely on your `[dependencies]` to populate this for you.
- `pch`: if you want to precompile a header, specify the header file relative to `src` that you want precompiled (only one header per project, all source files will be assumed to use it).
- **settings**: the following are broad toolchain agnostic settings that translate to various compiler and linker options. A * indicates a universal default if applicable.
    * `opt-level`: level of compiler optimization (`0|1|2|3`)
    * `opt-size`: optimize for smaller binaries (`true|false*`)
    * `opt-speed`: optimize agressively for fast code (`true|false*`), note: uses -Ofast on GNU, which can be problematic
    * `opt-linktime`: optimize at link time (`true|false`)
    * `debug-info`: generate debugging information (`true|false`)
    * `warn-level`: level of compiler warning diagnostics (`"none"|"basic"*|"high"`)
    * `warn-as-error`: treat compiler warnings as errors (`true|false*`)
    * `iso-compliant`: treat usage of compiler extensions as errors (`true|false*`)
    * `aslr`: use ASLR (`true*|false`)
    * `no-rtti`: (C++ only) disable RTTI (`true|false*`)
    * `no-except`: (C++ only) disable exceptions (`true|false*`)
    * `pthreads`: (GNU only) enable pthreads (`true|false*`)
- **sanitizers**: sanitizer settings work like any other, but their applicability is highly platform dependent. On UNIX systems, all sanitizers are always available. Windows is trickier. Windows only universally supports AddressSanitizer, while UndefinedBehaviorSanitizer has partial support when using clang. Options turning on unsupported sanitizers will simply be ignored, however there is one annoying edge-case that GNU/Clang on windows *does* support ASan, but does not ship with the required libraries bundled in, and will fail to link if you haven't installed them (MSVC/Clang does not have this problem). The following options are provided for enabling sanitizers:
    * `sanitize.address`: compile with AddressSanitizer (`true|false*`)
    * `sanitize.thread`: compile with ThreadSanitizer (`true|false*`)
    * `sanitize.leak`: compile with LeakSanitizer (`true|false*`)
    * `sanitize.undefined`: compile with UndefinedBehaviourSanitizer (`true|false*`)

- For finer control, the option is provided to pass compiler and linker flags directly, using the `compiler-options` and `linker-options` array fields. These are prepended as-is to the arguments generated by vango. Options you put here must of course be tailored to your platform.
- `inherits`: this field is exclusive to (but required for) custom profile definitions, as they require a base of settings to build upon. It may have the value `"debug"` or `"release"`.

**Note**: to maintain build predictability, most settings listed above (all but `warn-level`, `include`) will trigger a full rebuild of your project when changed (non-recursive). This also includes preprocessor definitions that are modified indirectly, such as the `VANGO_PKG` family of macros. This is worth keeping in mind for large projects with long build times.

**Important note**: all toolchain specific implementations of the options listed above may come with caveats not listed here. Arguments from different compilers will rarely be a perfect match. If you expect to be switching between toolchains often, a list of all implementations, aswell as profile defaults can be viewed in `docs/toolchains`, for further reading into platform specific quirks.


### Preprocessor Definitions
The defines array will contain a number of vango specific preprocessor definitions for various use cases.
- Universally defined:
    * `VANGO_PKG_NAME` a string literal matching the `name` field of the project.
    * `VANGO_PKG_VERSION` a string literal matching the `version` field of the project.
    * `VANGO_PKG_VERSION_MAJOR` an integer matching the major version of the project.
    * `VANGO_PKG_VERSION_MINOR` an integer matching the minor version of the project.
    * `VANGO_PKG_VERSION_PATCH` an integer matching the patch of the project.

- Debug builds (or those inheriting)
    * `VANGO_DEBUG` for conditional compilation.

- Release builds (or those inheriting)
    * `VANGO_RELEASE` for conditional compilation.

- Test builds
    * `VANGO_TEST` for conditional compilation.

- Windows builds
    * `VANGO_EXPORT_SHARED` defined for `sharedlib` projects, intended for use with `__declspec(dll*)`, though not required.
    * `UNICODE`, `_UNICODE` can be ignored, makes unicode the default mode for system functions.

