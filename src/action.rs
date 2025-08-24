use std::{io::Write, path::PathBuf};
use crate::{
    error::Error,
    exec::{self, BuildInfo},
    input::BuildSwitches, config::*,
    fetch,
};


pub fn new(library: bool, is_c: bool, name: &str) -> Result<(), Error> {
    std::fs::create_dir(name)?;
    std::env::set_current_dir(name)?;
    init(library, is_c)
}


pub fn init(library: bool, is_c: bool) -> Result<(), Error> {
    let name = std::env::current_dir().unwrap().file_name().unwrap().to_string_lossy().to_string();
    log_info!("creating new {} project: {}", if library { "library" } else { "application" }, name);
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
        let toml = format!("[build]\npackage = \"{name}\"\nversion = 0.1.0\nlang = \"{lang}\"\ninclude = [ \"src\", \"include/{name}\" ]\ninclude-pub = \"include\"\n\n[dependencies]\n");
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={lang}\n{}-DDEBUG\n-Isrc\n-Iinclude/{name}",
            if !is_c { "-xc++\n" } else { "" });
        std::fs::write(format!("src/lib.{ext}"), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}\n")?;
        std::fs::write("vango.toml", toml)?;
        std::fs::write("compile_flags.txt", flags)?;
    } else {
        let toml = format!("[build]\npackage = \"{name}\"\nversion = 0.1.0\nlang = \"{lang}\"\n\n[dependencies]\n");
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={lang}\n{}-DDEBUG\n-Isrc",
            if !is_c { "-xc++\n" } else { "" });
        std::fs::write(format!("src/main.{ext}"), format!("#include <{header}>\n\n\nint main() {{\n    printf(\"Hello World!\\n\");\n}}\n"))?;
        std::fs::write("vango.toml", toml)?;
        std::fs::write("compile_flags.txt", flags)?;
    }
    log_info!("successfully created project '{name}'");
    Ok(())
}


pub fn build(mut build: BuildFile, switches: BuildSwitches, test: bool) -> Result<(bool, PathBuf), Error> {
    let profile = build.take(&switches.profile)?;
    let mut headers = fetch::source_files(&profile.include_pub, ".h").unwrap();
    for incdir in profile.include.iter().chain(Some(&profile.src)) {
        headers.extend(fetch::source_files(incdir, ".h").unwrap());
    }
    let sources = fetch::source_files(&profile.src, build.build.lang.src_ext()).unwrap();
    let projkind = if headers.iter().any(|f| f.file_name().unwrap() == "lib.h") { ProjKind::Lib } else { ProjKind::App };

    let mut deps = fetch::libraries(build.dependencies, &switches, build.build.lang)?;
    deps.defines.extend(profile.defines);
    if test { deps.defines.push("VANGO_TEST".to_string()); }
    deps.incdirs.extend(profile.include);

    let rebuilt_dep = deps.rebuilt;
    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let outfile = if projkind == ProjKind::App {
        outdir.join(build.build.package).with_extension(switches.toolchain.app_ext())
    } else {
        outdir.join(format!("{}{}", switches.toolchain.lib_prefix(), build.build.package)).with_extension(switches.toolchain.lib_ext())
    };

    let info = BuildInfo{
        projkind,
        toolchain: switches.toolchain,
        profile:   switches.profile,
        lang:      build.build.lang,
        crtstatic: switches.crtstatic,

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

        comp_args: profile.compiler_options,
        link_args: profile.linker_options,
    };
    match exec::run_build(info, switches.echo, switches.verbose) {
        Err(e) => Err(e),
        Ok(rebuilt) => Ok((rebuilt_dep || rebuilt, outfile)),
    }
}


pub fn clean(build: BuildFile) -> Result<(), Error> {
    log_info!("cleaning build files for \"{}\"", build.build.package);
    let _ = std::fs::remove_dir_all("bin/debug/");
    let _ = std::fs::remove_dir_all("bin/release/");
    Ok(())
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
    }
    println!();
}


#[allow(unused)]
fn check_outdated(mut build: BuildFile, switches: BuildSwitches, test: bool) -> Result<bool, Error> {
    let profile = build.take(&switches.profile)?;
    let sources = fetch::source_files(&profile.src, build.build.lang.src_ext()).unwrap();
    let mut headers = fetch::source_files(&profile.src, ".h").unwrap();
    for incdir in profile.include.iter().chain(Some(&profile.include_pub)) {
        headers.extend(fetch::source_files(incdir, ".h").unwrap());
    }
    let projkind = if headers.iter().any(|f| f.file_name().unwrap() == "lib.h") { ProjKind::Lib } else { ProjKind::App };

    let mut deps = fetch::libraries(build.dependencies, &switches, build.build.lang)?;
    deps.defines.extend(profile.defines);
    if test { deps.defines.push("VANGO_TEST".to_string()); }
    deps.incdirs.extend(profile.include);

    let rebuilt_dep = deps.rebuilt;
    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let outfile = if projkind == ProjKind::App {
        outdir.join(build.build.package).with_extension(switches.toolchain.app_ext())
    } else {
        outdir.join(format!("{}{}", switches.toolchain.lib_prefix(), build.build.package)).with_extension(switches.toolchain.lib_ext())
    };

    let info = BuildInfo{
        projkind,
        toolchain: switches.toolchain,
        profile:   switches.profile,
        lang:      build.build.lang,
        crtstatic: switches.crtstatic,

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

        comp_args: profile.compiler_options,
        link_args: profile.linker_options,
    };
    let rebuilt = exec::run_check_outdated(info)?;
    Ok(rebuilt_dep || rebuilt)
}

