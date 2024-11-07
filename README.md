# MS Compile (MSBuild was taken)

Yes millions of options are lovely, but actually what's wrong with sensible defaults? Use JSON to define a minimal build script, for example:
```
{
    "project": "example",
    "cpp": "C++20",
    "dependencies": []
}
```
The above configuration is already the minimum requirement. `./src` is assumed as the main source file directory (what the hell else are you putting there?) and added to the include path. `./bin` holds any incremental build files (usually object files).
## Features supported so far
- Build, Run and Clean actions

- Specify header-only and binary libraries with a lib.json, supports multiple configurations...
```
{
    "library": "SFML",
    "minstd" : "c++11",
    "include": "include/",
    "configs": {
        ...
    }
},
```
(see example `lib.json` for more)

- ...and plug and play at will in main project. Libraries can be placed in `./lib` or specified otherwise
```
"dependencies": [ "SFML.static", "../Rusty" ],
```
- Preprocessor definitions
```
"defines": [ "MACRO" ],
```
- Precompiled headers were never easier
```
"pch": "pch.h",
```
- Incremental building based on recent file changes

- Source code dependencies are automatically built recursively in the case of updates

- Executable/Library output deduction using main.cpp/lib.cpp entry points

- Debug and Release configurations (work in progress)
```
"SETTING.debug": { ... },
"SETTING.release": { ... },
```

## It just works
Slap a `build.json` next to a `src` directory with a `main.cpp` in it and everything will just work. Was that so hard Microsoft? No Visual Studio needed.

