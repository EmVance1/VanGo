use std::{io::Write, path::{Path, PathBuf}};
use super::{BuildInfo, PreCompHead};
use crate::{config::{ProjKind, WarnLevel}, Error, exec::output, log_info_ln};


pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.compiler(info.lang.is_cpp());

    cmd.args(&info.comp_args);
    cmd.arg(format!("-std={}", info.lang));
    if info.settings.aslr && !cfg!(windows) {
        match info.projkind {
            ProjKind::App|ProjKind::StaticLib => { cmd.arg("-fpie"); },
            ProjKind::SharedLib{..} => { cmd.arg("-fPIC"); },
        }
    }
    cmd.arg("-c");
    match info.settings.opt_level {
        0 => { cmd.arg("-O0"); }
        1 => { cmd.arg("-O1"); }
        2 => { cmd.arg("-O2"); }
        3 => { cmd.arg("-O3"); }
        _ => (),
    }
    if info.settings.opt_size {
        cmd.arg("-Os");
    }
    if info.settings.opt_speed {
        cmd.arg("-Ofast");
    }
    if info.settings.opt_linktime {
        cmd.arg("-flto");
    }
    if info.settings.debug_info {
        cmd.arg("-g");
    }
    match info.settings.warn_level {
        WarnLevel::None  => { cmd.arg("-w"); }
        WarnLevel::Basic => { cmd.arg("-Wall"); }
        WarnLevel::High  => { cmd.args([ "-Wall", "-Wextra", "-Wpedantic", "-Wconversion", "-Wsign-conversion", "-Wshadow",
                                     "-Wformat=2", "-Wnull-dereference", "-Wdouble-promotion", "-Wimplicit-fallthrough" ]); }
    }
    if info.settings.warn_as_error {
        cmd.arg("-Werror");
    }
    if info.settings.iso_compliant {
        cmd.arg("-pedantic-errors");
    }
    if !info.settings.rtti {
        cmd.arg("-fnortti");
    }
    if info.settings.pthread {
        cmd.arg("-pthread");
    }
    cmd.args(info.incdirs.iter().map(|inc| format!("-I{}", inc.display())));
    cmd.args(info.defines.iter().map(|def| format!("-D{def}")));
    match pch {
        PreCompHead::Create(_) => {
            cmd.arg(format!("-x{}-header", if info.lang.is_cpp() { "c++" } else { "c" }));
        }
        PreCompHead::Use(_) => {
            if info.toolchain.is_clang() {
                cmd.arg("-include-pch");
            }
            cmd.arg(format!("-I{}/pch", info.outdir.display()));
        }
        PreCompHead::None => (),
    }
    // /showIncludes

    // -H

    cmd.arg(src);
    cmd.arg(format!("-o{}", obj.display()));

    if verbose { cmd.arg("--verbose"); }
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::piped());
    if echo { print_command(&cmd); }
    cmd
}

pub(super) fn link(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp() || info.cpprt);

    cmd.args(info.link_args);
    if let ProjKind::SharedLib{ implib } = info.projkind {
        if cfg!(target_os = "macos") {
            cmd.arg("-dynamiclib");
        } else {
            cmd.arg("-shared");
        }
        if implib {
            cmd.arg(format!("-Wl,--out-implib,{}", info.implib.unwrap().display()));
        }
    }
    if info.settings.aslr {
        if cfg!(windows) {
            cmd.arg("-Wl,--dynamicbase");
        } else if let ProjKind::App = info.projkind {
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
    if info.settings.pthread {
        cmd.arg("-pthread");
    }

    cmd.args(objs);
    cmd.args(info.libdirs .iter().map(|l| format!("-L{}", l.display())));
    cmd.args(info.archives.iter().map(|l| format!("-l{}", l.display())));
    cmd.arg(format!("-o{}", info.outfile.display()));
    if verbose { cmd.arg("--verbose"); }

    if echo { print_command(&cmd); }
    if output::gnu_linker(&cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?) {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    } else {
        Err(Error::LinkerFail(info.outfile))
    }
}

pub(super) fn archive(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.archiver();

    if verbose {
        cmd.arg("rcsv");
    } else {
        cmd.arg("rcs");
    }
    cmd.arg(&info.outfile);
    cmd.args(info.link_args);
    cmd.args(objs);

    if echo { print_command(&cmd); }
    if output::gnu_archiver(&cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?) {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    } else {
        Err(Error::ArchiverFail(info.outfile))
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
