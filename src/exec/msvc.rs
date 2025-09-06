use std::{ffi::OsString, io::Write, path::{Path, PathBuf}};
use super::{BuildInfo, PreCompHead};
use crate::{config::{Runtime, WarnLevel}, exec::output::on_msvc_link_finish, log_info_ln, Error};


pub(super) fn compile(src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, echo: bool, _verbose: bool) -> std::process::Command {
    let mut cmd = info.toolchain.compiler(info.lang.is_cpp());
    let args = info.toolchain.args();

    cmd.args(&info.comp_args);
    if info.lang.is_cpp() {
        cmd.args(args.eh_default_cpp());
    }
    cmd.arg(args.std(info.lang));
    cmd.arg(args.comp_only());
    match info.settings.runtime {
        Runtime::DynamicDebug   => { cmd.arg("/MDd"); }
        Runtime::DynamicRelease => { cmd.arg("/MD"); }
        Runtime::StaticDebug    => { cmd.arg("/MTd"); }
        Runtime::StaticRelease  => { cmd.arg("/MT"); }
    }
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
    cmd.args(info.defines.iter().map(|def| format!("/D{}", def)));
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
    cmd.arg(args.comp_output(&obj.to_string_lossy()));

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    if echo { print_command(&cmd); }
    cmd
}

pub(super) fn link_exe(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, _verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp());
    let args = info.toolchain.args();

    cmd.args(info.link_args);
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
    cmd.args(info.libdirs.iter().map(|l| format!("{}{}", args.L(), l.display())));
    cmd.args(info.archives.iter().map(|l| format!("{}{}", args.l(), l.display())));
    cmd.args(DEFAULT_LIBS);
    cmd.arg(args.link_output(&info.outfile.to_string_lossy()));

    if echo { print_command(&cmd); }
    let output = cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?;
    if !on_msvc_link_finish(output) {
        Err(Error::LinkerFail(info.outfile))
    } else {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    }
}

pub(super) fn link_shared_lib(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, _verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.linker(info.lang.is_cpp());
    let args = info.toolchain.args();

    cmd.args(info.link_args);
    cmd.arg("/DLL");
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
    cmd.args(info.libdirs.iter().map(|l| format!("{}{}", args.L(), l.display())));
    cmd.args(info.archives.iter().map(|l| format!("{}{}", args.l(), l.display())));
    // cmd.args(DEFAULT_LIBS);
    cmd.arg(args.link_output(&info.outfile.to_string_lossy()));
    if let Some(lib) = info.implib {
        cmd.arg(format!("/IMPLIB:{}", lib.display()));
    }

    if echo { print_command(&cmd); }
    let output = cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?;
    if !on_msvc_link_finish(output) {
        Err(Error::LinkerFail(info.outfile))
    } else {
        log_info_ln!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    }
}

pub(super) fn link_static_lib(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = info.toolchain.archiver();
    let args = info.toolchain.args();

    cmd.arg(args.link_output(&info.outfile.to_string_lossy()));
    cmd.args(info.link_args);
    cmd.arg("/MACHINE:X64");
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

