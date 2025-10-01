use super::{BuildInfo, PreCompHead};
use crate::{
    Error,
    config::{Lang, ProjKind, Runtime, WarnLevel},
    exec::output,
    log_info_ln,
};
use std::path::{Path, PathBuf};

pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, _verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.compiler(info.lang.is_cpp());

    cmd.args(&info.comp_args);
    cmd.arg("/nologo"); // output configuration (see output parser)
    cmd.arg("/showIncludes"); // "
    cmd.arg("/diagnostics:caret"); // "
    // /WL (one line diagnostics)   // "
    cmd.arg("/c");
    match info.lang {
        Lang::Cpp(123) => {
            cmd.arg("/std:c++latest");
        }
        Lang::Cpp(n) if n < 114 => {
            cmd.arg("/std:c++14");
        }
        Lang::C(120) => {
            cmd.arg("/std:clatest");
        }
        Lang::C(99) => {} // extensions on by default
        Lang::C(89) => {
            cmd.arg("/Za"); // disable MS pseudo C99 extensions
        }
        Lang::Cpp(_) | Lang::C(_) => {
            cmd.arg(format!("/std:{}", info.lang));
        }
    }
    if info.lang.is_cpp() {
        cmd.arg("/Zc:__cplusplus"); // correctly define '__cplusplus' macro
    } else {
        cmd.arg("/TC"); // enforce C for all sources (needed for MS pseudo C99)
    }
    match info.settings.runtime {
        Runtime::DynamicDebug => {
            if info.settings.asan && info.toolchain.is_clang() {
                cmd.arg("/MD");
            } else {
                cmd.arg("/MDd");
            }
        }
        Runtime::DynamicRelease => {
            cmd.arg("/MD");
        }
        Runtime::StaticDebug => {
            if info.settings.asan && info.toolchain.is_clang() {
                cmd.arg("/MT");
            } else {
                cmd.arg("/MTd");
            }
        }
        Runtime::StaticRelease => {
            cmd.arg("/MT");
        }
    }
    match info.settings.opt_level {
        0 => {
            cmd.arg("/Od");
        }
        1 => {
            cmd.arg("/Ox");
        }
        2 => {
            cmd.arg("/O1");
        }
        3 => {
            cmd.args(["/O2", "/Oi"]);
        }
        _ => (),
    }
    if info.settings.opt_size {
        cmd.arg("/Os");
    }
    if info.settings.opt_speed {
        cmd.arg("/Ot");
    }
    if info.settings.opt_linktime {
        cmd.arg("/GL");
    }
    if info.settings.debug_info {
        cmd.args(["/Zi", "/FS", "/sdl"]); // debug info, thread safe, extra security
        cmd.arg(format!("/Fd:{}\\", info.outdir.display())); // PDB output dir
        if !info.toolchain.is_clang() {
            cmd.arg("/Zf");
        } // faster PDB gen??
    }
    match info.settings.warn_level {
        WarnLevel::None => {
            cmd.arg("/w");
        }
        WarnLevel::Basic => {
            cmd.arg("/W1");
        }
        WarnLevel::High => {
            cmd.arg("/W4");
        }
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    if info.settings.iso_compliant {
        cmd.arg("/permissive-");
    }
    if info.lang.is_cpp() {
        if info.settings.no_rtti {
            cmd.arg("/GR-"); // rtti on by default
        }
        if info.settings.no_except {
            cmd.arg("/EHsc-");
        } else {
            cmd.arg("/EHsc"); // default C++ exception handling, extern "C" -> noexcept
        }
    }
    if info.settings.asan {
        cmd.arg("-fsanitize=address");
    }
    // most sanitizers not supported by MSVC
    /*
    if info.settings.tsan {
        cmd.arg("-fsanitize=thread");
    }
    if info.settings.lsan {
        cmd.arg("-fsanitize=leak");
    }
    */
    if info.settings.ubsan && info.toolchain.is_clang() {
        cmd.arg("-fsanitize=undefined");
    }
    cmd.args(info.incdirs.iter().map(|inc| format!("/I{}", inc.display())));
    cmd.args(info.defines.iter().map(|def| format!("/D{def}")));
    match pch {
        PreCompHead::Create(h) => {
            cmd.arg(format!("/Yc{}", h.display()));
            cmd.arg(format!("/Fp:{}", info.outdir.join("pch").join(h).with_extension("h.pch").display()));
        }
        PreCompHead::Use(h) => {
            cmd.arg(format!("/Yu{}", h.display()));
            cmd.arg(format!("/Fp:{}", info.outdir.join("pch").join(h).with_extension("h.pch").display()));
        }
        PreCompHead::None => (),
    }

    cmd.arg(src);
    cmd.arg(format!("/Fo:{}", obj.display()));

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    if echo {
        print_command(&cmd);
    }
    cmd
}

