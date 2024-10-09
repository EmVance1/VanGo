# MS Compile (MSBuild was taken)

Yes millions of options are lovely, but actually what's wrong with sensible defaults? Use JSON to define a minimal build script, for example:
```
{
    "project": "example",
    "cpp": "C++20",
    "dependencies": {}
}
```
The above configuration is already the minimum requirement. `./src` is assumed as the main source file directory (what the hell else are you putting there?) and added to the include path. `./obj` holds any incremental build files (usually object files).
## Features supported so far
- Build, Run and Clean actions

- Preprocessor definitions
```
"defines": [ "MACRO" ],
```
- Header-only and binary libraries
```
"dependencies": {
    "somelib": { "include": "path/to/include" },
    "morelib": {
        "include": "path/to/include",
        "bindir": "path/to/binary",
        "link": [ "binfile.lib" ]
    }
},
```
- Precompiled headers were never easier
```
"pch": "pch.h",
```
- Self-invocation for multi-project builds
```
"require": [ "../libproject" ],
```
- Incremental building based on recent file changes

- Executable/Library output deduction using main.cpp/lib.cpp entry points

- Debug and Release configurations (work in progress)
```
"cfg.debug": { ... },
"cfg.release": { ... },
```

## It just works
Slap a `build.json` next to a `src` directory with a `main.cpp` in it and everything will just work. Was that so hard Microsoft? No Visual Studio needed.
