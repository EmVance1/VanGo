pub mod fsutil;
mod incremental;
mod pch;
mod pool2;
pub mod toolchain;

use crate::{
    config::{Artefact, Language},
    error::Error,
    action::BuildInfo,
    log_info_ln, log_warn_ln,
};
use incremental::BuildLevel;
use pool2::ProcessPool;
use std::path::Path;
pub use toolchain::Toolchain;

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

pub fn run_build(info: BuildInfo, verbose: bool, echo: bool, output_depth: u32) -> Result<(), Error> {
    // replicate source directory hierarchy in output directory
    fsutil::ensure_out_dirs(Path::new("src"), &info.outdir);

    // remove all objects created from sources that no longer exist
    fsutil::cull_zombies(&info.srcdir, &info.outdir, info.lang.src_ext());

    // precompiled headers must finish before compilation can begin
    let use_pch = pch::precompile_headers(&info, verbose, echo)?;

    // incremental build, compute outdated files
    match incremental::get_build_level(&info) {
        BuildLevel::UpToDate => {
            if !output_depth == 0 {
                log_info_ln!("build up to date for project: {}", info.outfile.display());
            }
            return Ok(());
        }
        BuildLevel::LinkOnly => {
            if output_depth == 0 {
                log_info_ln!("{:=<80}", format!("building dependency: {} ", info.outfile.display()));
            } else {
                log_info_ln!("{:=<80}", format!("building project: {} ", info.outfile.display()));
            }
        }
        BuildLevel::CompileAndLink(jobs) => {
            if output_depth == 0 {
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

            let mut queue = ProcessPool::new();
            let mut failure = false;

            for (src, obj) in jobs {
                log_info_ln!("compiling: {}", src.to_string_lossy());
                let cmd = if let Some(header) = use_pch.get(src) {
                    info.toolchain.compiler().command(src, &obj, &info, pch::UseType::Use(header), verbose, echo)
                } else {
                    info.toolchain.compiler().command(src, &obj, &info, pch::UseType::None, verbose, echo)
                };
                if let Some(finished) = queue.enqueue_task(cmd, Error::CompilerNotFound(info.toolchain))? {
                    if !info.toolchain.compiler().output(&finished) {
                        failure = true;
                    }
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
    }

    let toolchain = info.toolchain;
    let outfile = info.outfile.clone();
    let objects = fsutil::scan_for_filetype(Path::new(&info.outdir), &[info.toolchain.object_extension().to_string()])?;
    match info.artefact {
        Artefact::Executable | Artefact::SharedLib | Artefact::Module => {
            log_info_ln!("linking:   {: <30}", info.outfile.display());
            let output = toolchain.linker().command(objects, info, verbose, echo)
                .output()
                .map_err(|_| Error::LinkerNotFound(toolchain))?;
            if toolchain.linker().output(&output) {
                log_info_ln!("successfully built project: {}\n", outfile.display());
                Ok(())
            } else {
                Err(Error::LinkerFail(outfile))
            }
        }
        Artefact::StaticLib => {
            log_info_ln!("archiving: {: <30}", info.outfile.display());
            let output = toolchain.archiver().command(objects, info, verbose, echo)
                .output()
                .map_err(|_| Error::ArchiverNotFound(toolchain))?;
            if toolchain.archiver().output(&output) {
                log_info_ln!("successfully built project: {}\n", outfile.display());
                Ok(())
            } else {
                Err(Error::ArchiverFail(outfile))
            }
        }
    }
}
