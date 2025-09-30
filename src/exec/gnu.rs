use super::{BuildInfo, PreCompHead, output};
use crate::{
    Error,
    config::{ProjKind, Runtime, WarnLevel},
    log_info_ln,
};
use std::path::{Path, PathBuf};

pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.compiler(info.lang.is_cpp());

    cmd.args(&info.comp_args);
    if !info.toolchain.is_emcc() {
        cmd.arg("-H"); // output configuration (see output parser)
    }
    cmd.arg(format!("-std={}", info.lang));
    if !cfg!(windows) && !info.toolchain.is_emcc() {
        match info.projkind {
            ProjKind::App => {
                if info.settings.aslr {
                    cmd.arg("-fpie");
                } else {
                    cmd.arg("-fno-pie"); // explicitly disable ASLR on macos 10.7 (2011)
                }
            }
            ProjKind::StaticLib | ProjKind::SharedLib { .. } => {
                if info.settings.aslr {
                    cmd.arg("-fPIC");
                } else {
                    cmd.arg("-fno-pic"); // explicitly disable ASLR on macos 10.7 (2011)
                }
            }
        }
    }
    cmd.arg("-c");
    match info.settings.opt_level {
        0 => {
            cmd.arg("-O0");
        }
        1 => {
            cmd.arg("-O1");
        }
        2 => {
            cmd.arg("-O2");
        }
        3 => {
            cmd.arg("-O3");
        }
        _ => (),
    }
    if info.settings.opt_size {
        cmd.arg("-Os");
    }
    if info.settings.opt_speed {
        cmd.arg("-Ofast"); // problematic, causes unpredictable IEEE and others
    }
    if info.settings.opt_linktime {
        cmd.arg("-flto");
    }
    if info.settings.debug_info {
        cmd.arg("-g");
    }
    match info.settings.warn_level {
        WarnLevel::None => {
            cmd.arg("-w");
        }
        WarnLevel::Basic => {
            cmd.arg("-Wall");
        }
        WarnLevel::High => {
            cmd.args([
                "-Wall",
                "-Wextra",
                "-Wpedantic",
                "-Wconversion",
                "-Wsign-conversion",
                "-Wshadow",
                "-Wformat=2",
                "-Wnull-dereference",
                "-Wdouble-promotion",
                "-Wimplicit-fallthrough",
            ]);
        }
    }
    if info.settings.warn_as_error {
        cmd.arg("-Werror");
    }
    if info.settings.iso_compliant {
        cmd.arg("-pedantic-errors");
    }
    if info.lang.is_cpp() {
        if info.settings.no_rtti {
            cmd.arg("-fno-rtti"); // rtti on by default
        }
        if info.settings.no_except {
            cmd.arg("-fno-exceptions"); // exceptions on by default
        }
    }
    if info.settings.pthreads {
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
    if info.toolchain.is_emcc() {
        cmd.arg("-sUSE_SDL=2");
    }

    cmd.arg(src);
    cmd.arg(format!("-o{}", obj.display()));

    if verbose {
        cmd.arg("--verbose");
    }
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::piped());
    if echo {
        print_command(&cmd);
    }
    cmd
}

pub(super) fn link(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<(), Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp() || info.cpprt); // use g++/clang++ etc. when combining C and C++

    cmd.args(info.link_args);
    if let ProjKind::SharedLib { implib } = info.projkind {
        if cfg!(target_os = "macos") {
            cmd.arg("-dynamiclib");
        } else {
            cmd.arg("-shared");
        }
        if implib {
            cmd.arg(format!("-Wl,--out-implib,{}", info.implib.unwrap().display())); // forward to LINK.exe
        }
    }
    if !info.toolchain.is_emcc() {
        if info.settings.aslr {
            if cfg!(windows) {
                cmd.arg("-Wl,--dynamicbase"); // forward --dynamicbase to LINK.exe
            } else if let ProjKind::App = info.projkind
                && cfg!(target_os = "linux")
            {
                cmd.arg("-pie");
            }
        } else if cfg!(target_os = "macos") {
            // ASLR on by default since macos 10.7 (2011)
            cmd.arg("-Wl,-no_pie"); // forward -no_pie to ld
        }
    }
    if matches!(info.settings.runtime, Runtime::StaticDebug | Runtime::StaticRelease) {
        if info.lang.is_cpp() || info.cpprt {
            cmd.arg("-static-libstdc++");
        }
        cmd.arg("-static-libgcc");
    }
    if info.settings.opt_linktime {
        cmd.arg("-flto");
    }
    if info.settings.pthreads {
        cmd.arg("-pthread");
    }
    if info.toolchain.is_emcc() {
        cmd.arg("-sUSE_SDL=2");
        cmd.arg("-sFULL_ES3");
    }
    cmd.args(objs);
    cmd.args(info.libdirs.iter().map(|l| format!("-L{}", l.display())));
    cmd.args(info.archives.iter().map(|l| format!("-l{}", l.display())));
    cmd.arg(format!("-o{}", info.outfile.display()));
    if verbose {
        cmd.arg("--verbose");
    }

    if echo {
        print_command(&cmd);
    }
    if output::gnu_linker(&cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?) {
        log_info_ln!("successfully built project: {}\n", info.outfile.display());
        Ok(())
    } else {
        Err(Error::LinkerFail(info.outfile))
    }
}

