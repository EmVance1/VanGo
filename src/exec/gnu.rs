use std::{ffi::OsString, io::Write, path::{Path, PathBuf}};
use super::{BuildInfo, PreCompHead};
use crate::{config::{ProjKind, WarnLevel}, exec::output::on_gnu_link_finish, log_info_ln, Error};


pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp() || info.cpprt);
    let args = info.toolchain.args();

    cmd.args(&info.comp_args);
    if !cfg!(target_os = "windows") {
        match info.projkind {
            ProjKind::App           => { cmd.arg("-fpie"); },
            ProjKind::SharedLib{..} => { cmd.arg("-fPIC"); },
            ProjKind::StaticLib     => { cmd.arg("-fpie"); },
        }
    }
    if info.lang.is_cpp() {
        cmd.args(args.eh_default_cpp());
    }
    cmd.arg(args.std(info.lang));
    cmd.arg(args.comp_only());
    match info.settings.opt_level {
        0 => { cmd.args(args.O0()); }
        1 => { cmd.args(args.O1()); }
        2 => { cmd.args(args.O2()); }
        3 => { cmd.args(args.O3()); }
        _ => (),
    }
    if info.settings.opt_size {
        cmd.args(args.Os());
    }
    if info.settings.opt_size {
        cmd.args(args.Ot());
    }
    if info.settings.opt_linktime {
        cmd.args(args.Olinktime());
    }
    if info.settings.debug_info {
        cmd.args(args.debug_symbols());
    }
    match info.settings.warn_level {
        WarnLevel::None  => { cmd.arg("-w"); }
        WarnLevel::Basic => { cmd.arg("-Wall"); }
        WarnLevel::High  => { cmd.args([ "-Wall", "-Wextra", "-Wpedantic", "-Wconversion", "-Wsign-conversion", "-Wshadow",
                                     "-Wformat=2", "-Wnull-dereference", "-Wdouble-promotion", "-Wimplicit-fallthrough=5" ]); }
    }
    if info.settings.warn_as_error {
        cmd.arg("-Werror");
    }
    if info.settings.iso_compliant {
        cmd.arg("-pedantic-errors");
    }
    cmd.args(info.incdirs.iter().map(|inc| format!("-I{}", inc.display())));
    cmd.args(info.defines.iter().map(|def| format!("-D{}", def)));
    match pch {
        PreCompHead::Use(_) => {
            let mut fparg = OsString::from("-I");
            fparg.push(info.outdir.join("pch"));
            cmd.arg(fparg);
        }
        _ => (),
    }

    cmd.arg(src);
    cmd.arg(args.comp_output(&obj.to_string_lossy()));

    cmd.stderr(std::process::Stdio::piped());
    if verbose {
        cmd.arg("--verbose");
        cmd.stdout(std::process::Stdio::piped());
    } else {
        cmd.stdout(std::process::Stdio::null());
    };
    if echo { print_command(&cmd); }
    cmd
}

pub(super) fn link_exe(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp() || info.cpprt);
    let args = info.toolchain.args();

    cmd.args(info.link_args);
    if info.settings.aslr {
        if cfg!(target_os = "windows") {
            cmd.arg("-Wl,--dynamicbase");
        } else {
            cmd.arg("-pie");
        }
    }
    if info.crtstatic {
        if info.lang.is_cpp() || info.cpprt {
            cmd.arg("-static-libstdc++");
        }
        cmd.arg("-static-libgcc");
    }
    if info.settings.opt_linktime {
        cmd.arg("-flto");
    }

    cmd.args(objs);
    cmd.args(info.libdirs.iter().map(|l| format!("{}{}", args.L(), l.display())));
    cmd.args(info.archives.iter().map(|l| format!("{}{}", args.l(), l.display())));
    cmd.arg(args.link_output(&info.outfile.to_string_lossy()));

    if echo { print_command(&cmd); }
    if verbose { cmd.arg("--verbose"); }
    let output = cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?;
    if !on_gnu_link_finish(output) {
        Err(Error::LinkerFail(info.outfile))
    } else {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    }
}

