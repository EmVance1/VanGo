use std::{io::Write, path::PathBuf};
use crate::{
    error::Error,
    exec::{self, BuildInfo},
    input::BuildSwitches, config::*,
    fetch,
};


pub fn build(mut build: BuildFile, switches: BuildSwitches) -> Result<(bool, PathBuf), Error> {
    let profile = build.take(&switches.profile)?;
    let mut headers = fetch::source_files(&profile.include_pub, "h").unwrap();
    if build.build.lang.is_cpp() {
        headers.extend(fetch::source_files(&profile.include_pub, "hpp").unwrap());
    }
    for incdir in profile.include.iter().chain(Some(&profile.src)) {
        headers.extend(fetch::source_files(incdir, "h").unwrap());
        if build.build.lang.is_cpp() {
            headers.extend(fetch::source_files(incdir, "hpp").unwrap());
        }
    }
    let sources = fetch::source_files(&profile.src, build.build.lang.src_ext()).unwrap();

    let mut deps = fetch::libraries(build.dependencies, &switches, build.build.lang)?;
    deps.defines.extend(profile.defines);
    if switches.is_test { deps.defines.push("VANGO_TEST".to_string()); }
    if cfg!(target_os = "windows") {
        deps.defines.push("UNICODE".to_string());
        deps.defines.push("_UNICODE".to_string());
        if let ProjKind::SharedLib{..} = build.build.kind {
            deps.defines.push("VANGO_EXPORT_SHARED".to_string());
        }
    }
    deps.incdirs.extend(profile.include);

    let rebuilt_dep = deps.rebuilt;
    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let (outfile, implib) = match build.build.kind {
        ProjKind::App => {
            (outdir.join(build.build.package).with_extension(switches.toolchain.app_ext()), None)
        }
        ProjKind::SharedLib{implib: false} => {
            (outdir.join(format!("{}{}", switches.toolchain.shared_lib_prefix(), build.build.package))
             .with_extension(switches.toolchain.shared_lib_ext()), None)
        }
        ProjKind::SharedLib{implib: true} => {
            (outdir.join(format!("{}{}", switches.toolchain.shared_lib_prefix(), build.build.package))
             .with_extension(switches.toolchain.shared_lib_ext()),
             Some(outdir.join(format!("{}{}", switches.toolchain.static_lib_prefix(), build.build.package))
             .with_extension(switches.toolchain.static_lib_ext())))
        }
        ProjKind::StaticLib => {
            (outdir.join(format!("{}{}", switches.toolchain.static_lib_prefix(), build.build.package))
             .with_extension(switches.toolchain.static_lib_ext()), None)
        }
    };

    let info = BuildInfo{
        projkind:  build.build.kind,
        toolchain: switches.toolchain,
        profile:   switches.profile,
        lang:      build.build.lang,
        crtstatic: switches.crtstatic,
        cpprt:     build.build.runtime.map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),

        defines:  deps.defines,

        srcdir:   profile.src,
        incdirs:  deps.incdirs,
        libdirs:  deps.libdirs,
        outdir,

        pch:      profile.pch,
        sources,
        headers,
        archives: deps.archives,
        relink:   deps.relink,
        outfile:  outfile.clone(),
        implib:   implib,

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
        println!("    help        Display detailed information about a command");
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
        let toml = format!("[build]\npackage = \"{name}\"\nversion = \"0.1.0\"\nlang = \"{lang}\"\nkind = \"staticlib\"\ninclude = [ \"src\", \"include/{name}\" ]\ninclude-pub = \"include\"\n\n[dependencies]\n");
        std::fs::write(format!("src/lib.{ext}"), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}\n")?;
        std::fs::write("Vango.toml", &toml)?;
        if clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            generate(build.unwrap_build())?;
        }
    } else {
        let toml = format!("[build]\npackage = \"{name}\"\nversion = \"0.1.0\"\nlang = \"{lang}\"\n\n[dependencies]\n");
        std::fs::write(format!("src/main.{ext}"), format!("#include <{header}>\n\n\nint main() {{\n    printf(\"Hello World!\\n\");\n}}\n"))?;
        std::fs::write("Vango.toml", &toml)?;
        if clangd {
            let build = VangoFile::from_str(&toml).unwrap();
            generate(build.unwrap_build())?;
        }
    }
    log_info_ln!("successfully created project '{name}'");
    Ok(())
}


pub fn clean(build: BuildFile) -> Result<(), Error> {
    log_info_ln!("cleaning build files for \"{}\"", build.build.package);
    let _ = std::fs::remove_dir_all("bin/debug/");
    let _ = std::fs::remove_dir_all("bin/release/");
    Ok(())
}


pub fn generate(build: BuildFile) -> Result<(), Error> {
    log_info_ln!("generating 'compile_flags.txt' for \"{}\"", build.build.package);
    let mut flags = format!(
"-Wall
-Wextra
-Wshadow
-Wconversion
-Wfloat-equal
-Wno-unused-const-variable
-Wno-sign-conversion
-std={}
{}-DVANGO_DEBUG\n",
        build.build.lang, if build.build.lang.is_cpp() { "-xc++\n" } else { "" });

    let mut incdirs = Vec::new();
    let mut defines = Vec::new();

    for lib in build.dependencies {
        let path = match lib {
            #[allow(unused)]
            Dependency::Local { path, features } => {
                path
            }
            #[allow(unused)]
            Dependency::Git { git, tag, recipe, features } => {
                continue;
            }
            #[allow(unused)]
            Dependency::Headers { headers, features } => {
                incdirs.push(headers);
                continue;
            }
        };

        if !std::fs::exists(&path).unwrap() {
            return Err(Error::DirectoryNotFound(path))
        }

        if let Some(build) = if cfg!(target_os = "windows") && std::fs::exists(path.join("win.vango.toml"))? {
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
                    let mut libinfo = LibFile::try_from(build).unwrap();
                    let profile = libinfo.take(&Profile::Debug)?;
                    incdirs.push(path.join(profile.include));
                    defines.extend(profile.defines);
                }
                VangoFile::Lib(mut lib) => {
                    let profile = lib.take(&Profile::Debug)?;
                    incdirs.push(path.join(profile.include));
                    defines.extend(profile.defines);
                }
            }
        } else {
            return Err(Error::MissingBuildScript(path))
        }
    }

    for inc in incdirs {
        flags.push_str(&format!("-I{}\n", inc.display()));
    }
    flags.push_str(&format!("-I{}\n", build.profile.get("debug").unwrap().src.display()));


    std::fs::write("compile_flags.txt", flags)?;
    Ok(())
}

