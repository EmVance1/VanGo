use std::{io::Write, path::PathBuf};
use crate::{
    error::Error,
    exec::{self, BuildInfo},
    input::BuildSwitches, config::*,
    fetch,
};


pub fn build(mut build: BuildFile, switches: &BuildSwitches) -> Result<(bool, PathBuf), Error> {
    let profile = build.take(&switches.profile)?;
    let srcdir = PathBuf::from("src");
    let mut headers = fetch::source_files(&profile.include_pub, "h").unwrap();
    if build.lang.is_cpp() {
        headers.extend(fetch::source_files(&profile.include_pub, "hpp").unwrap());
    }
    for incdir in &profile.include {
        headers.extend(fetch::source_files(incdir, "h").unwrap());
        if build.lang.is_cpp() {
            headers.extend(fetch::source_files(incdir, "hpp").unwrap());
        }
    }
    let sources = fetch::source_files(&srcdir, build.lang.src_ext()).unwrap();

    let mut deps = fetch::libraries(build.dependencies, switches, build.lang)?;
    deps.defines.extend(profile.defines);
    if switches.is_test { deps.defines.push("VANGO_TEST".to_string()); }
    if cfg!(windows) {
        deps.defines.push("UNICODE".to_string());
        deps.defines.push("_UNICODE".to_string());
        if let ProjKind::SharedLib{..} = build.kind {
            deps.defines.push("VANGO_EXPORT_SHARED".to_string());
        }
    }
    deps.incdirs.extend(profile.include);

    let rebuilt_dep = deps.rebuilt;
    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let (outfile, implib) = match build.kind {
        ProjKind::App => {
            (outdir.join(build.name).with_extension(switches.toolchain.app_ext()), None)
        }
        ProjKind::SharedLib{implib: false} => {
            (outdir.join(format!("{}{}", ToolChain::shared_lib_prefix(), build.name))
             .with_extension(ToolChain::shared_lib_ext()), None)
        }
        ProjKind::SharedLib{implib: true} => {
            (outdir.join(format!("{}{}", ToolChain::shared_lib_prefix(), build.name))
             .with_extension(ToolChain::shared_lib_ext()),
             Some(outdir.join(format!("{}{}", switches.toolchain.static_lib_prefix(), build.name))
             .with_extension(switches.toolchain.static_lib_ext())))
        }
        ProjKind::StaticLib => {
            (outdir.join(format!("{}{}", switches.toolchain.static_lib_prefix(), build.name))
             .with_extension(switches.toolchain.static_lib_ext()), None)
        }
    };

    let info = BuildInfo{
        projkind:  build.kind,
        toolchain: switches.toolchain,
        lang:      build.lang,
        crtstatic: switches.crtstatic,
        cpprt:     build.runtime.map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),
        settings:  profile.settings,

        defines:  deps.defines,

        srcdir,
        incdirs:  deps.incdirs,
        libdirs:  deps.libdirs,
        outdir,

        pch:      profile.pch,
        sources,
        headers,
        archives: deps.archives,
        relink:   deps.relink,
        outfile:  outfile.clone(),
        implib,

        comp_args: profile.compiler_options,
        link_args: profile.linker_options,
    };
    match exec::run_build(info, switches.echo, switches.verbose) {
        Err(e) => Err(e),
        Ok(rebuilt) => Ok((rebuilt_dep || rebuilt, outfile)),
    }
}


