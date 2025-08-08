mod input;
mod repr;
mod fetch;
mod exec;
mod testfw;
mod error;
#[macro_use]
mod log;

use std::{ io::Write, path::PathBuf };
use repr::*;
use fetch::FileInfo;
use exec::BuildInfo;
use error::Error;



fn action_new(name: &str, library: bool, isc: bool) -> Result<(), Error> {
    log_info!("creating new {} project: {}", if library { "library" } else { "application" }, name);
    std::fs::create_dir(name).unwrap();
    std::fs::create_dir(format!("{}/src", name)).unwrap();
    let ext = if isc { "c" } else { "cpp" };
    let header = if isc { "stdio.h" } else { "cstdio" };
    let cstd = if isc { "c11" } else { "c++17" };
    if library {
        std::fs::create_dir(format!("{}/include", name)).unwrap();
        std::fs::create_dir(format!("{}/include/{}", name, name)).unwrap();
        if isc {
            std::fs::write(format!("{}/include/{}/lib.h", name, name), "#ifndef LIB_H\n#define LIB_H\n\nint func(int a, int b);\n\n#endif").unwrap();
        } else {
            std::fs::write(format!("{}/include/{}/lib.h", name, name), "#pragma once\n\nint func(int a, int b);\n").unwrap();
        }
        std::fs::write(format!("{}/src/lib.{ext}", name), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}\n").unwrap();
        let json = format!("{{\n    \"project\": \"{}\",\n    \"cpp\": \"{cstd}\",\n    \"dependencies\": [],\n    \"incdirs\": [ \"src/\", \"include/{}\" ],\n    \"include-public\": \"include/\"\n}}", name, name);
        std::fs::write(format!("{}/build.json", name), json).unwrap();
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={cstd}\n{}-DDEBUG\n-Isrc\n-Iinclude/{}",
                    if !isc { "-xc++\n" } else { "" }, name
                );
        std::fs::write(format!("{}/compile_flags.txt", name), flags).unwrap();
    } else {
        std::fs::write(format!("{}/src/main.{ext}", name), format!("#include <{header}>\n\n\nint main() {{\n    printf(\"Hello World!\\n\");\n}}\n")).unwrap();
        let json = format!("{{\n    \"project\": \"{}\",\n    \"cpp\": \"{cstd}\",\n    \"dependencies\": []\n}}", name);
        std::fs::write(format!("{}/build.json", name), json).unwrap();
        let flags = format!(
            "-Wall\n-Wextra\n-Wshadow\n-Wconversion\n-Wfloat-equal\n-Wno-unused-const-variable\n-Wno-sign-conversion\n-std={cstd}\n{}-DDEBUG\n-Isrc",
                    if !isc { "-xc++\n" } else { "" }
                );
        std::fs::write(format!("{}/compile_flags.txt", name), flags).unwrap();
    }
    log_info!("successfully created project '{}'", name);
    Ok(())
}

fn action_clean(build: BuildFile) -> Result<(), Error> {
    log_info!("cleaning build files for \"{}\"", build.project);
    let _ = std::fs::remove_dir_all("bin/debug/");
    let _ = std::fs::remove_dir_all("bin/release/");
    if let Some(pch) = build.pch {
        let _ = std::fs::remove_file(format!("src/{}.gch", pch));
    }
    Ok(())
}

fn action_build(build: BuildFile, config: Config, mingw: bool, test: bool) -> Result<(bool, String), Error> {
    let kind = fetch::get_project_kind(&build.srcdir, &build.inc_public)?;

    let _ = repr::u32_from_cppstd(&build.cpp)?;

    let toolset = if cfg!(target_os = "windows") && !mingw {
        ToolSet::MSVC
    } else if cfg!(target_os = "linux") || mingw {
        ToolSet::GNU
    } else {
        ToolSet::CLANG
    };

    let mut deps = fetch::get_libraries(build.dependencies, config, mingw, &build.cpp)?;
    deps.defines.extend(build.defines);
    if test {
        deps.defines.push("TEST".to_string());
    }
    let rebuilt_dep = deps.rebuilt;
    let outpath = if let ProjKind::App = kind {
        format!("bin/{}/{}{}", config, build.project, kind.ext(mingw))
    } else {
        format!("bin/{}/{}{}{}", config, toolset.lib_prefix(), build.project, toolset.ext(kind))
    };
    let outfile = FileInfo::from_str(&outpath);

    let mut headers = fetch::get_source_files(&PathBuf::from(&build.srcdir), ".h").unwrap();
    if let Some(inc) = build.inc_public {
        headers.extend(fetch::get_source_files(&PathBuf::from(&inc), ".h").unwrap());
    }
    for dep in &build.incdirs {
        headers.extend(fetch::get_source_files(&PathBuf::from(dep), ".h").unwrap());
    }
    deps.incdirs.extend(build.incdirs);

    let cppstd = build.cpp.to_ascii_lowercase();
    let is_c = !cppstd.starts_with("c++");

    let info = BuildInfo{
        sources: fetch::get_source_files(&PathBuf::from(&build.srcdir), if is_c { ".c" } else { ".cpp" }).unwrap(),
        headers,
        relink: deps.relink,
        srcdir: build.srcdir,
        outdir: format!("bin/{}/obj/", config),
        outfile: outfile.clone(),
        defines: deps.defines,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,
        cppstd,
        is_c,
        config,
        toolset,
        comp_args: build.compiler_options,
        link_args: build.linker_options,
    };
    match exec::run_build(info) {
        Err(e) => Err(e),
        Ok(rebuilt) => Ok((rebuilt_dep || rebuilt, outpath)),
    }
}


