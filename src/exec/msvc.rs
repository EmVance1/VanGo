use std::{ffi::OsString, io::Write, path::{Path, PathBuf}};
use super::{BuildInfo, PreCompHead};
use crate::{config::{ProjKind, WarnLevel, Runtime}, Error, exec::output, log_info_ln};


pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, _verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.compiler(info.lang.is_cpp());

    cmd.args(&info.comp_args);
    if info.lang.is_latest() {
        cmd.arg(format!("/std:{}", info.lang));
    } else if info.lang.is_cpp() {
        cmd.arg("/std:c++latest");
    } else {
        cmd.arg("/std:clatest");
    }
    if info.lang.is_cpp() {
        cmd.arg("/EHsc");
    }
    cmd.arg("/c");
    match info.settings.runtime {
        Runtime::DynamicDebug   => { cmd.arg("/MDd"); }
        Runtime::DynamicRelease => { cmd.arg("/MD"); }
        Runtime::StaticDebug    => { cmd.arg("/MTd"); }
        Runtime::StaticRelease  => { cmd.arg("/MT"); }
    }
    match info.settings.opt_level {
        0 => { cmd.arg("/O0"); }
        1 => { cmd.arg("/01"); }
        2 => { cmd.arg("/01"); }
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
        cmd.args([ "/Zi", "/Fd:bin\\debug\\obj\\", "/FS" ]);
    }
    cmd.arg("/diagnostics:caret");
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
    cmd.args(info.incdirs.iter().map(|inc| format!("/I{}", inc.display())));
    cmd.args(info.defines.iter().map(|def| format!("/D{def}")));
    match pch {
        PreCompHead::Create(h) => {
            let mut ycarg = OsString::from("/Yc");
            ycarg.push(h);
            let mut fparg = OsString::from("/Fp:");
            fparg.push(info.outdir.join("pch").join(h).with_extension("h.pch"));
            cmd.arg(ycarg);
            cmd.arg(fparg);
        }
        PreCompHead::Use(h) => {
            let mut yuarg = OsString::from("/Yu");
            yuarg.push(h);
            cmd.arg(yuarg);
            let mut fparg = OsString::from("/Fp:");
            fparg.push(info.outdir.join("pch").join(h).with_extension("h.pch"));
            cmd.arg(fparg);
        }
        _ => ()
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
    cmd.args(objs);
    cmd.args(info.libdirs .iter().map(|l| format!("/LIBPATH:{}", l.display())));
    cmd.args(info.archives);
    cmd.args(DEFAULT_LIBS);
    cmd.arg(format!("/OUT:{}", info.outfile.display()));

    if echo { print_command(&cmd); }
    if !output::msvc_linker(cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?) {
        Err(Error::LinkerFail(info.outfile))
    } else {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    }
}

pub(super) fn archive(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.archiver();

    cmd.args(info.link_args);
    cmd.arg("/MACHINE:X64");
    cmd.arg(format!("/OUT:{}", info.outfile.display()));
    cmd.args(objs);

    if echo { print_command(&cmd); }
    let output = cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?;
    if !output.status.success() {
        if verbose { let _ = std::io::stderr().write_all(&output.stderr); }
        let _ = std::io::stderr().write_all(&output.stdout);
        eprintln!();
        Err(Error::ArchiverFail(info.outfile))
    } else {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
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


#[cfg(test)]
mod tests {
    // use std::path::PathBuf;
    // use crate::config::{ToolChain, Profile, ProjKind, Lang};
    // use super::*;

    /*
    #[test]
    pub fn compile_cmd_msvc_dbg() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.obj");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Debug,
            toolchain: ToolChain::Msvc,
            projkind: ProjKind::App,
            lang: Lang::Cpp(120),
            crtstatic: false,
            outdir: &out,
            defines: &vec![ "UNICODE".to_string(), "_UNICODE".to_string() ],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/EHsc",
                "/std:c++20",
                "/c",
                "/MDd",
                "/Od",
                "/Zi",
                "/Fd:bin\\debug\\obj\\",
                "/FS",
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

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Debug,
            toolchain: ToolChain::Msvc,
            projkind: ProjKind::App,
            lang: Lang::Cpp(123),
            crtstatic: false,
            outdir: &out,
            defines: &vec![ "UNICODE".to_string(), "_UNICODE".to_string() ],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/EHsc",
                "/std:c++latest",
                "/c",
                "/MDd",
                "/Od",
                "/Zi",
                "/Fd:bin\\debug\\obj\\",
                "/FS",
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
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.obj");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Release,
            toolchain: ToolChain::Msvc,
            projkind: ProjKind::App,
            lang: Lang::Cpp(123),
            crtstatic: false,
            outdir: &out,
            defines: &vec![ "UNICODE".to_string(), "_UNICODE".to_string() ],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/EHsc",
                "/std:c++latest",
                "/c",
                "/MD",
                "/O2",
                "/Oi",
                "/GL",
                "/Isrc/",
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
        let out = PathBuf::from("bin/release");
        let obj = PathBuf::from("bin/release/obj/main.obj");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Release,
            toolchain: ToolChain::Msvc,
            projkind: ProjKind::App,
            lang: Lang::Cpp(123),
            crtstatic: true,
            outdir: &out,
            defines: &vec![ "UNICODE".to_string(), "_UNICODE".to_string() ],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/EHsc",
                "/std:c++latest",
                "/c",
                "/MT",
                "/O2",
                "/Oi",
                "/GL",
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
            ]
        );
    }
    */
}