pub fn help(action: Option<String>) {
    let print_build_details = || {
        println!("Options:");
        println!("  -d, --debug             Build project in debug profile (default)");
        println!("  -r, --release           Build project in release profile");
        println!("  -t, --toolchain=<TOOL>  Specify a toolchain for compilation (user default: {})", ToolChain::default());
        println!("      --install           On unix-like systems: installs headers and binaries into /usr/local/* on build");
        println!("      --crtstatic         Link statically with the C runtime library");
        println!("      --echo              Echo the entire build command composed by vango");
        println!("  -v, --verbose           Forward '--verbose' to invoked tool, if available");
        println!();
        println!("Profiles:");
        println!("    debug    Build with no optimization; Generate debug symbols; 'VANGO_DEBUG' macro defined; Generally faster compile times;");
        println!("    release  Build with high optimization; 'VANGO_RELEASE' macro defined; Generally slower compile times;");
    };

    if let Some(action) = action {
        match action.as_str() {
            "new" => {
                println!("Create a new folder with a boilerplate C++ project");
                println!();
                println!("Usage: vango new <NAME> [OPTIONS]");
                println!();
                println!("Options:");
                println!("    --lib  Generate library boilerplate instead of application");
                println!("    --c    Generate C boilerplate instead of C++");
            }
            "init" => {
                println!("Create a boilerplate C++ project inside an existing folder");
                println!();
                println!("Usage: vango init [OPTIONS]");
                println!();
                println!("Options:");
                println!("    --lib  Generate library boilerplate instead of application");
                println!("    --c    Generate C boilerplate instead of C++");
            }
            "clean" => {
                println!("Remove all generated build files from the current project");
                println!();
                println!("Usage: vango clean");
            }
            "clangd" => {
                println!("Generate 'compile_flags.txt corresponding to the current project (language standard, include dirs, definitions");
                println!();
                println!("Usage: vango clangd");
            }
            "build" => {
                println!("Build the current project");
                println!();
                println!("Usage: vango build [OPTIONS]");
                println!();
                print_build_details();
            }
            "run" => {
                println!("Build and run the current project, with the working directory as the project root, and forwarding commandline arguments");
                println!();
                println!("Usage: vango run [OPTIONS] [-- ARGS]");
                println!();
                print_build_details();
            }
            "test" => {
                println!("Build the current project in test configuration, link it to your test app and run it. Defines 'VANGO_TEST'.");
                println!();
                println!("Usage: vango test [OPTIONS] [TESTS]");
                println!();
                print_build_details();
            }
            "toolchains" => {
                println!("Toolchains currently installed on this system:");
                println!();
                if std::process::Command::new("gcc").output().is_ok() {
                    if cfg!(windows) {
                        println!("    gcc        - GCC, Gnu Compiler Collection for MinGW");
                    } else {
                        println!("    gcc        - GCC, Gnu Compiler Collection");
                    }
                } else {
                    println!("    gcc        - unavailable");
                }
                if std::process::Command::new("clang").output().is_ok() {
                    if cfg!(windows) {
                        println!("    clang-gnu  - Clang Compiler with LLVM Backend");
                        println!("    clang-msvc - Clang/LLVM Compatible with MSVC Toolchain");
                    } else {
                        println!("    clang      - Clang Compiler with LLVM Backend");
                    }
                } else {
                    println!("    clang      - unavailable");
                }
                if std::process::Command::new("cl.exe").output().is_ok() {
                    println!("    msvc       - MSVC, Microsoft Visual C/C++ Compiler for Windows ");
                } else {
                    println!("    msvc       - unavailable");
                }
                if std::process::Command::new("zig").output().is_ok() {
                    println!("    zig        - Zig Wrapper for Clang/LLVM Compiler");
                } else {
                    println!("    zig        - unavailable");
                }
            }
            _ => (),
        }
    } else {
        println!("VanGo {} - C/C++ build automation tool", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Usage: vango [ACTION] [OPTIONS]");
        println!();
        println!("Commands:");
        println!("    new         Create a new empty project");
        println!("    init        Create a new empty project in an existing location");
        println!("    help        Display help about a command (list toolchains with 'help toolchains')");
        println!("    clean, c    Remove all build files of the current project");
        println!("    build, b    Build the current project");
        println!("    run,   r    Build the current project and run it");
        println!("    test,  t    Build the current project and test it");
        println!("    clangd      Generate 'compile_flags.txt for the current project");
    }
    println!();
}


pub fn new(library: bool, is_c: bool, clangd: bool, name: &str) -> Result<(), Error> {
    std::fs::create_dir(name)?;
    std::env::set_current_dir(name)?;
    init(library, is_c, clangd)
}


pub fn init(library: bool, is_c: bool, clangd: bool) -> Result<(), Error> {
    let name = std::env::current_dir().unwrap().file_name().unwrap().to_string_lossy().to_string();
    log_info_ln!("creating new {} project: {}", if library { "library" } else { "application" }, name);
    let ext =    if is_c { "c" } else { "cpp" };
    let lang =   if is_c { "c11" } else { "c++17" };
    let header = if is_c { "stdio.h" } else { "cstdio" };
    std::fs::create_dir("src")?;
    if library {
        std::fs::create_dir_all(format!("include/{name}"))?;
        std::fs::write(format!("include/{name}/lib.h"), if is_c {
            "#ifndef LIB_H\n#define LIB_H\n\nint func(int a, int b);\n\n#endif"
        } else {
            "#pragma once\n\nint func(int a, int b);\n"
        })?;
        let toml = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nlang = \"{lang}\"\nkind = \"staticlib\"\ninclude = [ \"src\", \"include/{name}\" ]\ninclude-pub = \"include\"\n\n[dependencies]\n");
        std::fs::write(format!("src/lib.{ext}"), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}\n")?;
        std::fs::write("Vango.toml", &toml)?;
        if clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            generate(&build.unwrap_build())?;
        }
    } else {
        let toml = format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nlang = \"{lang}\"\n\n[dependencies]\n");
        std::fs::write(format!("src/main.{ext}"), format!("#include <{header}>\n\n\nint main() {{\n    printf(\"Hello World!\\n\");\n}}\n"))?;
        std::fs::write("Vango.toml", &toml)?;
        if clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            generate(&build.unwrap_build())?;
        }
    }
    log_info_ln!("successfully created project '{name}'");
    Ok(())
}


