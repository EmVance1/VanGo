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



fn action_clean(build: BuildFile) -> Result<(), Error> {
    let kind = fetch::get_project_kind(&build.srcdir)?;
    let outpath = PathBuf::from(&format!("{}{}", build.project, kind.ext()));
    log_info!("cleaning build files for \"{}\"", outpath.to_str().unwrap());
    std::process::Command::new("rm").args(["-f", "-r", "bin/*"]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.ilk", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.pdb", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &outpath.to_str().unwrap()]).status().unwrap();
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
    let bfile = std::fs::read_to_string("build.json")
        .map_err(|_| Error::FileNotFound("build.json".to_string()))
        .unwrap_or_else(|e| exit_with!("{}", e));
    let build = BuildFile::from_str(&bfile)
        .map_err(Error::JsonParse)
        .unwrap_or_else(|e| exit_with!("{}", e))
        .finalise(cmd.config);

    if cmd.action == input::Action::Clean {
        action_clean(build).unwrap_or_else(|e| exit_with!("{}", e));
    } else {
        let outfile = action_build(build.clone(), cmd.config, cmd.mingw, cmd.action.test()).unwrap_or_else(|e| exit_with!("{}", e));
        if cmd.action.run() {
            exec::run_app(&outfile, cmd.args)
        } else if cmd.action.test() {
            testfw::test_lib(build, cmd.config)
        }
    }
}

