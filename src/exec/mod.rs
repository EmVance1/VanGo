pub mod fsutil;
mod incremental;
pub mod toolchain;
mod queue;

use crate::{
    config::{Artefact, BuildSettings, Language, PrecompiledHeader},
    error::Error,
    log_info_ln, log_warn_ln,
};
use incremental::BuildLevel;
use std::path::{Path, PathBuf};
pub use toolchain::Toolchain;

#[derive(Debug)]
pub struct BuildInfo {
    pub artefact: Artefact,
    pub toolchain: Toolchain,
    pub lang: Language,
    pub cpprt: bool,
    pub settings: BuildSettings,
    pub changed: bool,
    pub is_testexe: bool,

    pub defines: Vec<String>,

    pub srcdir: PathBuf,
    pub incdirs: Vec<PathBuf>,
    pub libdirs: Vec<PathBuf>,
    pub rpaths: Vec<PathBuf>,
    pub outdir: PathBuf,

    pub pch: Vec<PrecompiledHeader>,
    pub sources: Vec<PathBuf>,
    pub headers: Vec<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub relink: Vec<PathBuf>,
    pub outfile: PathBuf,
    pub implib: Option<PathBuf>,

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

fn msvc_check_iso(lang: Language) {
    match lang {
        Language::Cpp(123) => {
            log_warn_ln!("MSVC C++23: using latest working draft (/std:c++latest) - may be incomplete");
        }
        Language::Cpp(n) if n < 114 => {
            log_warn_ln!(
                "MSVC {}: no longer supported - defaulting to C++14",
                lang.to_string().to_ascii_uppercase()
            );
        }
        Language::C(123) => {
            log_warn_ln!("MSVC C23: using latest working draft (/std:clatest) - may be incomplete");
        }
        Language::C(99) => {
            log_warn_ln!("MSVC C99: not officially supported - defaulting to C89 with extensions, may be incomplete");
        }
        _ => (),
    }
}


pub fn run_build(info: BuildInfo, echo: bool, verbose: bool, recursive: bool) -> Result<(), Error> {
    // replicate source directory hierarchy in output directory
    fsutil::ensure_out_dirs(Path::new("src"), &info.outdir);

    // remove all objects created from sources that no longer exist
    fsutil::cull_zombies(&info.srcdir, &info.outdir, info.lang.src_ext());

    // incremental build, compute outdated files
    let jobs = incremental::get_build_level(&info);

    match jobs {
        BuildLevel::UpToDate => {
            if !recursive {
                log_info_ln!("build up to date for project: {}", info.outfile.display());
            }
            return Ok(());
        }
        BuildLevel::LinkOnly => {
            if recursive {
                log_info_ln!("{:=<80}", format!("building dependency: {} ", info.outfile.display()));
            } else {
                log_info_ln!("{:=<80}", format!("building project: {} ", info.outfile.display()));
            }
        }
        BuildLevel::CompileAndLink(..) => {
            if recursive {
                log_info_ln!("{:=<80}", format!("building dependency: {} ", info.outfile.display()));
            } else if info.changed {
                log_info_ln!(
                    "{:=<80}",
                    format!("environment changed - rebuilding project: {} ", info.outfile.display())
                );
            } else {
                log_info_ln!("{:=<80}", format!("building project: {} ", info.outfile.display()));
            }

            // MSVC has sketchy ISO compliance...
            if info.toolchain.is_msvc_compatible() {
                msvc_check_iso(info.lang);
            }
        }
    }

    // precompiled headers must finish before compilation can begin
    let pch_use = if let Some(pch) = &info.pch.first() {
        let pch = &pch.header;
        let _ = std::fs::create_dir(info.outdir.join("pch"));
        let inpch = info.srcdir.join(pch); // path/to/header
        let incpp = info.outdir.join(format!("pch/pch_impl.{}", info.lang.src_ext())); // including cpp file (MSVC style)
        let outfile = if info.toolchain.is_msvc_compatible() {
            // output file
            let _ = std::fs::write(&incpp, format!("#include \"{}\"", pch.display()));
            info.outdir.join("obj").join(pch).with_extension("h.obj") // MSVC internally reates a .obj and .pch
        } else {
            info.outdir.join("pch").join(pch).with_extension("h.gch") // GNU .gch
        };

        // if PCH requires rebuild
        if info.changed
            || !std::fs::exists(&outfile)?
            || (std::fs::metadata(&inpch)?.modified()? > std::fs::metadata(&outfile)?.modified()?)
        {
            log_info_ln!("precompiling header: {}", inpch.display());
            let var = PreCompHead::Create(pch);
            let mut comp = if info.toolchain.is_msvc_compatible() {
                info.toolchain.compiler().command(&incpp, &outfile, &info, &var, verbose, echo)
            } else {
                info.toolchain.compiler().command(&inpch, &outfile, &info, &var, verbose, echo)
            };
            let output = comp
                .spawn()
                .map_err(|_| Error::CompilerNotFound(info.toolchain))?
                .wait_with_output()
                .unwrap();
            if !info.toolchain.compiler().output(&output) {
                return Err(Error::CompilerFail(info.outfile));
            }
        }
        PreCompHead::Use(pch)
    } else {
        PreCompHead::None
    };

    // recompile all outdated objects, subprocess queue with capacity #cores
    if let BuildLevel::CompileAndLink(jobs) = jobs {
        let mut queue = queue::ProcQueue::new();
        let mut failure = false;

        for (src, obj) in jobs {
            log_info_ln!("compiling: {}", src.to_string_lossy());
            let mut comp = info.toolchain.compiler().command(src, &obj, &info, &pch_use, verbose, echo);
            if let Some(output) = queue.push(comp.spawn().map_err(|_| Error::CompilerNotFound(info.toolchain))?)
                && !info.toolchain.compiler().output(&output)
            {
                failure = true;
            }
        }

        while !queue.is_empty() {
            if !info.toolchain.compiler().output(&queue.flush_one()) {
                failure = true;
            }
        }

        if failure {
            return Err(Error::CompilerFail(info.outfile));
        }
    }

    match info.artefact {
        Artefact::Executable | Artefact::SharedLib => {
            log_info_ln!("linking:   {: <30}", info.outfile.display());
        }
        Artefact::StaticLib => log_info_ln!("archiving: {: <30}", info.outfile.display()),
    }
    let toolchain = info.toolchain.clone();
    let outfile   = info.outfile.clone();
    let objects   = fsutil::scan_for_filetype(Path::new(&info.outdir), &[ info.toolchain.object_extension().to_string() ])?;
    match info.artefact {
        Artefact::Executable | Artefact::SharedLib => {
            let mut cmd = toolchain.linker().command(objects, info, verbose, echo);
            if toolchain.linker().output(&cmd.output().map_err(|_| Error::LinkerNotFound(toolchain))?) {
                log_info_ln!("successfully built project: {}\n", outfile.display());
                Ok(())
            } else {
                Err(Error::LinkerFail(outfile))
            }
        }
        Artefact::StaticLib => {
            let mut cmd = toolchain.archiver().command(objects, info, verbose, echo);
            if toolchain.archiver().output(&cmd.output().map_err(|_| Error::ArchiverNotFound(toolchain))?) {
                log_info_ln!("successfully built project: {}\n", outfile.display());
                Ok(())
            } else {
                Err(Error::ArchiverFail(outfile))
            }
        }
    }
}