pub fn clean(build: &BuildFile) -> Result<(), Error> {
    log_info_ln!("cleaning build files for \"{}\"", build.name);
    std::fs::remove_dir_all("bin/debug/")?;
    std::fs::remove_dir_all("bin/release/")?;
    Ok(())
}


pub fn generate(build: &BuildFile) -> Result<(), Error> {
    log_info_ln!("generating 'compile_flags.txt' for \"{}\"", build.name);
    let mut flags = format!("-std={}\n{}", build.lang, if build.lang.is_cpp() { "-xc++\n" } else { "" });

    let profile = build.get(&Profile::Debug)?;
    match profile.settings.warn_level {
        WarnLevel::None => {
            flags.push_str("-w\n");
            if profile.settings.iso_compliant {
                flags.push_str("-Wpedantic\n");
            }
        }
        WarnLevel::Basic => {
            flags.push_str("-Wall\n");
            if profile.settings.iso_compliant {
                flags.push_str("-Wpedantic\n");
            }
        }
        WarnLevel::High => {
            flags.push_str("-Wall\n");
            flags.push_str("-Wextra\n");
            flags.push_str("-Wpedantic\n");
            flags.push_str("-Wconversion\n");
            flags.push_str("-Wsign-conversion\n");
            flags.push_str("-Wshadow\n");
            flags.push_str("-Wformat=2\n");
            flags.push_str("-Wnull-dereference\n");
            flags.push_str("-Wdouble-promotion\n");
            flags.push_str("-Wimplicit-fallthrough\n");
        }
    }

    let mut defines = Vec::new();
    let mut incdirs = Vec::new();

    for lib in &build.dependencies {
        let path = match lib {
            Dependency::Local { path, .. } => {
                path.clone()
            }
            #[allow(unused)]
            Dependency::Git { git, tag, .. } => {
                continue;
            }
            Dependency::Headers { headers, .. } => {
                incdirs.push(headers.clone());
                continue;
            }
            Dependency::System{..} => continue,
        };

        if !std::fs::exists(&path).unwrap() {
            return Err(Error::DirectoryNotFound(path))
        }

        if let Some(build) = if cfg!(windows) && std::fs::exists(path.join("win.vango.toml"))? {
            std::fs::read_to_string(path.join("win.vango.toml")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(path.join("lnx.vango.toml"))? {
            std::fs::read_to_string(path.join("lnx.vango.toml")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(path.join("mac.vango.toml"))? {
            std::fs::read_to_string(path.join("mac.vango.toml")).ok()
        } else {
            std::fs::read_to_string(path.join("vango.toml")).ok()
        } {
            match VangoFile::from_str(&build)? {
                VangoFile::Build(build) => {
                    let mut libinfo = LibFile::try_from(build)?;
                    let profile = libinfo.take(&Profile::Debug)?;
                    defines.extend(profile.defines);
                    incdirs.push(path.join(profile.include));
                }
                VangoFile::Lib(mut lib) => {
                    let profile = lib.take(&Profile::Debug)?;
                    defines.extend(profile.defines);
                    incdirs.push(path.join(profile.include));
                }
            }
        } else {
            return Err(Error::MissingBuildScript(path))
        }
    }

    if cfg!(windows) {
        defines.push("UNICODE".to_string());
        defines.push("_UNICODE".to_string());
        if let ProjKind::SharedLib{..} = build.kind {
            defines.push("VANGO_EXPORT_SHARED".to_string());
        }
    }
    for dep in defines {
        flags.push_str(&format!("-D{dep}\n"));
    }
    for inc in incdirs {
        flags.push_str(&format!("-I{}\n", inc.display()));
    }
    for inc in &profile.include {
        flags.push_str(&format!("-I{}\n", inc.display()));
    }

    std::fs::write("compile_flags.txt", flags)?;
    Ok(())
}

