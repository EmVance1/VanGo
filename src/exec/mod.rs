mod incremental;
mod msvc;
mod posix;
mod prep;

use crate::{
    error::Error,
    fetch::FileInfo,
    log_error, log_info,
    repr::{Config, ProjKind, ToolChain, Lang},
};
use incremental::BuildLevel;
use std::{io::Write, path::PathBuf, process::Command};


#[derive(Debug)]
pub struct BuildInfo {
    pub projkind: ProjKind,
    pub toolchain: ToolChain,
    pub config: Config,
    pub lang: Lang,

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

    pub comp_args: Vec<String>,
    pub link_args: Vec<String>,
}

impl BuildInfo {
    fn compile_info(&self) -> CompileInfo<'_> {
        CompileInfo {
            toolchain: self.toolchain,
            config: self.config,
            lang: self.lang,
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
    toolchain: ToolChain,
    config: Config,
    lang: Lang,
    outdir: &'a str,
    defines: &'a [String],
    incdirs: &'a [String],
    pch: &'a Option<String>,
    comp_args: &'a [String],
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum PreCompHead<'a> {
    #[default]
    None,
    Create(&'a str),
    Use(&'a str),
}


pub fn run_build(info: BuildInfo, verbose: bool) -> Result<bool, Error> {
    prep::ensure_out_dirs(&info.srcdir, &info.outdir);
    let mut built_pch = false;

    let pch_use = if let Some(pch) = &info.pch {
        let inpch = format!("{}{}", info.srcdir, pch);
        let incpp = format!("{}pch/pch_impl.cpp", info.outdir);
        let outfile = if info.toolchain.is_msvc() {
            std::fs::write(&incpp, format!("#include \"{pch}\"")).unwrap();
            format!("{}obj/{}.obj", info.outdir, pch)
        } else {
            format!("{}pch/{}.gch", info.outdir, pch)
        };

        if !std::fs::exists(&outfile).unwrap() ||
            (std::fs::metadata(&inpch).unwrap().modified().unwrap() > std::fs::metadata(&outfile).unwrap().modified().unwrap())
        {
            built_pch = true;
            log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.repr));
            log_info!("compiling precompiled header: {}{}", info.srcdir, pch);
            let mut cmd = if info.toolchain.is_msvc() {
                msvc::compile_cmd(&incpp, &outfile, info.compile_info(), PreCompHead::Create(pch), verbose)
            } else {
                posix::compile_cmd(&inpch, &outfile, info.compile_info(), verbose)
            };
            let output = cmd.output().map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?;
            if !output.status.success() {
                log_error!("failed to compile precompiled header");
                std::io::stderr().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();
                eprintln!();
                return Err(Error::CompilerFail(info.outfile.repr));
            }
        }
        PreCompHead::Use(pch)
    } else {
        PreCompHead::None
    };

    match incremental::get_build_level(&info) {
        BuildLevel::UpToDate => {
            log_info!("build up to date for \"{}\"", info.outfile.repr);
            return Ok(false);
        }
        BuildLevel::LinkOnly => {
            let _ = std::fs::remove_file(&info.outfile.repr);
        }
        BuildLevel::CompileAndLink(elems) => {
            if !built_pch {
                log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.repr));
            }
            let _ = std::fs::remove_file(&info.outfile.repr);
            let mut handles = Vec::new();
            let mut failure = false;
            const LIMIT: u32 = 12;
            let mut batch = 0;
            for (src, obj) in elems {
                log_info!("compiling: {}", src);
                if info.toolchain.is_msvc() {
                    handles.push((src, msvc::compile_cmd(src, &obj, info.compile_info(), pch_use, verbose)
                        .spawn()
                        .map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?,
                    ));
                } else {
                    handles.push((src, posix::compile_cmd(src, &obj, info.compile_info(), verbose)
                        .spawn()
                        .map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?,
                    ));
                };
                batch += 1;
                if batch == LIMIT {
                    for (src, proc) in handles {
                        let output = proc.wait_with_output().unwrap();
                        if !output.status.success() {
                            log_error!("failed to compile file '{src}'");
                            std::io::stderr().write_all(&output.stderr).unwrap();
                            std::io::stderr().write_all(&output.stdout).unwrap();
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
                    std::io::stderr().write_all(&output.stderr).unwrap();
                    std::io::stderr().write_all(&output.stdout).unwrap();
                    eprintln!();
                    failure = true;
                }
            }

            if failure {
                return Err(Error::CompilerFail(info.outfile.repr));
            }
        }
    }

    log_info!("linking:   {}", info.outfile.repr);
    if info.toolchain.is_msvc() {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), ".obj").unwrap();
        if info.projkind == ProjKind::App {
            msvc::link_exe(all_objs, info, verbose)
        } else {
            msvc::link_lib(all_objs, info, verbose)
        }
    } else {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), ".o").unwrap();
        if info.projkind == ProjKind::App {
            posix::link_exe(all_objs, info, verbose)
        } else {
            posix::link_lib(all_objs, info, verbose)
        }
    }
}

pub fn run_app(outfile: &str, runargs: Vec<String>) -> u8 {
    log_info!("running application {:=<63}", format!("\"{outfile}\" "));
    Command::new(format!("./{outfile}"))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap()
        .code()
        .unwrap() as u8
}


#[allow(unused)]
pub fn run_check_outdated(info: BuildInfo) -> Result<bool, Error> {
    Ok(true)
}

/*
pub fn run_check_outdated(info: BuildInfo) -> Result<bool, Error> {
    log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.repr));
    prep::ensure_out_dirs(&info.srcdir, &info.outdir);

    if let Some(pch) = &info.pch {
        let cmd = if info.toolchain.is_msvc() {
            msvc::precompile_header(pch, &info, false)
        } else {
            posix::precompile_header(pch, &info, false)
        };
        if let Some(mut cmd) = cmd {
            log_info!("compiling precompiled header: {}{}", info.srcdir, pch);
            let output = cmd.output().map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?;
            if !output.status.success() {
                log_error!("failed to compile precompiled header");
                std::io::stderr().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();
                eprintln!();
                return Err(Error::CompilerFail(info.outfile.repr));
            }
        }
    }

    if let BuildLevel::UpToDate = incremental::get_build_level(&info) {
        Ok(false)
    } else {
        Ok(true)
    }
}
*/