pub(super) fn link(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, _verbose: bool) -> Result<(), Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp());

    cmd.args(info.link_args);
    cmd.arg("/NOLOGO");
    cmd.arg("/MACHINE:X64");
    if let ProjKind::SharedLib { implib } = info.projkind {
        cmd.arg("/DLL");
        if implib {
            cmd.arg(format!("/IMPLIB:{}", info.implib.unwrap().display()));
        }
    }
    if info.settings.aslr {
        cmd.arg("/DYNAMICBASE");
    }
    if info.settings.debug_info {
        cmd.arg("/DEBUG");
    }
    if info.settings.opt_linktime {
        cmd.arg("/LTCG"); // link-time codegen, iff /GL
        cmd.arg("/OPT:REF"); // strip unreferenced symbols
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    cmd.args(objs);
    cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{}", l.display())));
    cmd.args(info.archives);
    if info.settings.asan && info.toolchain.is_clang() {
        cmd.arg("clang_rt.asan_dynamic-x86_64.lib");
        cmd.arg("clang_rt.asan_dynamic_runtime_thunk-x86_64.lib");
    }
    cmd.args(DEFAULT_LIBS);
    cmd.arg(format!("/OUT:{}", info.outfile.display()));

    if echo {
        print_command(&cmd);
    }
    if output::msvc_linker(
        &cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?,
        info.toolchain.is_clang(),
    ) {
        log_info_ln!("successfully built project: {}\n", info.outfile.display());
        Ok(())
    } else {
        Err(Error::LinkerFail(info.outfile))
    }
}

pub(super) fn archive(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, _verbose: bool) -> Result<(), Error> {
    let mut cmd = info.toolchain.archiver();

    cmd.args(info.link_args);
    cmd.arg("/NOLOGO");
    cmd.arg("/MACHINE:X64");
    if info.settings.opt_linktime {
        cmd.arg("/LTCG"); // link-time codegen HINT, iff /GL
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    cmd.args(objs);
    cmd.arg(format!("/OUT:{}", info.outfile.display()));

    if echo {
        print_command(&cmd);
    }
    if output::msvc_archiver(
        &cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?,
        info.toolchain.is_clang(),
    ) {
        log_info_ln!("successfully built project: {}\n", info.outfile.display());
        Ok(())
    } else {
        Err(Error::ArchiverFail(info.outfile))
    }
}

const DEFAULT_LIBS: &[&str] = &[
    "kernel32.lib",
    "user32.lib",
    "winspool.lib",
    "comdlg32.lib",
    "advapi32.lib",
    "shell32.lib",
    "ole32.lib",
    "oleaut32.lib",
    "uuid.lib",
    "odbc32.lib",
    "odbccp32.lib",
    "gdi32.lib",
];

fn print_command(cmd: &std::process::Command) {
    print!("{} ", cmd.get_program().to_string_lossy());
    for arg in cmd.get_args() {
        print!("{} ", arg.to_string_lossy());
    }
    println!();
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use crate::config::{Lang, ProjKind, ToolChain};
    use std::path::PathBuf;

    #[test]
    pub fn compile_cmd_msvc_dbg() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.obj");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_debug(&out, ProjKind::App, Lang::Cpp(120), ToolChain::Msvc, None, false),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(
            cmd,
            [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++20",
                "/Zc:__cplusplus",
                "/MDd",
                "/Od",
                "/Zi",
                "/FS",
                "/sdl",
                "/Fd:bin\\debug\\",
                "/Zf",
                "/W1",
                "/EHsc",
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
            ]
        );
    }

    #[test]
    pub fn compile_cmd_msvc_dbg2() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.obj");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_debug(&out, ProjKind::App, Lang::Cpp(123), ToolChain::ClangMsvc, None, true),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(
            cmd,
            [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++latest",
                "/Zc:__cplusplus",
                "/MTd",
                "/Od",
                "/Zi",
                "/FS",
                "/sdl",
                "/Fd:bin\\debug\\",
                "/W1",
                "/EHsc",
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
            ]
        );
    }

    #[test]
    pub fn compile_cmd_msvc_rel1() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.obj");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_release(&out, ProjKind::App, Lang::Cpp(123), ToolChain::Msvc, None, false),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(
            cmd,
            [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++latest",
                "/Zc:__cplusplus",
                "/MD",
                "/O2",
                "/Oi",
                "/GL",
                "/W1",
                "/EHsc",
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
            ]
        );
    }

    #[test]
    pub fn compile_cmd_msvc_rel2() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.obj");

        let cmd = super::compile(
            &src,
            &obj,
            &BuildInfo::mock_release(&out, ProjKind::App, Lang::Cpp(123), ToolChain::Msvc, None, true),
            &PreCompHead::None,
            false,
            false,
        );

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(
            cmd,
            [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++latest",
                "/Zc:__cplusplus",
                "/MT",
                "/O2",
                "/Oi",
                "/GL",
                "/W1",
                "/EHsc",
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
            ]
        );
    }
}
