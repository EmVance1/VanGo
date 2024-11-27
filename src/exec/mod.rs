mod incremental;
mod prep;
mod msvc;
mod gcc;

use std::{io::Write, path::PathBuf, process::Command};
use crate::{repr::Config, fetch::FileInfo, error::Error, log_info};


#[derive(Debug)]
pub struct BuildInfo {
    pub cppstd: String,
    pub config: Config,
    pub mingw: bool,
    pub src_dir: String,
    pub out_dir: String,
    pub defines: Vec<String>,
    pub sources: Vec<FileInfo>,
    pub headers: Vec<FileInfo>,
    pub incdirs: Vec<String>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
    pub pch: Option<String>,
    pub outfile: FileInfo,
}

impl BuildInfo {
    fn compile_info(&self) -> CompileInfo<'_> {
        CompileInfo{
            cppstd: &self.cppstd,
            config: self.config,
            out_dir: &self.out_dir,
            defines: &self.defines,
            incdirs: &self.incdirs,
            pch: &self.pch,
        }
    }
}

#[derive(Debug)]
struct CompileInfo<'a> {
    cppstd: &'a str,
    config: Config,
    out_dir: &'a str,
    defines: &'a [String],
    incdirs: &'a [String],
    pch: &'a Option<String>,
}


#[cfg(target_os = "windows")]
pub fn run_build(info: BuildInfo) -> Result<(), Error> {
    log_info!("starting build for \"{}\":", info.outfile.repr);
    prep::assert_out_dirs(&info.src_dir, &info.out_dir);

    if let Some(pch) = &info.pch {
        if cfg!(windows) && !info.mingw {
            prep::precompile_header(pch, &info)
            // msvc::prep::precompile_header(pch, &info)
        } else {
            // gcc::prep::precompile_header(pch, &info)
        }
    }

    match incremental::get_outdated(&info) {
        None => {
            log_info!("build up to date for \"{}\"", info.outfile.repr);
            return Ok(())
        }
        Some(elems) => {
            for (src, obj) in elems {
                log_info!("compiling: {}", src);
                let output = if cfg!(windows) && !info.mingw {
                    std::process::Command::new("cl")
                        .args(msvc::compile_cmd(src, &obj, info.compile_info()))
                        .output()
                        .unwrap()
                } else {
                    std::process::Command::new("g++")
                        .args(gcc::compile_cmd(src, &obj, info.compile_info()))
                        .output()
                        .unwrap()
                };
                std::io::stdout().write_all(&output.stdout).unwrap();
                println!();
                if !output.status.success() { return Err(Error::CompilerFail(src.to_string())) }
            }
        }
    }

    let all_objs = crate::fetch::get_source_files(&PathBuf::from(&info.out_dir), ".obj").unwrap();
    log_info!("linking: {}", info.outfile.repr);
    if cfg!(windows) && !info.mingw {
        if info.outfile.repr.ends_with(".lib") {
            msvc::link_lib(all_objs, info)
        } else {
            msvc::link_exe(all_objs, info)
        }
    } else {
        if info.outfile.repr.ends_with(".a") {
            gcc::link_lib(all_objs, info)
        } else {
            gcc::link_exe(all_objs, info)
        }
    }
}


pub fn run_app(outfile: PathBuf,  runargs: Vec<String>) {
    log_info!("running application \"{}\"...", outfile.to_str().unwrap());
    Command::new(format!("./{}", outfile.to_str().unwrap()))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
}



#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::*;

    #[test]
    pub fn test_compile_cmd() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";
        let defines = vec![];
        let incdirs = vec![ "src/".to_string() ];
        let pch = None;
        let info = CompileInfo{
            cppstd: "c++20",
            config: Config::Debug,
            out_dir: "bin/debug/obj/",
            defines: &defines,
            incdirs: &incdirs,
            pch: &pch,
        };

        let args = msvc::compile_cmd(src, obj, info);
        assert_eq!(args, vec![
            "src/main.cpp".to_string(),
            "/c".to_string(),
            "/EHsc".to_string(),
            "/std:c++20".to_string(),
            "/Fo:bin/debug/obj/main.obj".to_string(),
            "/Isrc/".to_string(),
            "/MDd".to_string(),
            "/Od".to_string(),
        ]);
    }
}

