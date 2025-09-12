mod incremental;
mod queue;
mod msvc;
mod gnu;
mod prep;
mod output;
#[cfg(test)]
mod mocks;

use std::{io::Write, path::{Path, PathBuf}, process::Command};
use incremental::BuildLevel;
use crate::{
    config::{BuildSettings, Lang, ProjKind, ToolChain}, error::Error, log_info_ln, log_warn_ln
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


#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum PreCompHead<'a> {
    #[default]
    None,
    Create(&'a Path),
    Use(&'a Path),
}


fn on_compile_finish(tc: ToolChain, output: &std::process::Output) -> bool {
    match tc {
        ToolChain::Msvc => output::msvc_compiler(output),
        _  => output::gnu_compiler(output),
    }
}

fn msvc_check_iso(lang: Lang) {
    match lang {
        Lang::Cpp(123) => {
            log_warn_ln!("MSVC C++23: using latest working draft (/std:c++latest) - may be incomplete");
        }
        Lang::Cpp(114) => {
            log_warn_ln!("MSVC {}: no longer supported - defaulting to C++14", lang.to_string().to_ascii_uppercase());
        }
        Lang::C(123) => {
            log_warn_ln!("MSVC C23: using latest working draft (/std:clatest) - may be incomplete");
        }
        Lang::C(99) => {
            log_warn_ln!("MSVC C99: not officially supported - defaulting to C89 with extensions, may be incomplete");
        }
        _ => ()
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
            info.outdir.join("obj").join(pch).with_extension("h.obj")
        } else {
            info.outdir.join("pch").join(pch).with_extension("h.gch")
        };

        if !std::fs::exists(&outfile)? || (std::fs::metadata(&inpch).unwrap().modified()? > std::fs::metadata(&outfile).unwrap().modified()?) {
            built_pch = true;
            log_info_ln!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.display()));
            if info.toolchain.is_msvc() { msvc_check_iso(info.lang); }
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
            if !on_compile_finish(info.toolchain, &output) {
                return Err(Error::CompilerFail(info.outfile));
            }
        }
        PreCompHead::Use(pch)
    } else {
        PreCompHead::None
    };

    match incremental::get_build_level(&info) {
        BuildLevel::UpToDate => {
            if !built_pch {
                log_info_ln!("build up to date for \"{}\"", info.outfile.display());
            }
            return Ok(false);
        }
        BuildLevel::LinkOnly => {
            // ALL GOOD BOSS
        }
        BuildLevel::CompileAndLink(elems) => {
            if !built_pch {
                log_info_ln!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.display()));
                if info.toolchain.is_msvc() { msvc_check_iso(info.lang); }
            }

            let mut queue = queue::ProcQueue::new();
            let mut failure = false;

            for (src, obj) in elems {
                log_info_ln!("compiling: {}", src.to_string_lossy());
                let mut comp = if info.toolchain.is_msvc() {
                    msvc::compile(src, &obj, &info, &pch_use, echo, verbose)
                } else {
                    gnu::compile(src, &obj, &info, &pch_use, echo, verbose)
                };
                if let Some(output) = queue.push(comp.spawn().map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?)
                    && !on_compile_finish(info.toolchain, &output) {
                    failure = true;
                }
            }

            while !queue.is_empty() {
                if !on_compile_finish(info.toolchain, &queue.flush_one()) {
                    failure = true;
                }
            }

            if failure { return Err(Error::CompilerFail(info.outfile)); }
        }
    }


    match info.projkind {
        ProjKind::App|ProjKind::SharedLib{..} => log_info_ln!("linking:   {: <30}", info.outfile.display()),
        ProjKind::StaticLib => log_info_ln!("archiving: {: <30}", info.outfile.display()),
    }
    if info.toolchain.is_msvc() {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), "obj")?;
        match info.projkind {
            ProjKind::App|ProjKind::SharedLib{..} => msvc::link(all_objs, info, echo, verbose),
            ProjKind::StaticLib => msvc::archive(all_objs, info, echo, verbose),
        }
    } else {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), "o")?;
        match info.projkind {
            ProjKind::App|ProjKind::SharedLib{..} => gnu::link(all_objs, info, echo, verbose),
            ProjKind::StaticLib => gnu::archive(all_objs, info, echo, verbose),
        }
    }
}

pub fn run_app(outfile: &Path, runargs: Vec<String>) -> Result<u8, Error> {
    log_info_ln!("running application {:=<63}", format!("\"{}\" ", outfile.display()));
    Ok(Command::new(PathBuf::from(".").join(outfile))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .map_err(|_| Error::InvalidExe(outfile.to_owned()))?
        .code()
        .ok_or(Error::ExeKilled(outfile.to_owned()))? as u8)
}

