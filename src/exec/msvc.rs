use std::path::{Path, PathBuf};
use super::{BuildInfo, PreCompHead};
use crate::{config::{Lang, ProjKind, Runtime, WarnLevel}, exec::output, Error, log_info_ln};


pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, _verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.compiler(info.lang.is_cpp());

    cmd.args(&info.comp_args);
    cmd.arg("/nologo");
    cmd.arg("/showIncludes");
    cmd.arg("/diagnostics:caret");
    // /WL (one line diagnostics)
    cmd.arg("/c");
    match info.lang {
        Lang::Cpp(123) => {
            cmd.arg("/std:c++latest");
        }
        Lang::Cpp(114) => {
            cmd.arg("/std:c++14");
        }
        Lang::C(120) => {
            cmd.arg("/std:clatest");
        }
        Lang::C(99) => {} // extensions on by default
        Lang::C(80) => {
            cmd.arg("/Za");
        }
        Lang::Cpp(_)|Lang::C(_) => {
            cmd.arg(format!("/std:{}", info.lang));
        }
    }
    if info.lang.is_cpp() {
        cmd.arg("/Zc:__cplusplus");
    } else {
        cmd.arg("/TC");
    }
    if info.lang.is_cpp() {
        cmd.arg("/EHsc");
    }
    match info.settings.runtime {
        Runtime::DynamicDebug   => { cmd.arg("/MDd"); }
        Runtime::DynamicRelease => { cmd.arg("/MD"); }
        Runtime::StaticDebug    => { cmd.arg("/MTd"); }
        Runtime::StaticRelease  => { cmd.arg("/MT"); }
    }
    match info.settings.opt_level {
        0 => { cmd.arg("/Od"); }
        1 => { cmd.arg("/Ox"); }
        2 => { cmd.arg("/O1"); }
        3 => { cmd.args([ "/O2", "/Oi" ]); }
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
        cmd.args([ "/Zi", "/FS", "/sdl" ]);
        cmd.arg(format!("/Fd:{}\\", info.outdir.display()));
        if !info.toolchain.is_clang() { cmd.arg("/Zf"); }
    }
    match info.settings.warn_level {
        WarnLevel::None  => { cmd.arg("/w"); }
        WarnLevel::Basic => { cmd.arg("/W1"); }
        WarnLevel::High  => { cmd.arg("/W4"); }
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    if info.settings.iso_compliant {
        cmd.arg("/permissive-");
    }
    if !info.settings.rtti && info.lang.is_cpp() {
        cmd.arg("/GR-");
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
    if echo { print_command(&cmd); }
    cmd
}

pub(super) fn link(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, _verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp());

    cmd.args(info.link_args);
    if let ProjKind::SharedLib{ implib } = info.projkind {
        cmd.arg("/DLL");
        if implib {
            cmd.arg(format!("/IMPLIB:{}", info.implib.unwrap().display()));
        }
    }
    cmd.arg("/MACHINE:X64");
    cmd.arg("/NOLOGO");
    if info.settings.aslr {
        cmd.arg("/DYNAMICBASE");
    }
    if info.settings.debug_info {
        cmd.arg("/DEBUG");
    }
    if info.settings.opt_linktime {
        cmd.arg("/LTCG");
        cmd.arg("/OPT:REF");
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    cmd.args(objs);
    cmd.args(info.libdirs .iter().map(|l| format!("/LIBPATH:{}", l.display())));
    cmd.args(info.archives);
    cmd.args(DEFAULT_LIBS);
    cmd.arg(format!("/OUT:{}", info.outfile.display()));

    if echo { print_command(&cmd); }
    if output::msvc_linker(&cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?, info.toolchain.is_clang()) {
        log_info_ln!("successfully built project: {}\n", info.outfile.display());
        Ok(true)
    } else {
        Err(Error::LinkerFail(info.outfile))
    }
}

pub(super) fn archive(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, _verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.archiver();

    cmd.args(info.link_args);
    cmd.arg("/MACHINE:X64");
    cmd.arg("/NOLOGO");
    if info.settings.opt_linktime {
        cmd.arg("/LTCG");
    }
    if info.settings.warn_as_error {
        cmd.arg("/WX");
    }
    cmd.args(objs);
    cmd.arg(format!("/OUT:{}", info.outfile.display()));

    if echo { print_command(&cmd); }
    if output::msvc_archiver(&cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?, info.toolchain.is_clang()) {
        log_info_ln!("successfully built project: {}\n", info.outfile.display());
        Ok(true)
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
    use std::path::PathBuf;
    use crate::config::{Lang, ProjKind, ToolChain};
    use super::*;

    #[test]
    pub fn compile_cmd_msvc_dbg() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.obj");

        let cmd = super::compile(&src, &obj,
            &BuildInfo::mock_debug(&out, ProjKind::App, Lang::Cpp(20), ToolChain::Msvc, None, false),
            &PreCompHead::None, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++20",
                "/Zc:__cplusplus",
                "/EHsc",
                "/MDd",
                "/Od",
                "/Zi",
                "/Fd:bin\\debug\\obj\\",
                "/FS",
                "/sdl",
                "/Zf",
                "/W1",
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

        let cmd = super::compile(&src, &obj,
            &BuildInfo::mock_debug(&out, ProjKind::App, Lang::Cpp(23), ToolChain::ClangMsvc, None, true),
            &PreCompHead::None, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++23",
                "/Zc:__cplusplus",
                "/EHsc",
                "/MTd",
                "/Od",
                "/Zi",
                "/Fd:bin\\debug\\obj\\",
                "/FS",
                "/sdl",
                "/W1",
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

        let cmd = super::compile(&src, &obj,
            &BuildInfo::mock_release(&out, ProjKind::App, Lang::Cpp(23), ToolChain::Msvc, None, false),
            &PreCompHead::None, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++23",
                "/Zc:__cplusplus",
                "/EHsc",
                "/MD",
                "/O2",
                "/Oi",
                "/GL",
                "/W1",
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

        let cmd = super::compile(&src, &obj,
            &BuildInfo::mock_release(&out, ProjKind::App, Lang::Cpp(23), ToolChain::Msvc, None, true),
            &PreCompHead::None, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/nologo",
                "/showIncludes",
                "/diagnostics:caret",
                "/c",
                "/std:c++23",
                "/Zc:__cplusplus",
                "/EHsc",
                "/MT",
                "/O2",
                "/Oi",
                "/GL",
                "/W1",
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
            ]
        );
    }
}

