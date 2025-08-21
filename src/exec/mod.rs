mod incremental;
mod msvc;
mod posix;
mod prep;

use std::{ io::Write, num::NonZero, path::{ Path, PathBuf }, process::Command };
use incremental::BuildLevel;
use crate::{
    error::Error,
    repr::{ProjKind, ToolChain, Profile, Lang},
    log_error, log_info,
};


#[derive(Debug)]
pub struct BuildInfo {
    pub projkind: ProjKind,
    pub toolchain: ToolChain,
    pub profile: Profile,
    pub lang: Lang,
    pub crtstatic: bool,

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

    pub comp_args: Vec<String>,
    pub link_args: Vec<String>,

}

impl BuildInfo {
    fn compile_info<'a, 'b>(&'a self, pch: &'b PreCompHead) -> CompileInfo<'a, 'b> {
        CompileInfo {
            toolchain: self.toolchain,
            profile: &self.profile,
            lang: self.lang,
            crtstatic: self.crtstatic,
            outdir: &self.outdir,
            defines: &self.defines,
            incdirs: &self.incdirs,
            pch,
            comp_args: &self.comp_args,
        }
    }
}

#[derive(Debug)]
struct CompileInfo<'a, 'b> {
    toolchain: ToolChain,
    profile: &'a Profile,
    lang: Lang,
    crtstatic: bool,
    outdir: &'a Path,
    defines: &'a [String],
    incdirs: &'a [PathBuf],
    pch: &'b PreCompHead<'b>,
    comp_args: &'a [String],
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum PreCompHead<'a> {
    #[default]
    None,
    Create(&'a Path),
    Use(&'a Path),
}


fn on_compile_finish(src: &Path, proc: std::process::Child) -> bool {
    let output = proc.wait_with_output().unwrap();
    if !output.status.success() {
        log_error!("failed to compile '{}'", src.display());
        let _ = std::io::stderr().write_all(&output.stderr);
        let _ = std::io::stderr().write_all(&output.stdout);
        eprintln!();
        true
    } else {
        false
    }
}

pub fn run_build(info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    prep::ensure_out_dirs(&info.srcdir, &info.outdir);
    let mut built_pch = false;

    let pch_use = if let Some(pch) = &info.pch {
        let inpch = info.srcdir.join(pch);
        let incpp = info.outdir.join("pch/pch_impl.cpp");
        let outfile = if info.toolchain.is_msvc() {
            let _ = std::fs::write(&incpp, format!("#include \"{}\"", pch.to_string_lossy()));
            info.outdir.join("obj").join(pch).with_extension("obj")
        } else {
            info.outdir.join("pch").join(pch).with_extension("gch")
        };

        if !std::fs::exists(&outfile)? || (std::fs::metadata(&inpch).unwrap().modified()? > std::fs::metadata(&outfile).unwrap().modified()?) {
            built_pch = true;
            log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.display()));
            log_info!("precompiling header: '{}'", inpch.display());
            let var = PreCompHead::Create(pch);
            let mut cmd = if info.toolchain.is_msvc() {
                msvc::compile_cmd(&incpp, &outfile, info.compile_info(&var), echo, verbose)
            } else {
                posix::compile_cmd(&inpch, &outfile, info.compile_info(&var), echo, verbose)
            };
            if on_compile_finish(&inpch, cmd.spawn().map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?) {
                return Err(Error::CompilerFail(info.outfile));
            }
        }
        PreCompHead::Use(pch)
    } else {
        PreCompHead::None
    };

    match incremental::get_build_level(&info) {
        BuildLevel::UpToDate => {
            log_info!("build up to date for \"{}\"", info.outfile.display());
            return Ok(false);
        }
        BuildLevel::LinkOnly => {
            let _ = std::fs::remove_file(&info.outfile);
        }
        BuildLevel::CompileAndLink(elems) => {
            if !built_pch {
                log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.display()));
            }
            let _ = std::fs::remove_file(&info.outfile);
            let mut handles: Vec<Option<(&Path, std::process::Child)>> = Vec::new();
            handles.resize_with(std::thread::available_parallelism().unwrap_or(NonZero::new(4).unwrap()).get(), || None);
            let mut failure = false;
            let mut count = 0;

