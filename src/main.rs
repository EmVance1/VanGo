mod input;
mod repr;
mod fetch;
mod exec;
mod error;
#[macro_use]
mod log;

use std::io::Write;
use std::path::PathBuf;
use repr::*;
use fetch::FileInfo;
use exec::BuildInfo;
use error::Error;



fn action_clean(build: BuildDef) -> Result<(), Error> {
    let sources = fetch::get_source_files(&PathBuf::from(&build.src_dir), ".cpp").unwrap();
    let kind = fetch::get_project_kind(&sources)?;
    let outpath = PathBuf::from(&format!("{}.{}", build.project, kind.ext()));
    let outfile = FileInfo::from_path(&outpath);
    log_info!("cleaning build files for \"{}\"", outfile.repr);
    std::process::Command::new("rm").args(["-f", "-r", "bin/*"]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.ilk", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.pdb", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &outfile.repr]).status().unwrap();
    Ok(())
}

fn action_build(build: BuildDef, config: Config) -> Result<FileInfo, Error> {
    let mut defines = build.defines;
    defines.push(config.as_arg());
    let sources = fetch::get_source_files(&PathBuf::from(&build.src_dir), ".cpp").unwrap();
    let deps = fetch::get_dependencies(build.inc_dirs, build.dependencies, config, &build.cppstd)?;
    defines.extend(deps.defines);
    let oplevel = if config.is_release() { "/O2".to_string() } else { "/Od".to_string() };
    let kind = fetch::get_project_kind(&sources)?;
    let outpath = PathBuf::from(&format!("bin/{}/{}.{}", config.to_string(), build.project, kind.ext()));
    let outfile = FileInfo::from_path(&outpath);
    let info = BuildInfo{
        cppstd: build.cppstd,
        config,
        src_dir: build.src_dir,
        out_dir: format!("bin/{}/obj/", config),
        defines,
        sources,
        headers: deps.headers,
        incdirs: deps.incdirs,
        libdirs: deps.libdirs,
        links: deps.links,
        pch: build.pch,
        oplevel,
        outfile: outfile.clone(),
    };
    if let Err(e) = exec::run_build(info) {
        Err(e)
    } else {
        Ok(outfile)
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
        .map_err(|e| Error::JsonParse(e))
        .unwrap_or_else(|e| exit_with!("{}", e));

    if cmd.action == input::Action::Clean {
        action_clean(build).unwrap_or_else(|e| exit_with!("{}", e));
    } else if cmd.action.build() {
        let outfile = action_build(build, cmd.config).unwrap_or_else(|e| exit_with!("{}", e));
        if cmd.action.run() {
            exec::run_app(outfile, cmd.args)
        }
    }
}

