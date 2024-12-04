mod input;
mod repr;
mod fetch;
mod exec;
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



fn action_clean(build: BuildDef) -> Result<(), Error> {
    let kind = fetch::get_project_kind(&PathBuf::from(&build.src_dir))?;
    let outpath = PathBuf::from(&format!("{}.{}", build.project, kind.ext()));
    log_info!("cleaning build files for \"{}\"", outpath.to_str().unwrap());
    std::process::Command::new("rm").args(["-f", "-r", "bin/*"]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.ilk", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.pdb", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &outpath.to_str().unwrap()]).status().unwrap();
    Ok(())
}

fn action_build(build: BuildDef, config: Config, mingw: bool) -> Result<PathBuf, Error> {
    let mut defines = build.defines;
    defines.push(config.as_arg());
    let sources = fetch::get_source_files(&PathBuf::from(&build.src_dir), if build.cppstd == "c" { ".c" } else { ".cpp" }).unwrap();
    let deps = fetch::get_dependencies(build.inc_dirs, build.dependencies, config, &build.cppstd)?;
    defines.extend(deps.defines);
    let kind = fetch::get_project_kind(&PathBuf::from(&build.src_dir))?;
    let outpath = PathBuf::from(&format!("bin/{}/{}.{}", config, build.project, kind.ext()));
    let outfile = FileInfo::from_path(&outpath);
    let info = BuildInfo{
        cppstd: build.cppstd,
        config,
        mingw,
        src_dir: build.src_dir,
        out_dir: format!("bin/{}/obj/", config),
        defines,
        sources,
        headers: deps.headers,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,
        outfile: outfile.clone(),
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
    let build: BuildDef = serde_json::from_str(&bfile)
        .map_err(Error::JsonParse)
        .unwrap_or_else(|e| exit_with!("{}", e));

    if cmd.action == input::Action::Clean {
        action_clean(build).unwrap_or_else(|e| exit_with!("{}", e));
    } else if cmd.action.build() {
        let outfile = action_build(build, cmd.config, cmd.mingw).unwrap_or_else(|e| exit_with!("{}", e));
        if cmd.action.run() {
            exec::run_app(outfile, cmd.args)
        }
    }
}

