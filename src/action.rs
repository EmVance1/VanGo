use crate::{
    exec::{self, BuildInfo},
    fetch::{self, FileInfo},
    repr::*,
    error::Error,
};
use std::{io::Write, path::PathBuf};


pub fn new(library: bool, is_c: bool, name: &str) -> Result<(), Error> {
    std::fs::create_dir(name)?;
    std::env::set_current_dir(name)?;
    init(library, is_c)
}


pub fn init(library: bool, is_c: bool) -> Result<(), Error> {
    let name = std::env::current_dir().unwrap().file_name().unwrap().to_string_lossy().to_string();
    log_info!("creating new {} project: {}", if library { "library" } else { "application" }, name);
    let ext =    if is_c { "c" } else { "cpp" };
    let cstd =   if is_c { "C11" } else { "C++17" };
    let header = if is_c { "stdio.h" } else { "cstdio" };
    std::fs::create_dir("src")?;
    if library {
        std::fs::create_dir_all(format!("include/{name}"))?;
        std::fs::write(format!("include/{name}/lib.h"), if is_c {
            "#ifndef LIB_H\n#define LIB_H\n\nint func(int a, int b);\n\n#endif"
        } else {
            "#pragma once\n\nint func(int a, int b);\n"
        })?;
        let json = format!("{{\n    \"project\": \"{name}\",\n    \"cpp\": \"{cstd}\",\n    \"dependencies\": [],\n    \"incdirs\": [ \"src/\", \"include/{name}\" ],\n    \"include-public\": \"include/\"\n}}");
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={cstd}\n{}-DDEBUG\n-Isrc\n-Iinclude/{name}",
            if !is_c { "-xc++\n" } else { "" });
        std::fs::write(format!("src/lib.{ext}"), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}\n")?;
        std::fs::write("build.json", json)?;
        std::fs::write("compile_flags.txt", flags)?;
    } else {
        let json = format!("{{\n    \"project\": \"{name}\",\n    \"cpp\": \"{cstd}\",\n    \"dependencies\": []\n}}");
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={cstd}\n{}-DDEBUG\n-Isrc",
            if !is_c { "-xc++\n" } else { "" });
        std::fs::write(format!("src/main.{ext}"), format!("#include <{header}>\n\n\nint main() {{\n    printf(\"Hello World!\\n\");\n}}\n"))?;
        std::fs::write("build.json", json)?;
        std::fs::write("compile_flags.txt", flags)?;
    }
    log_info!("successfully created project '{name}'");
    Ok(())
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


pub fn build(build: BuildFile, config: Config, toolchain: ToolChain, verbose: bool, test: bool) -> Result<(bool, String), Error> {
    let mut headers = fetch::source_files(&PathBuf::from(&build.srcdir), ".h").unwrap();
    for incdir in build.incdirs.iter().chain(&build.include_public) {
        headers.extend(fetch::source_files(&PathBuf::from(incdir), ".h").unwrap());
    }
    let projkind = if headers.iter().any(|f| f.file_name() == "lib.h") { ProjKind::Lib } else { ProjKind::App };
    let lang = Lang::try_from(&build.lang)?;

    let mut deps = fetch::libraries(build.dependencies, config, toolchain, &build.lang)?;
    deps.defines.extend(build.defines);
    if test { deps.defines.push("TEST".to_string()); }
    deps.incdirs.extend(build.incdirs);

    let rebuilt_dep = deps.rebuilt;
    let outpath = if projkind == ProjKind::App {
        format!("bin/{}/{}{}", config, build.project, toolchain.app_ext())
    } else {
        format!("bin/{}/{}{}{}", config, toolchain.lib_prefix(), build.project, toolchain.lib_ext())
    };
    let outfile = FileInfo::from_str(&outpath);

    let info = BuildInfo {
        projkind,
        toolchain,
        config,
        lang,

        sources: fetch::source_files(&PathBuf::from(&build.srcdir), lang.src_ext()).unwrap(),
        headers,
        relink: deps.relink,
        srcdir: build.srcdir,
        outdir: format!("bin/{config}/obj/"),
        outfile: outfile.clone(),
        defines: deps.defines,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,

        comp_args: build.compiler_options,
        link_args: build.linker_options,
    };
    match exec::run_build(info, verbose) {
        Err(e) => Err(e),
        Ok(rebuilt) => Ok((rebuilt_dep || rebuilt, outpath)),
    }
}


