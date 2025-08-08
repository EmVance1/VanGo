mod incremental;
mod prep;
mod msvc;
mod posix;

use std::{io::Write, path::PathBuf, process::Command};
use crate::{error::Error, fetch::FileInfo, repr::{Config, ProjKind, ToolSet}, log_info, log_error };
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
    pub is_c: bool,
    pub config: Config,
    pub toolset: ToolSet,
    pub projkind: ProjKind,
    pub comp_args: Vec<String>,
    pub link_args: Vec<String>,
}

impl BuildInfo {
    fn compile_info(&self) -> CompileInfo<'_> {
        CompileInfo{
            cppstd: &self.cppstd,
            is_c: self.is_c,
            toolset: self.toolset,
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
    is_c: bool,
    toolset: ToolSet,
    config: Config,
    outdir: &'a str,
    defines: &'a [String],
    incdirs: &'a [String],
    pch: &'a Option<String>,
    comp_args: &'a [String],
}


pub fn run_build(info: BuildInfo) -> Result<bool, Error> {
    log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.repr));
    prep::assert_out_dirs(&info.srcdir, &info.outdir);

    if let Some(pch) = &info.pch {
        if info.toolset.is_msvc() {
            if let Some(mut cmd) = msvc::precompile_header(pch, &info) {
                log_info!("compiling precompiled header: {}{}", info.srcdir, pch);
                let output = cmd.output().map_err(|_| Error::MissingCompiler(info.toolset.to_string()))?;
                if !output.status.success() {
                    log_error!("failed to compile precompiled header");
                    std::io::stderr().write_all(&output.stdout).unwrap();
                    eprintln!();
                    return Err(Error::CompilerFail(info.outfile.repr))
                }
            }
        } else {
            if let Some(mut cmd) = posix::precompile_header(pch, &info) {
                log_info!("compiling precompiled header: {}{}", info.srcdir, pch);
                let output = cmd.output().map_err(|_| Error::MissingCompiler(info.toolset.to_string()))?;
                if !output.status.success() {
                    log_error!("failed to compile precompiled header");
                    std::io::stderr().write_all(&output.stderr).unwrap();
                    eprintln!();
                    return Err(Error::CompilerFail(info.outfile.repr))
                }
            }
        }
    }

    match incremental::get_build_level(&info) {
        BuildLevel::UpToDate => {
            log_info!("build up to date for \"{}\"", info.outfile.repr);
            return Ok(false)
        }
        BuildLevel::LinkOnly => {
            let _ = std::fs::remove_file(&info.outfile.repr);
        }
        BuildLevel::CompileAndLink(elems) => {
            let _ = std::fs::remove_file(&info.outfile.repr);
            let mut handles = Vec::new();
            let mut failure = false;
            const LIMIT: u32 = 12;
            let mut batch = 0;
            for (src, obj) in elems {
                log_info!("compiling: {}", src);
                if info.toolset.is_msvc() {
                    handles.push((src, msvc::compile_cmd(src, &obj, info.compile_info())
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                            .map_err(|_| Error::MissingCompiler(info.toolset.to_string()))?));
                } else {
                    handles.push((src, posix::compile_cmd(src, &obj, info.compile_info())
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::piped())
                            .spawn()
                            .map_err(|_| Error::MissingCompiler(info.toolset.to_string()))?));
                };
                batch += 1;
                if batch == LIMIT {
                    for (src, proc) in handles {
                        let output = proc.wait_with_output().unwrap();
                        if !output.status.success() {
                            log_error!("failed to compile file '{src}'");
                            std::io::stderr().write_all(&output.stdout).unwrap();
                            std::io::stderr().write_all(&output.stderr).unwrap();
                            eprintln!();
                            failure = true;
                        }
                    }
                    batch = 0;
                    handles = Vec::new();
                }
            }

            for (src, proc) in handles {
                let output = proc.wait_with_output().unwrap();
                if !output.status.success() {
                    log_error!("failed to compile file '{src}'");
                    std::io::stderr().write_all(&output.stdout).unwrap();
                    std::io::stderr().write_all(&output.stderr).unwrap();
                    eprintln!();
                    failure = true;
                }
            }

            if failure {
                return Err(Error::CompilerFail(info.outfile.repr))
            }
        }
    }

    log_info!("linking:   {}", info.outfile.repr);
    if info.toolset.is_msvc() {
        let all_objs = crate::fetch::get_source_files(&PathBuf::from(&info.outdir), ".obj").unwrap();
        if info.projkind == ProjKind::App {
            msvc::link_exe(all_objs, info)
        } else {
            msvc::link_lib(all_objs, info)
        }
    } else {
        let all_objs = crate::fetch::get_source_files(&PathBuf::from(&info.outdir), ".o").unwrap();
        if info.projkind == ProjKind::App {
            posix::link_exe(all_objs, info)
        } else {
            posix::link_lib(all_objs, info)
        }
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


#[allow(unused)]
pub fn run_check_outdated(info: BuildInfo) -> Result<bool, Error> {
    log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.repr));
    prep::assert_out_dirs(&info.srcdir, &info.outdir);

    if let Some(pch) = &info.pch {
        if info.toolset.is_msvc() {
            if let Some(mut cmd) = msvc::precompile_header(pch, &info) {
                log_info!("compiling precompiled header: {}{}", info.srcdir, pch);
                let output = cmd.output().map_err(|_| Error::MissingCompiler(info.toolset.to_string()))?;
                if !output.status.success() {
                    log_error!("failed to compile precompiled header");
                    std::io::stderr().write_all(&output.stdout).unwrap();
                    eprintln!();
                    return Err(Error::CompilerFail(info.outfile.repr))
                }
            }
        } else {
            if let Some(mut cmd) = posix::precompile_header(pch, &info) {
                log_info!("compiling precompiled header: {}{}", info.srcdir, pch);
                let output = cmd.output().map_err(|_| Error::MissingCompiler(info.toolset.to_string()))?;
                if !output.status.success() {
                    log_error!("failed to compile precompiled header");
                    std::io::stderr().write_all(&output.stderr).unwrap();
                    eprintln!();
                    return Err(Error::CompilerFail(info.outfile.repr))
                }
            }
        }
    }

    if let BuildLevel::UpToDate = incremental::get_build_level(&info) {
        return Ok(false)
    } else {
        return Ok(true)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    pub fn test_compile_cmd() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";
        let empty = vec![];
        let incdirs = vec![ "src/".to_string() ];
        let pch = None;
        let info = CompileInfo{
            cppstd: "c++20",
            is_c: false,
            config: Config::Debug,
            toolset: ToolSet::MSVC,
            outdir: "bin/debug/obj/",
            defines: &empty,
            incdirs: &incdirs,
            pch: &pch,
            comp_args: &empty,
        };

        let cmd = msvc::compile_cmd(src, obj, info);
        assert_eq!(cmd.get_args().collect::<Vec<_>>(), &[
            "/std:c++20",
            "/c",
            "src/main.cpp",
            "/Fo:bin/debug/obj/main.obj",
            "/EHsc",
            "/Isrc/",
            "/MDd",
            "/Od",
            "/Zi",
            "/FS",
        ]);
    }
}