pub(super) fn link_shared_lib(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp() || info.cpprt);
    let args = info.toolchain.args();

    cmd.args(info.link_args);
    if cfg!(target_os = "macos") {
        cmd.arg("-dynamiclib");
    } else {
        cmd.arg("-shared");
    }
    if cfg!(target_os = "windows") {
        if let Some(lib) = info.implib {
            if info.settings.aslr {
                cmd.arg(format!("-Wl,--dynamicbase,--out-implib,{}", lib.display()));
            } else {
                cmd.arg(format!("-Wl,--out-implib,{}", lib.display()));
            }
        } else if info.settings.aslr {
            cmd.arg("-Wl,--dynamicbase");
        }
    } else if info.settings.aslr {
        cmd.arg("-fPIC");
    }
    if info.crtstatic {
        if info.lang.is_cpp() || info.cpprt {
            cmd.arg("-static-libstdc++");
        }
        cmd.arg("-static-libgcc");
    }
    if info.settings.opt_linktime {
        cmd.arg("-flto");
    }

    cmd.args(objs);
    cmd.args(info.libdirs.iter().map(|l| format!("{}{}", args.L(), l.display())));
    cmd.args(info.archives.iter().map(|l| format!("{}{}", args.l(), l.display())));
    cmd.arg(args.link_output(&info.outfile.to_string_lossy()));

    if echo { print_command(&cmd); }
    if verbose { cmd.arg("--verbose"); }
    let output = cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?;
    if !on_gnu_link_finish(output) {
        Err(Error::LinkerFail(info.outfile))
    } else {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    }
}

pub(super) fn link_static_lib(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.archiver();
    // let args = info.toolchain.args();

    if verbose {
        cmd.arg("rcsv");
    } else {
        cmd.arg("rcs");
    }
    cmd.arg(&info.outfile);
    cmd.args(info.link_args);
    cmd.args(objs);

    if echo { print_command(&cmd); }
    let output = cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?;
    if !output.status.success() {
        let _ = std::io::stderr().write_all(&output.stderr);
        if verbose { let _ = std::io::stderr().write_all(&output.stdout); }
        eprintln!();
        Err(Error::ArchiverFail(info.outfile))
    } else {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    }
}


fn print_command(cmd: &std::process::Command) {
    print!("{} ", cmd.get_program().display());
    for arg in cmd.get_args() {
        print!("{} ", arg.display());
    }
    println!();
}


#[cfg(test)]
mod tests {
    // use std::path::PathBuf;
    // use crate::config::{ToolChain, Profile, ProjKind, Lang};
    // use super::*;

    /*
    #[test]
    pub fn compile_cmd_gcc_dbg1() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.o");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Debug,
            toolchain: ToolChain::Gcc,
            projkind: ProjKind::App,
            lang: Lang::Cpp(120),
            crtstatic: false,
            outdir: &out,
            defines: &vec![],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-std=c++20",
                "-c",
                "-O0",
                "-g",
                "-Isrc",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display()),
            ]
        );
    }

    #[test]
    pub fn compile_cmd_clang_dbg1() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.o");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Debug,
            toolchain: ToolChain::Clang,
            projkind: ProjKind::App,
            lang: Lang::Cpp(123),
            crtstatic: false,
            outdir: &out,
            defines: &vec![],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-std=c++23",
                "-c",
                "-O0",
                "-g",
                "-Isrc",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display()),
            ]
        );
    }

    #[test]
    pub fn compile_cmd_gcc_rel1() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.o");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Release,
            toolchain: ToolChain::Gnu,
            projkind: ProjKind::App,
            lang: Lang::Cpp(120),
            crtstatic: false,
            outdir: &out,
            defines: &vec![],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-std=c++20",
                "-c",
                "-O2",
                "-flto",
                "-Isrc",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display()),
            ]
        );
    }

    #[test]
    pub fn compile_cmd_gcc_rel2() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.o");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Release,
            toolchain: ToolChain::Gnu,
            projkind: ProjKind::App,
            lang: Lang::Cpp(120),
            crtstatic: true,
            outdir: &out,
            defines: &vec![],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-std=c++20",
                "-c",
                "-O2",
                "-flto",
                "-Isrc",
                src.to_str().unwrap(),
                &format!("-o{}", obj.display()),
            ]
        );
    }
    */
}
