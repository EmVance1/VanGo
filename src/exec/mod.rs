mod incremental;
mod queue;
mod msvc;
mod gnu;
mod prep;
mod output;
// mod agnostic;

use std::{io::Write, path::{Path, PathBuf}, process::Command};
use incremental::BuildLevel;
use crate::{
    config::{BuildSettings, Lang, ProjKind, ToolChain}, error::Error, exec::output::*, log_info_ln
};


#[derive(Debug)]
pub struct BuildInfo {
    pub projkind: ProjKind,
    pub toolchain: ToolChain,
    pub lang: Lang,
    pub crtstatic: bool,
    pub cpprt: bool,
    pub settings: BuildSettings,

    pub defines:  Vec<String>,

    pub srcdir:   PathBuf,
    pub incdirs:  Vec<PathBuf>,
    pub libdirs:  Vec<PathBuf>,
    pub outdir:   PathBuf,

    pub pch:      Option<PathBuf>,
    pub sources:  Vec<PathBuf>,
    pub headers:  Vec<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub relink:   Vec<PathBuf>,
    pub outfile:  PathBuf,
    pub implib:   Option<PathBuf>,

    pub comp_args: Vec<String>,
    pub link_args: Vec<String>,

}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum PreCompHead<'a> {
    #[default]
    None,
    Create(&'a Path),
    Use(&'a Path),
}


fn on_compile_finish(tc: ToolChain, output: std::process::Output) -> bool {
    match tc {
        ToolChain::Msvc => on_msvc_compile_finish(output),
        _  => on_gnu_compile_finish(output),
    }
}

pub fn run_build(info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    prep::ensure_out_dirs(&info.srcdir, &info.outdir);
    let mut built_pch = false;

    let pch_use = if let Some(pch) = &info.pch {
        let inpch = info.srcdir.join(pch);
        let incpp = info.outdir.join(format!("pch/pch_impl.{}", info.lang.src_ext()));
        let outfile = if info.toolchain.is_msvc() {
            let _ = std::fs::write(&incpp, format!("#include \"{}\"", pch.to_string_lossy()));
            info.outdir.join("obj").join(pch).with_extension("obj")
        } else {
            info.outdir.join("pch").join(pch).with_extension("gch")
        };

        if !std::fs::exists(&outfile)? || (std::fs::metadata(&inpch).unwrap().modified()? > std::fs::metadata(&outfile).unwrap().modified()?) {
            built_pch = true;
            log_info_ln!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.display()));
            log_info_ln!("precompiling header: '{}'", inpch.display());
            let var = PreCompHead::Create(pch);
            let mut comp = if info.toolchain.is_msvc() {
                msvc::compile(&incpp, &outfile, &info, &var, echo, verbose)
            } else {
                gnu::compile(&inpch, &outfile, &info, &var, echo, verbose)
            };
            let output = comp.spawn()
                .map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?
                .wait_with_output()
                .unwrap();
            if !on_compile_finish(info.toolchain, output) {
                return Err(Error::CompilerFail(info.outfile));
            }
        }
        PreCompHead::Use(pch)
    } else {
        PreCompHead::None
    };

    match incremental::get_build_level(&info) {
        BuildLevel::UpToDate => {
            log_info_ln!("build up to date for \"{}\"", info.outfile.display());
            return Ok(false);
        }
        BuildLevel::LinkOnly => {
            let _ = std::fs::remove_file(&info.outfile);
        }
        BuildLevel::CompileAndLink(elems) => {
            if !built_pch {
                log_info_ln!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.display()));
            }
            let _ = std::fs::remove_file(&info.outfile);

            let mut queue = queue::ProcQueue::new();
            let mut failure = false;

            for (src, obj) in elems {
                log_info_ln!("compiling: {}", src.to_string_lossy());
                let mut comp = if info.toolchain.is_msvc() {
                    msvc::compile(src, &obj, &info, &pch_use, echo, verbose)
                } else {
                    gnu::compile(src, &obj, &info, &pch_use, echo, verbose)
                };
                if let Some(output) = queue.push(comp.spawn().map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?) {
                    if !on_compile_finish(info.toolchain, output) {
                        failure = true;
                    }
                }
            }

            while !queue.is_empty() {
                let output = queue.flush_one();
                if !on_compile_finish(info.toolchain, output) {
                    failure = true;
                }
            }

            if failure { return Err(Error::CompilerFail(info.outfile)); }
        }
    }

    log_info_ln!("linking:   {: <30}", info.outfile.display());
    if info.toolchain.is_msvc() {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), "obj")?;
        match info.projkind {
            ProjKind::App           => msvc::link_exe(all_objs, info, echo, verbose),
            ProjKind::SharedLib{..} => msvc::link_shared_lib(all_objs, info, echo, verbose),
            ProjKind::StaticLib     => msvc::link_static_lib(all_objs, info, echo, verbose),
        }
    } else {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), "o")?;
        match info.projkind {
            ProjKind::App           => gnu::link_exe(all_objs, info, echo, verbose),
            ProjKind::SharedLib{..} => gnu::link_shared_lib(all_objs, info, echo, verbose),
            ProjKind::StaticLib     => gnu::link_static_lib(all_objs, info, echo, verbose),
        }
    }
}

pub fn run_app(outfile: &Path, runargs: Vec<String>) -> Result<u8, Error> {
    log_info_ln!("running application {:=<63}", format!("\"{}\" ", outfile.display()));
    let ext = outfile.extension().unwrap_or_default().to_string_lossy();
    if ext == "a" || ext == "lib" {
        Err(Error::LibNotExe(outfile.to_owned()))
    } else {
        Ok(Command::new(PathBuf::from(".").join(outfile))
            .args(runargs)
            .current_dir(std::env::current_dir().unwrap())
            .status()
            .map_err(|_| Error::InvalidExe(outfile.to_owned()))?
            .code()
            .ok_or(Error::ExeKilled(outfile.to_owned()))? as u8)
    }
}

