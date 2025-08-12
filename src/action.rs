use std::{io::Write, path::PathBuf};
use crate::{
    error::Error,
    exec::{self, BuildInfo},
    fetch::{self, FileInfo},
    input::BuildSwitches, repr::*,
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
        let json = format!("{{\n    \"project\": \"{name}\",\n    \"lang\": \"{lang}\",\n    \"dependencies\": [],\n    \"incdirs\": [ \"src/\", \"include/{name}\" ],\n    \"include-public\": \"include/\"\n}}");
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={lang}\n{}-DDEBUG\n-Isrc\n-Iinclude/{name}",
            if !is_c { "-xc++\n" } else { "" });
        std::fs::write(format!("src/lib.{ext}"), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}\n")?;
        std::fs::write("build.json", json)?;
        std::fs::write("compile_flags.txt", flags)?;
    } else {
        let json = format!("{{\n    \"project\": \"{name}\",\n    \"lang\": \"{lang}\",\n    \"dependencies\": []\n}}");
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={lang}\n{}-DDEBUG\n-Isrc",
            if !is_c { "-xc++\n" } else { "" });
        std::fs::write(format!("src/main.{ext}"), format!("#include <{header}>\n\n\nint main() {{\n    printf(\"Hello World!\\n\");\n}}\n"))?;
        std::fs::write("build.json", json)?;
        std::fs::write("compile_flags.txt", flags)?;
    }
    log_info!("successfully created project '{name}'");
    Ok(())
}


pub fn build(build: BuildFile, switches: BuildSwitches, test: bool) -> Result<(bool, String), Error> {
    let mut headers = fetch::source_files(&PathBuf::from(&build.srcdir), ".h").unwrap();
    for incdir in build.incdirs.iter().chain(&build.include_public) {
        headers.extend(fetch::source_files(&PathBuf::from(incdir), ".h").unwrap());
    }
    let projkind = if headers.iter().any(|f| f.file_name() == "lib.h") { ProjKind::Lib } else { ProjKind::App };
    let lang: Lang = build.lang.parse()?;

    let mut deps = fetch::libraries(build.dependencies, switches, lang)?;
    deps.defines.extend(build.defines);
    if test { deps.defines.push("TEST".to_string()); }
    deps.incdirs.extend(build.incdirs);

    let rebuilt_dep = deps.rebuilt;
    let outpath = if projkind == ProjKind::App {
        format!("bin/{}/{}{}", switches.config, build.project, switches.toolchain.app_ext())
    } else {
        format!("bin/{}/{}{}{}", switches.config, switches.toolchain.lib_prefix(), build.project, switches.toolchain.lib_ext())
    };
    let outfile = FileInfo::from_str(&outpath);

    let info = BuildInfo {
        projkind,
        toolchain: switches.toolchain,
        config: switches.config,
        lang,
        crtstatic: switches.crtstatic,

        sources: fetch::source_files(&PathBuf::from(&build.srcdir), lang.src_ext()).unwrap(),
        headers,
        relink: deps.relink,
        srcdir: build.srcdir,
        outdir: format!("bin/{}/", switches.config),
        outfile,
        defines: deps.defines,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,

        comp_args: build.compiler_options,
        link_args: build.linker_options,
    };
    match exec::run_build(info, switches.verbose) {
        Err(e) => Err(e),
        Ok(rebuilt) => Ok((rebuilt_dep || rebuilt, outpath)),
    }
}


pub fn clean(build: BuildFile) -> Result<(), Error> {
    log_info!("cleaning build files for \"{}\"", build.project);
    let _ = std::fs::remove_dir_all("bin/debug/");
    let _ = std::fs::remove_dir_all("bin/release/");
    if let Some(pch) = build.pch {
        let _ = std::fs::remove_file(format!("src/{pch}.gch"));
    }
    Ok(())
}


pub fn help(action: Option<String>) {
    let print_build_details = || {
        println!("Options:");
        println!("  -d, --debug             Build project in debug profile (default)");
        println!("  -r, --release           Build project in release profile");
        println!("  -t, --toolchain=<TOOL>  Specify a toolchain for compilation (user default: {})", ToolChain::default());
        println!("      --crtstatic         Link statically with the C runtime library");
        println!("  -v, --verbose           Echo build command and complete compiler output");
        println!();
        println!("Profiles:");
        println!("    debug    Build with no optimization; Generate debug symbols; 'DEBUG' macro defined; Generally faster compile times;");
        println!("    release  Build with high optimization; 'RELEASE' macro defined; Generally slower compile times;");
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
                println!("Build the current project in test configuration, link it to your test app and run it");
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
fn check_outdated(build: BuildFile, switches: BuildSwitches, test: bool) -> Result<bool, Error> {
    let mut headers = fetch::source_files(&PathBuf::from(&build.srcdir), ".h").unwrap();
    for incdir in build.incdirs.iter().chain(&build.include_public) {
        headers.extend(fetch::source_files(&PathBuf::from(incdir), ".h").unwrap());
    }
    let projkind = if headers.iter().any(|f| f.file_name() == "lib.h") { ProjKind::Lib } else { ProjKind::App };
    let lang: Lang = build.lang.parse()?;

    let mut deps = fetch::libraries(build.dependencies, switches, lang)?;
    deps.defines.extend(build.defines);
    if test {
        deps.defines.push("TEST".to_string());
    }
    deps.incdirs.extend(build.incdirs);

    let rebuilt_dep = deps.rebuilt;
    let outpath = if projkind == ProjKind::App {
        format!("bin/{}/{}{}", switches.config, build.project, switches.toolchain.app_ext())
    } else {
        format!("bin/{}/{}{}{}", switches.config, switches.toolchain.lib_prefix(), build.project, switches.toolchain.lib_ext())
    };
    let outfile = FileInfo::from_str(&outpath);

    let info = BuildInfo{
        projkind,
        toolchain: switches.toolchain,
        config: switches.config,
        lang,
        crtstatic: switches.crtstatic,

        sources: fetch::source_files(&PathBuf::from(&build.srcdir), lang.src_ext()).unwrap(),
        headers,
        relink: deps.relink,
        srcdir: build.srcdir,
        outdir: format!("bin/{}/obj/", switches.config),
        outfile: outfile.clone(),
        defines: deps.defines,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,

        comp_args: build.compiler_options,
        link_args: build.linker_options,
    };
    let rebuilt = exec::run_check_outdated(info)?;
    Ok(rebuilt_dep || rebuilt)
}

