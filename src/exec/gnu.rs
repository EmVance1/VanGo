use super::{BuildInfo, PreCompHead, output};
use crate::{
    Error,
    config::{Artefact, Runtime, WarnLevel},
    log_info_ln,
};
use std::path::{Path, PathBuf};

pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.compiler(info.lang.is_cpp());

    cmd.args(&info.comp_args);
    if !info.toolchain.is_emcc() {
        // breaks miniaudio
        cmd.arg("-H"); // output configuration (see output parser)
    }
    cmd.arg(format!("-std={}", info.lang));
    if !info.toolchain.is_windows() && !info.toolchain.is_emcc() {
        match info.artefact {
            Artefact::Executable => {
                if info.settings.aslr {
                    cmd.arg("-fpie");
                } else {
                    cmd.arg("-fno-pie"); // explicitly disable ASLR on macos 10.7 (2011)
                }
            }
            Artefact::StaticLib | Artefact::SharedLib => {
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
    if info.settings.asan && (!info.toolchain.is_windows() || info.toolchain.is_llvm()) {
        cmd.arg("-fsanitize=address");
    }
    if info.settings.tsan && !info.toolchain.is_windows() {
        cmd.arg("-fsanitize=thread");
    }
    if info.settings.lsan && !info.toolchain.is_windows() {
        cmd.arg("-fsanitize=leak");
    }
    if info.settings.ubsan && (!info.toolchain.is_windows() || info.toolchain.is_llvm()) {
        cmd.arg("-fsanitize=undefined");
    }
    match pch {
        PreCompHead::Create(_) => {
            cmd.arg(format!("-x{}-header", if info.lang.is_cpp() { "c++" } else { "c" }));
        }
        PreCompHead::Use(header) => {
            if info.toolchain.is_llvm() {
                cmd.arg("-include-pch");
                cmd.arg(format!("{}/pch/{}.gch", info.outdir.display(), header.display()));
            } else {
                cmd.arg(format!("-I{}/pch", info.outdir.display()));
            }
        }
        PreCompHead::None => (),
    }
    cmd.args(info.incdirs.iter().map(|inc| format!("-I{}", inc.display())));
    cmd.args(info.defines.iter().map(|def| format!("-D{def}")));
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
    if let Artefact::SharedLib = info.artefact {
        if cfg!(target_os = "macos") {
            cmd.arg("-dynamiclib");
        } else {
            cmd.arg("-shared");
        }
        if let Some(implib) = info.implib {
            cmd.arg(format!("-Wl,--out-implib,{}", implib.display())); // forward to LINK.exe
        }
    }
    if !info.toolchain.is_emcc() {
        if info.settings.aslr {
            if cfg!(windows) {
                cmd.arg("-Wl,--dynamicbase"); // forward --dynamicbase to LINK.exe
            } else if let Artefact::Executable = info.artefact
                && cfg!(target_os = "linux")
            {
                cmd.arg("-pie");
            }
        } else if cfg!(target_os = "linux") {
            cmd.arg("-no-pie"); // ASLR often on by default
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
    if info.settings.asan && (!info.toolchain.is_windows() || info.toolchain.is_llvm()) {
        cmd.arg("-fsanitize=address");
    }
    if info.settings.tsan && !info.toolchain.is_windows() {
        cmd.arg("-fsanitize=thread");
    }
    if info.settings.lsan && !info.toolchain.is_windows() {
        cmd.arg("-fsanitize=leak");
    }
    if info.settings.ubsan && (!info.toolchain.is_windows() || info.toolchain.is_llvm()) {
        cmd.arg("-fsanitize=undefined");
    }
    if info.toolchain.is_emcc() {
        cmd.arg("-sUSE_SDL=2");
        cmd.arg("-sFULL_ES3");
    }
    cmd.args(objs);
    cmd.args(info.libdirs.iter().map(|l| format!("-L{}", l.display())));
    cmd.args(info.rpaths.iter().map(|l| format!("-Wl,-rpath,{}", l.display())));
    cmd.args(info.archives.iter().map(|l| format!("-l{}", l.display())));
    cmd.arg(format!("-o{}", info.outfile.display()));
    if verbose {
        cmd.arg("--verbose");
    }

    if echo {
        print_command(&cmd);
    }
    if output::gnu_linker(&cmd.output().map_err(|_| Error::LinkerNotFound(info.toolchain))?) {
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
    if output::gnu_archiver(&cmd.output().map_err(|_| Error::ArchiverNotFound(info.toolchain))?) {
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
