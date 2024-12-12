mod input;
mod repr;
mod fetch;
mod exec;
mod testfw;
mod error;
#[macro_use]
mod log;

use std::{
    io::Write,
    path::PathBuf,
};
use repr::*;
use fetch::FileInfo;
use exec::BuildInfo;
use error::Error;



fn action_new(name: &str, library: bool) -> Result<(), Error> {
    log_info!("creating new {} project: {}", if library { "library" } else { "application" }, name);
    std::fs::create_dir(name).unwrap();
    std::fs::create_dir(format!("{}/src", name)).unwrap();
    if library {
        std::fs::create_dir(format!("{}/include", name)).unwrap();
        std::fs::create_dir(format!("{}/include/{}", name, name)).unwrap();
        std::fs::write(format!("{}/src/lib.h", name),   "#pragma once\n\nint func(int a, int b);\n").unwrap();
        std::fs::write(format!("{}/src/lib.cpp", name), "#include \"lib.h\"\n\nint func(int a, int b) {\n    return a + b;\n}").unwrap();
        let json = format!("{{\n    \"project\": \"{}\",\n    \"cpp\": \"C++17\",\n    \"dependencies\": [],\n    \"incdirs\": [ \"./src/\", \"./include/{}\" ],\n    \"include-public\": \"include/\"\n}}", name, name);
        std::fs::write(format!("{}/build.json", name), json).unwrap();
    } else {
        std::fs::write(format!("{}/src/main.cpp", name), "#include <cstdio>\n\nint main() {\n    printf(\"Hello World!\");\n}").unwrap();
        let json = format!("{{\n    \"project\": \"{}\",\n    \"cpp\": \"C++17\",\n    \"dependencies\": []\n}}", name);
        std::fs::write(format!("{}/build.json", name), json).unwrap();
    }
    log_info!("successfully created project '{}'", name);
    Ok(())
}

fn action_clean(build: BuildFile) -> Result<(), Error> {
    log_info!("cleaning build files for \"{}\"", build.project);
    std::fs::remove_dir_all("bin/").unwrap();
    Ok(())
}

fn action_build(build: BuildFile, config: Config, mingw: bool, test: bool) -> Result<String, Error> {
    let kind = fetch::get_project_kind(&build.srcdir)?;
    let mut deps = fetch::get_libraries(build.dependencies, config, &build.cpp)?;
    deps.defines.extend(build.defines);
    if test {
        deps.defines.push("TEST".to_string());
    }
    deps.incdirs.extend(build.incdirs);
    let outpath = format!("bin/{}/{}{}", config, build.project, kind.ext());
    let outfile = FileInfo::from_str(&outpath);

    let mut headers = fetch::get_source_files(&PathBuf::from(&build.srcdir), ".h").unwrap();
    if !build.inc_public.is_empty() {
        headers.extend(fetch::get_source_files(&PathBuf::from(&build.inc_public), ".h").unwrap());
    }

    let info = BuildInfo{
        sources: fetch::get_source_files(&PathBuf::from(&build.srcdir), if build.cpp == "c" { ".c" } else { ".cpp" }).unwrap(),
        headers,
        relink: vec![],
        srcdir: build.srcdir,
        outdir: format!("bin/{}/obj/", config),
        outfile: outfile.clone(),
        defines: deps.defines,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,
        cppstd: build.cpp,
        config,
        mingw,
    };
    if let Err(e) = exec::run_build(info) {
        Err(e)
    } else {
        Ok(outpath)
    }
}


macro_rules! exit_with {
    () => { { eprintln!(); std::process::exit(1); } };
    ($($arg:tt)*) => { {
        log_error!($($arg)*);
        std::process::exit(1);
    } };
}


fn main() {
    let args: Vec<_> = std::env::args().collect();
    let cmd = input::parse_input(args).unwrap_or_else(|e| exit_with!("{}", e));

    if let input::Action::New{ name, library } = &cmd {
        action_new(name, *library).unwrap_or_else(|e| exit_with!("{}", e));
    } else {
        let bfile = std::fs::read_to_string("build.json")
            .map_err(|_| Error::FileNotFound("build.json".to_string()))
            .unwrap_or_else(|e| exit_with!("{}", e));
        let build = BuildFile::from_str(&bfile)
            .map_err(Error::JsonParse)
            .unwrap_or_else(|e| exit_with!("{}", e));

        match cmd {
            input::Action::Clean => {
                action_clean(build).unwrap_or_else(|e| exit_with!("{}", e));
            }
            input::Action::Build{ config, mingw } => {
                let build = build.finalise(config);
                action_build(build.clone(), config, mingw, false).unwrap_or_else(|e| exit_with!("{}", e));
            }
            input::Action::Run{ config, mingw, args } => {
                let build = build.finalise(config);
                let outfile = action_build(build.clone(), config, mingw, false).unwrap_or_else(|e| exit_with!("{}", e));
                exec::run_app(&outfile, args)
            }
            input::Action::Test{ config, mingw } => {
                let build = build.finalise(config);
                action_build(build.clone(), config, mingw, true).unwrap_or_else(|e| exit_with!("{}", e));
                testfw::test_lib(build, config)
            }
            _ => (),
        }
    }
}

