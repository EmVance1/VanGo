mod input;
mod repr;
mod fetch;
mod exec;

use std::path::PathBuf;
use input::Config;
use repr::*;
use fetch::FileInfo;
use exec::BuildInfo;


fn action_clean(build: BuildDef) -> Result<(), String> {
    let sources = fetch::get_source_files(&PathBuf::from(&build.src_dir), ".cpp").unwrap();
    let kind = fetch::get_project_kind(&sources)?;
    let outpath = PathBuf::from(&format!("{}.{}", build.project, kind.ext()));
    let outfile = FileInfo::from_path(&outpath);
    println!("[mscmp:  info] cleaning build files for \"{}\"", outfile.repr);
    std::process::Command::new("rm").args(["-f", "-r", "bin/*"]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.ilk", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &format!("{}.pdb", build.project)]).status().unwrap();
    std::process::Command::new("rm").args(["-f", &outfile.repr]).status().unwrap();
    Ok(())
}

fn action_build(build: BuildDef, config: Config) -> Result<FileInfo, String> {
    let mut defines = build.defines;
    defines.push(config.as_arg());
    let sources = fetch::get_source_files(&PathBuf::from(&build.src_dir), ".cpp").unwrap();
    let deps = fetch::get_dependencies(build.inc_dirs, build.dependencies, config, u32_from_cppstd(&build.cppstd))?;
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
    if exec::run_build(info).is_ok() {
        Ok(outfile)
    } else {
        Err("[mscmp: error] build failed".to_string())
    }
}


fn main() {
    let cmd = input::get_input().unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1) });
    let bfile = std::fs::read_to_string("build.json")
        .unwrap_or_else(|_| { eprintln!("[mscmp: error] build.json was not found"); std::process::exit(1) });
    let build: BuildDef = serde_json::from_str(&bfile)
        .unwrap_or_else(|e| { eprintln!("[mscmp: error] parse json error: {}", e); std::process::exit(1) });

    if cmd.action == input::Action::Clean {
        action_clean(build).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
    } else if cmd.action.build() {
        let outfile = action_build(build, cmd.config).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); });
        if cmd.action.run() {
            exec::run_app(outfile, cmd.args)
        }
    }
}