pub fn help(action: Option<String>) {
    if let Some(action) = action {
        match action.as_str() {
            "new" => {
                println!("Create a new folder with a boilerplate C++ application");
                println!();
                println!("Usage: vango new <NAME> [OPTIONS]");
                println!();
                println!("Options:");
                println!("    --lib  Generate library boilerplate instead of application");
                println!("    --c    Generate C boilerplate instead of C++");
            }
            "init" => {
                println!("Create a boilerplate C++ application inside an existing folder");
                println!();
                println!("Usage: vango init [OPTIONS]");
                println!();
                println!("Options:");
                println!("    --lib  Generate library boilerplate instead of application");
                println!("    --c    Generate C boilerplate instead of C++");
            }
            "clean" => {
                println!("Remove all generated build files from the current project");
            }
            "build" => {
                println!("Build the current project");
                println!();
                println!("Usage: vango build [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -d, --debug    Generate library boilerplate instead of application");
                println!("  -r, --release  Generate library boilerplate instead of application");
                println!("  -v, --verbose  Echo build command and entire compiler output");
                println!("  -t=<TOOL>      Specify a toolchain for compilation");
            }
            "run" => {
                println!("Build and run the current project, with the working directory as the project root, and forwarding commandline arguments");
                println!();
                println!("Usage: vango run [OPTIONS] [-- ARGS]");
                println!();
                println!("Options:");
                println!("  -d, --debug    Generate library boilerplate instead of application");
                println!("  -r, --release  Generate library boilerplate instead of application");
                println!("  -v, --verbose  Echo build command and entire compiler output");
                println!("  -t=<TOOL>      Specify a toolchain for compilation");
            }
            "test" => {
                println!("Build the current project in test configuration, link it to your test app and run it");
                println!();
                println!("Usage: vango test [OPTIONS] [TESTS]");
                println!();
                println!("Options:");
                println!("  -d, --debug    Generate library boilerplate instead of application");
                println!("  -r, --release  Generate library boilerplate instead of application");
                println!("  -v, --verbose  Echo build command and entire compiler output");
                println!("  -t=<TOOL>      Specify a toolchain for compilation");
            }
            _ => (),
        }
    } else {
        println!("C/C++ build automation tool");
        println!();
        println!("Usage: vango [ACTION] [OPTIONS]");
        println!();
        println!("Commands:");
        println!("    new         Create a new empty project");
        println!("    init        Create a new empty project in an existing location");
        println!("    clean, c    Remove all build files of the current project");
        println!("    build, b    Build the current project");
        println!("    run,   r    Build the current project and run it");
        println!("    test,  t    Build the current project and test it");
    }
    println!();
}


#[allow(unused)]
fn check_outdated(build: BuildFile, config: Config, toolchain: ToolChain, test: bool) -> Result<bool, Error> {
    let mut headers = fetch::source_files(&PathBuf::from(&build.srcdir), ".h").unwrap();
    for incdir in build.incdirs.iter().chain(&build.include_public) {
        headers.extend(fetch::source_files(&PathBuf::from(incdir), ".h").unwrap());
    }
    let projkind = if headers.iter().any(|f| f.file_name() == "lib.h") { ProjKind::Lib } else { ProjKind::App };
    let lang = Lang::try_from(&build.lang)?;

    let mut deps = fetch::libraries(build.dependencies, config, toolchain, &build.lang)?;
    deps.defines.extend(build.defines);
    if test {
        deps.defines.push("TEST".to_string());
    }
    deps.incdirs.extend(build.incdirs);

    let rebuilt_dep = deps.rebuilt;
    let outpath = if projkind == ProjKind::App {
        format!("bin/{}/{}{}", config, build.project, toolchain.app_ext())
    } else {
        format!("bin/{}/{}{}{}", config, toolchain.lib_prefix(), build.project, toolchain.lib_ext())
    };
    let outfile = FileInfo::from_str(&outpath);

    let info = BuildInfo{
        projkind,
        toolchain,
        config,
        lang,

        sources: fetch::source_files(&PathBuf::from(&build.srcdir), lang.src_ext()).unwrap(),
        headers,
        relink: deps.relink,
        srcdir: build.srcdir,
        outdir: format!("bin/{config}/obj/"),
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