macro_rules! exit_with {
    () => { { eprintln!(); std::process::exit(1); } };
    ($($arg:tt)*) => { {
        log_error!($($arg)*);
        std::process::exit(1);
    } };
}


fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().collect();
    let cmd = input::parse_input(args).unwrap_or_else(|e| exit_with!("{}", e));

    if let input::Action::New{ name, library, isc } = &cmd {
        action_new(name, *library, *isc).unwrap_or_else(|e| exit_with!("{}", e));
        0.into()
    } else if let input::Action::Set{ .. } = &cmd {
        0.into()
    } else {
        let bfile = if cfg!(target_os = "windows") && std::fs::exists("win.build.json").unwrap() {
            std::fs::read_to_string("win.build.json").unwrap()
        } else if cfg!(target_os = "linux") && std::fs::exists("linux.build.json").unwrap() {
            std::fs::read_to_string("linux.build.json").unwrap()
        } else if cfg!(target_os = "macos") && std::fs::exists("macos.build.json").unwrap() {
            std::fs::read_to_string("macos.build.json").unwrap()
        } else {
            std::fs::read_to_string("build.json")
                .map_err(|_| Error::FileNotFound("build.json".to_string()))
                .unwrap_or_else(|e| exit_with!("{}", e))
        };
        let build = BuildFile::from_str(&bfile)
            .map_err(Error::JsonParse)
            .unwrap_or_else(|e| exit_with!("{}", e));

        match cmd {
            input::Action::Clean => {
                action_clean(build).unwrap_or_else(|e| exit_with!("{}", e));
                0.into()
            }
            input::Action::Build{ config, mingw } => {
                let build = build.finalise(config);
                let (rebuilt, _) = action_build(build, config, mingw, false).unwrap_or_else(|e| exit_with!("{}", e));
                if rebuilt {
                    8.into()
                } else {
                    0.into()
                }
            }
            input::Action::Run{ config, mingw, args } => {
                let build = build.finalise(config);
                let (_, outfile) = action_build(build, config, mingw, false).unwrap_or_else(|e| exit_with!("{}", e));
                exec::run_app(&outfile, args).into()
            }
            input::Action::Test{ config, mingw, args } => {
                let build = build.finalise(config);
                action_build(build.clone(), config, mingw, true).unwrap_or_else(|e| exit_with!("{}", e));
                testfw::test_lib(build, config, mingw, args).unwrap_or_else(|e| exit_with!("{}", e));
                0.into()
            }
            _ => 0.into(),
        }
    }
}


#[allow(unused)]
fn action_check_outdated(build: BuildFile, config: Config, mingw: bool, test: bool) -> Result<bool, Error> {
    let kind = fetch::get_project_kind(&build.srcdir, &build.inc_public)?;

    let _ = repr::u32_from_cppstd(&build.cpp)?;

    let toolset = if cfg!(target_os = "windows") && !mingw {
        ToolSet::MSVC
    } else if cfg!(target_os = "linux") || mingw {
        ToolSet::GNU
    } else {
        ToolSet::CLANG
    };

    let mut deps = fetch::get_libraries(build.dependencies, config, mingw, &build.cpp)?;
    deps.defines.extend(build.defines);
    if test {
        deps.defines.push("TEST".to_string());
    }
    let rebuilt_dep = deps.rebuilt;
    let outpath = format!("bin/{}/{}{}", config, build.project, kind.ext(mingw));
    let outfile = FileInfo::from_str(&outpath);

    let mut headers = fetch::get_source_files(&PathBuf::from(&build.srcdir), ".h").unwrap();
    if let Some(inc) = build.inc_public {
        headers.extend(fetch::get_source_files(&PathBuf::from(&inc), ".h").unwrap());
    }
    for dep in &build.incdirs {
        headers.extend(fetch::get_source_files(&PathBuf::from(dep), ".h").unwrap());
    }
    deps.incdirs.extend(build.incdirs);

    let cppstd = build.cpp.to_ascii_lowercase();
    let is_c = !cppstd.starts_with("c++");

    let info = BuildInfo{
        sources: fetch::get_source_files(&PathBuf::from(&build.srcdir), if build.cpp.to_ascii_lowercase().starts_with("c++") { ".cpp" } else { ".c" }).unwrap(),
        headers,
        relink: deps.relink,
        srcdir: build.srcdir,
        outdir: format!("bin/{}/obj/", config),
        outfile: outfile.clone(),
        defines: deps.defines,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,
        cppstd,
        is_c,
        config,
        toolset,
        comp_args: build.compiler_options,
        link_args: build.linker_options,
    };
    let rebuilt = exec::run_check_outdated(info);
    Ok(rebuilt_dep || rebuilt)
}

