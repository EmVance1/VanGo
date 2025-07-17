mod incremental;
mod prep;
mod msvc;
mod gcc;

use std::{io::Write, path::PathBuf, process::Command};
use crate::{repr::Config, fetch::FileInfo, error::Error, log_info};
use incremental::BuildLevel;


#[derive(Debug)]
pub struct BuildInfo {
    pub sources: Vec<FileInfo>,
    pub headers: Vec<FileInfo>,
    pub relink: Vec<FileInfo>,
    pub srcdir: String,
    pub outdir: String,
    pub outfile: FileInfo,
    pub defines: Vec<String>,
    pub incdirs: Vec<String>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
    pub pch: Option<String>,
    pub cppstd: String,
    pub config: Config,
    pub mingw: bool,
    pub comp_args: Vec<String>,
    pub link_args: Vec<String>,
}

impl BuildInfo {
    fn compile_info(&self) -> CompileInfo<'_> {
        CompileInfo{
            cppstd: &self.cppstd,
            config: self.config,
            outdir: &self.outdir,
            defines: &self.defines,
            incdirs: &self.incdirs,
            pch: &self.pch,
            comp_args: &self.comp_args,
        }
    }
}

#[derive(Debug)]
struct CompileInfo<'a> {
    cppstd: &'a str,
    config: Config,
    outdir: &'a str,
    defines: &'a [String],
    incdirs: &'a [String],
    pch: &'a Option<String>,
    comp_args: &'a [String],
}


#[cfg(target_os = "windows")]
pub fn run_build(info: BuildInfo) -> Result<bool, Error> {
    log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.repr));
    prep::assert_out_dirs(&info.srcdir, &info.outdir);

    if let Some(pch) = &info.pch {
        if cfg!(windows) && !info.mingw {
            msvc::prep::precompile_header(pch, &info)
        } else {
            gcc::prep::precompile_header(pch, &info)
        }
    }

    match incremental::get_build_level(&info) {
        BuildLevel::UpToDate => {
            log_info!("build up to date for \"{}\"", info.outfile.repr);
            return Ok(false)
        }
        BuildLevel::LinkOnly => {
            let _ = std::fs::remove_file(&info.outfile.repr); // .unwrap();
        }
        BuildLevel::CompileAndLink(elems) => {
            let _ = std::fs::remove_file(&info.outfile.repr);
            let mut handles = Vec::new();
            const LIMIT: u32 = 12;
            let mut batch = 0;
            for (src, obj) in elems {
                log_info!("compiling: {}", src);
                if cfg!(windows) && !info.mingw {
                    let args = msvc::compile_cmd(src, &obj, info.compile_info());
                    handles.push((src, std::process::Command::new("cl")
                        .args(args)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .unwrap()));
                } else if info.cppstd.starts_with("c++") {
                    handles.push((src, std::process::Command::new("g++")
                        .args(gcc::compile_cmd(src, &obj, info.compile_info()))
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .unwrap()));
                } else {
                    handles.push((src, std::process::Command::new("gcc")
                        .args(gcc::compile_cmd(src, &obj, info.compile_info()))
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .unwrap()));
                };
                batch += 1;
                if batch == LIMIT {
                    for (src, proc) in handles {
                        let output = proc.wait_with_output().unwrap();
                        if !output.status.success() {
                            std::io::stdout().write_all(&output.stdout).unwrap();
                            return Err(Error::CompilerFail(src.to_string()))
                        }
                    }
                    batch = 0;
                    handles = Vec::new();
                }
            }

            for (src, proc) in handles {
                let output = proc.wait_with_output().unwrap();
                if !output.status.success() {
                    std::io::stdout().write_all(&output.stdout).unwrap();
                    return Err(Error::CompilerFail(src.to_string()))
                }
            }
        }
    }

    let all_objs = crate::fetch::get_source_files(&PathBuf::from(&info.outdir), ".obj").unwrap();
    log_info!("linking:   {}", info.outfile.repr);
    if cfg!(windows) && !info.mingw {
        if info.outfile.repr.ends_with(".lib") {
            msvc::link_lib(all_objs, info)
        } else {
            msvc::link_exe(all_objs, info)
        }
    } else if info.outfile.repr.ends_with(".a") {
            gcc::link_lib(all_objs, info)
        } else {
            gcc::link_exe(all_objs, info)
        }
}

#[cfg(target_os = "windows")]
#[allow(unused)]
pub fn run_check_outdated(info: BuildInfo) -> bool {
    log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.repr));
    prep::assert_out_dirs(&info.srcdir, &info.outdir);

    if let Some(pch) = &info.pch {
        if cfg!(windows) && !info.mingw {
            msvc::prep::precompile_header(pch, &info)
        } else {
            gcc::prep::precompile_header(pch, &info)
        }
    }

    if let BuildLevel::UpToDate = incremental::get_build_level(&info) {
        return false
    } else {
        return true
    }
}


pub fn run_app(outfile: &str,  runargs: Vec<String>) -> u8 {
    log_info!("running application {:=<63}", format!("\"{}\" ", outfile));
    Command::new(format!("./{}", outfile))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap()
        .code()
        .unwrap() as u8
}



#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use super::*;

    #[test]
    pub fn test_compile_cmd() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";
        let empty = vec![];
        let incdirs = vec![ "src/".to_string() ];
        let pch = None;
        let info = CompileInfo{
            cppstd: "c++20",
            config: Config::Debug,
            outdir: "bin/debug/obj/",
            defines: &empty,
            incdirs: &incdirs,
            pch: &pch,
            comp_args: &empty,
        };

        let args = msvc::compile_cmd(src, obj, info);
        assert_eq!(args, vec![
            "src/main.cpp".to_string(),
            "/c".to_string(),
            "/EHsc".to_string(),
            "/Fo:bin/debug/obj/main.obj".to_string(),
            "/std:c++20".to_string(),
            "/Isrc/".to_string(),
            "/MDd".to_string(),
            "/Od".to_string(),
            "/Zi".to_string(),
            "/FS".to_string(),
        ]);
    }
}