pub(super) fn archive(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<(), Error> {
    let mut cmd = info.toolchain.archiver();

    if verbose {
        cmd.arg("rcsv");
    } else {
        cmd.arg("rcs");
    }
    cmd.arg(&info.outfile);
    cmd.args(info.link_args);
    cmd.args(objs);

    if echo {
        print_command(&cmd);
    }
    if output::gnu_archiver(&cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?) {
        log_info_ln!("successfully built project: {}\n", info.outfile.display());
        Ok(())
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
    use super::*;
    use crate::config::{Lang, ProjKind, ToolChain};
    use std::path::PathBuf;

    #[test]
    pub fn compile_cmd_gcc_dbg1() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.o");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_debug(&out, ProjKind::App, Lang::Cpp(20), ToolChain::Gcc, None, false),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        if cfg!(windows) {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    "-DUNICODE",
                    "-D_UNICODE",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        } else {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-fpie",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        }
    }

    #[test]
    pub fn compile_cmd_clang_dbg1() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.o");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_debug(&out, ProjKind::App, Lang::Cpp(23), ToolChain::ClangGnu, None, true),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        if cfg!(windows) {
            assert_eq!(
                cmd,
                [
                    "--target=x86_64-w64-mingw32",
                    "-H",
                    "-std=c++23",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    "-DUNICODE",
                    "-D_UNICODE",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        } else {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++23",
                    "-fpie",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        }
    }

    #[test]
    pub fn compile_cmd_gcc_rel1() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.o");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_release(&out, ProjKind::App, Lang::Cpp(20), ToolChain::Gcc, None, false),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        if cfg!(windows) {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-c",
                    "-O3",
                    "-flto",
                    "-Wall",
                    "-Isrc",
                    "-DUNICODE",
                    "-D_UNICODE",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        } else {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-fpie",
                    "-c",
                    "-O3",
                    "-flto",
                    "-Wall",
                    "-Isrc",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        }
    }

    #[test]
    pub fn compile_cmd_gcc_rel2() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.o");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_release(&out, ProjKind::App, Lang::Cpp(23), ToolChain::Gcc, None, true),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        if cfg!(windows) {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++23",
                    "-c",
                    "-O3",
                    "-flto",
                    "-Wall",
                    "-Isrc",
                    "-DUNICODE",
                    "-D_UNICODE",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        } else {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++23",
                    "-fpie",
                    "-c",
                    "-O3",
                    "-flto",
                    "-Wall",
                    "-Isrc",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        }
    }

    #[test]
    pub fn compile_cmd_gcc_staticlib() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.o");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_debug(&out, ProjKind::StaticLib, Lang::Cpp(20), ToolChain::Gcc, None, true),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        if cfg!(windows) {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    "-DUNICODE",
                    "-D_UNICODE",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        } else {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-fPIC",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        }
    }

    #[test]
    pub fn compile_cmd_gcc_sharedlib() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.o");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_debug(
                &out,
                ProjKind::SharedLib { implib: true },
                Lang::Cpp(20),
                ToolChain::Gcc,
                None,
                true,
            ),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        if cfg!(windows) {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    "-DUNICODE",
                    "-D_UNICODE",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        } else {
            assert_eq!(
                cmd,
                [
                    "-H",
                    "-std=c++20",
                    "-fPIC",
                    "-c",
                    "-O0",
                    "-g",
                    "-Wall",
                    "-Isrc",
                    src.to_str().unwrap(),
                    &format!("-o{}", obj.display())
                ]
            );
        }
    }
}
