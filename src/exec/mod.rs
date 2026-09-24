pub mod fsutil;
mod incremental;
mod queue;
pub mod toolchain;

use crate::{
    config::{Artefact, BuildSettings, Language, PrecompiledHeader},
    error::Error,
    log_info_ln, log_warn_ln,
};
use incremental::BuildLevel;
use std::{path::{Path, PathBuf}, collections::{HashSet, HashMap}};
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
pub enum PreCompHead<'a> {
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

pub fn run_build(info: BuildInfo, verbose: bool, echo: bool, output_depth: u32) -> Result<(), Error> {
    // replicate source directory hierarchy in output directory
    fsutil::ensure_out_dirs(Path::new("src"), &info.outdir);

    // remove all objects created from sources that no longer exist
    fsutil::cull_zombies(&info.srcdir, &info.outdir, info.lang.src_ext());

    // incremental build, compute outdated files
    let jobs = incremental::get_build_level(&info);

    match jobs {
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
        BuildLevel::CompileAndLink(..) => {
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
        }
    }

    // precompiled headers must finish before compilation can begin
    let mut use_pch = HashMap::new();
    for pch in &info.pch {
        let _ = std::fs::create_dir(info.outdir.join("pch"));
        let inpch = info.srcdir.join(&pch.header); // path/to/header
        let incpp = info.outdir.join(format!("pch/pch_impl.{}", info.lang.src_ext())); // including cpp file (MSVC style)
        let infile = if info.toolchain.is_msvc_compatible() {
            &incpp
        } else {
            &inpch
        };
        let outfile = if info.toolchain.is_msvc_compatible() {
            // output file
            let _ = std::fs::write(&incpp, format!("#include \"{}\"", pch.header.display()));
            info.outdir.join("obj").join(&pch.header).with_extension("h.obj") // MSVC internally reates a .obj and .pch
        } else {
            info.outdir.join("pch").join(&pch.header).with_extension("h.gch") // GNU .gch
        };

        // if PCH requires rebuild
        if info.changed
            || !std::fs::exists(&outfile)?
            || (std::fs::metadata(&inpch)?.modified()? > std::fs::metadata(&outfile)?.modified()?)
        {
            log_info_ln!("precompiling header: {}", inpch.display());
            let var = PreCompHead::Create(&pch.header);
            let output = info.toolchain.compiler().command(&infile, &outfile, &info, &var, verbose, echo)
                .spawn()
                .map_err(|_| Error::CompilerNotFound(info.toolchain))?
                .wait_with_output()
                .unwrap();
            if !info.toolchain.compiler().output(&output) {
                return Err(Error::CompilerFail(info.outfile));
            }
        }

        let used_by: HashSet<_> = if pch.used_by.is_empty() {
            fsutil::scan_for_filetype(Path::new("src"), &[ info.lang.src_ext().to_string() ])?.into_iter().collect()
        } else {
            let mut total = Vec::new();
            for used in &pch.used_by {
                if used.is_dir() {
                    total.extend(fsutil::scan_for_filetype(&used, &[ info.lang.src_ext().to_string() ])?);
                } else {
                    total.push(used.clone());
                }
            }
            total.into_iter().collect()
        };
        let ignored_by: HashSet<_> = {
            let mut total = Vec::new();
            for ignored in &pch.ignored_by {
                if ignored.is_dir() {
                    total.extend(fsutil::scan_for_filetype(&ignored, &[ info.lang.src_ext().to_string() ])?);
                } else {
                    total.push(ignored.clone());
                }
            }
            total.into_iter().collect()
        };
        for entry in used_by.difference(&ignored_by) {
            if let Some(_) = use_pch.insert(entry.to_owned(), pch.header.clone()) {
                log_warn_ln!("source '{}' included in multiple PCH sets - not disjoint", entry.display());
            }
        }
    }

    // recompile all outdated objects, subprocess queue with capacity #cores
    if let BuildLevel::CompileAndLink(jobs) = jobs {
        let mut queue = queue::ProcQueue::new();
        let mut failure = false;

        for (src, obj) in jobs {
            log_info_ln!("compiling: {}", src.to_string_lossy());
            let pch = if let Some(header) = use_pch.get(src) {
                PreCompHead::Use(header)
            } else {
                PreCompHead::None
            };
            let handle = info.toolchain.compiler().command(src, &obj, &info, &pch, verbose, echo)
                .spawn()
                .map_err(|_| Error::CompilerNotFound(info.toolchain))?;
            if let Some(output) = queue.push(handle) && !info.toolchain.compiler().output(&output) {
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
    let toolchain = info.toolchain;
    let outfile = info.outfile.clone();
    let objects = fsutil::scan_for_filetype(Path::new(&info.outdir), &[info.toolchain.object_extension().to_string()])?;
    match info.artefact {
        Artefact::Executable | Artefact::SharedLib => {
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