            for (src, obj) in elems {
                log_info!("compiling: {}", src.to_string_lossy());
                let mut slot = None;
                for (i, handle) in handles.iter_mut().enumerate() {
                    // IF SLOT IS FULL :: AND ::  SLOT IS FINISHED :: THEN :: FLUSH SLOT
                    if let Some((_, proc)) = handle.as_mut() {
                        if proc.try_wait().unwrap().is_some() {
                            let (src, proc) = std::mem::take(handle).unwrap();
                            failure = failure || on_compile_finish(src, proc);
                            count -= 1;
                            *handle = None;
                            slot = Some(i);
                        }
                    // ELSE SLOT IS FREE
                    } else {
                        slot = Some(i);
                    }
                }

                let slot = slot.unwrap_or_else(|| {
                    // ALL SLOTS FULL - BLOCK ON FIRST ONE
                    let (src, proc) = std::mem::take(&mut handles[0]).unwrap();
                    failure = failure || on_compile_finish(src, proc);
                    count -= 1;
                    0
                });

                if info.toolchain.is_msvc() {
                    handles[slot] = Some((src, msvc::compile_cmd(src, &obj, info.compile_info(&pch_use), echo, verbose).spawn()
                        .map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?));
                } else {
                    handles[slot] = Some((src, posix::compile_cmd(src, &obj, info.compile_info(&pch_use), echo, verbose).spawn()
                        .map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?));
                };
                count += 1;
            }

            for (src, proc) in handles.into_iter().flatten() {
                failure = failure || on_compile_finish(src, proc);
            }

            if failure { return Err(Error::CompilerFail(info.outfile)); }
        }
    }

    log_info!("linking:   {}", info.outfile.display());
    if info.toolchain.is_msvc() {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), ".obj")?;
        if info.projkind == ProjKind::App {
            msvc::link_exe(all_objs, info, echo, verbose)
        } else {
            msvc::link_lib(all_objs, info, echo, verbose)
        }
    } else {
        let all_objs = crate::fetch::source_files(&PathBuf::from(&info.outdir), ".o")?;
        if info.projkind == ProjKind::App {
            posix::link_exe(all_objs, info, echo, verbose)
        } else {
            posix::link_lib(all_objs, info, echo, verbose)
        }
    }
}

pub fn run_app(outfile: &Path, runargs: Vec<String>) -> Result<u8, Error> {
    log_info!("running application {:=<63}", format!("\"{}\" ", outfile.display()));
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
            .unwrap() as u8)
    }
}


#[allow(unused)]
pub fn run_check_outdated(info: BuildInfo) -> Result<bool, Error> {
    Ok(true)
}

/*
pub fn run_check_outdated(info: BuildInfo) -> Result<bool, Error> {
    log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.path.display()));
    prep::ensure_out_dirs(&info.srcdir, &info.outdir);

    if let Some(pch) = &info.pch {
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
            log_info!("starting build for {:=<64}", format!("\"{}\" ", info.outfile.path.display()));
            log_info!("precompiling header: '{inpch}'");
            let mut cmd = if info.toolchain.is_msvc() {
                msvc::compile_cmd(&incpp, &outfile, info.compile_info(), PreCompHead::Create(pch), verbose)
            } else {
                posix::compile_cmd(&inpch, &outfile, info.compile_info(), verbose)
            };
            if on_compile_finish(&inpch, cmd.spawn().map_err(|_| Error::MissingCompiler(info.toolchain.to_string()))?) {
                return Err(Error::CompilerFail(info.outfile.path));
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

