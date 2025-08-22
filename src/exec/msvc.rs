use std::{ffi::OsString, io::Write, path::{Path, PathBuf}, process::Command};
use super::{BuildInfo, CompileInfo, PreCompHead};
use crate::{Error, log_info};


pub(super) fn compile_cmd(src: &Path, obj: &Path, info: CompileInfo, echo: bool, verbose: bool) -> std::process::Command {
    let mut cmd = std::process::Command::new(info.toolchain.compiler(info.lang.is_cpp()));
    let args = info.toolchain.args();
    cmd.args(info.toolchain.compiler_as_arg(info.lang.is_cpp()));
    cmd.args(info.comp_args);

    if info.lang.is_cpp() {
        cmd.args(args.eh_default_cpp());
    }

    cmd.arg(args.std(info.lang));
    cmd.arg(args.no_link());
    cmd.arg(src);
    cmd.arg(args.comp_output(&obj.to_string_lossy()));

    cmd.args(info.incdirs.iter().map(|inc| format!("{}{}", args.I(), inc.display())));
    cmd.args(info.defines.iter().map(|def| format!("{}{}", args.D(), def)));
    cmd.arg("/DUNICODE");
    cmd.arg("/D_UNICODE");

    if info.crtstatic {
        cmd.arg(args.crt_static(info.profile));
    } else {
        cmd.args(args.crt_dynamic(info.profile));
    }

    if info.profile.is_release() {
        cmd.args(args.opt_profile_high());
    } else {
        cmd.args(args.opt_profile_none());
    }

    match info.pch {
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

    cmd.stdout(std::process::Stdio::piped());
    if verbose {
        cmd.stderr(std::process::Stdio::piped());
    } else {
        cmd.stderr(std::process::Stdio::null());
    };
    if echo { print_command(&cmd); }
    cmd
}

pub(super) fn link_lib(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolchain.archiver());
    let args = info.toolchain.args();
    cmd.args(info.toolchain.archiver_as_arg());
    cmd.args(info.link_args);

    cmd.args(objs.into_iter().map(|o| format!("{}{}", args.l(), o.display())));
    cmd.arg(args.link_output(&info.outfile.to_string_lossy()));
    cmd.arg("/MACHINE:X64");
    if echo { print_command(&cmd); }
    let output = cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?;
    if !output.status.success() {
        if verbose { let _ = std::io::stderr().write_all(&output.stderr); }
        let _ = std::io::stderr().write_all(&output.stdout);
        eprintln!();
        Err(Error::ArchiverFail(info.outfile))
    } else {
        log_info!("successfully built project {}\n", info.outfile.display());
        Ok(true)
    }
}

pub(super) fn link_exe(objs: Vec<PathBuf>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolchain.linker(info.lang.is_cpp()));
    let args = info.toolchain.args();
    cmd.args(info.toolchain.linker_as_arg(info.lang.is_cpp()));
    cmd.args(info.link_args);

    cmd.args(objs);
    cmd.arg(args.link_output(&info.outfile.to_string_lossy()));
    cmd.args(info.libdirs.iter().map(|l| format!("{}{}", args.L(), l.display())));
    cmd.args(info.archives.iter().map(|l| format!("{}{}", args.l(), l.display())));

    cmd.args(DEFAULT_LIBS);
    cmd.arg("/MACHINE:X64");
    cmd.arg("/DYNAMICBASE");
        // "/LTCG".to_string(),
        // format!("/{}", info.config.as_arg()),
        // "/OPT:REF".to_string(),
    if info.profile.is_debug() {
        cmd.arg("/DEBUG");
    }
    if echo { print_command(&cmd); }
    let output = cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?;
    if !output.status.success() {
        if verbose { let _ = std::io::stderr().write_all(&output.stderr); }
        let _ = std::io::stderr().write_all(&output.stdout);
        eprintln!();
        Err(Error::LinkerFail(info.outfile))
    } else {
        log_info!("successfully built project {}\n", info.outfile.display());
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
    use std::path::PathBuf;
    use crate::repr::{ToolChain, Profile, Lang};
    use super::*;

    #[test]
    pub fn compile_cmd_msvc_dbg() {
        let src = PathBuf::from("src/main.cpp");
        let out = PathBuf::from("bin/debug");
        let obj = PathBuf::from("bin/debug/obj/main.obj");

        let cmd = super::compile_cmd(&src, &obj, super::CompileInfo {
            profile: &Profile::Debug,
            toolchain: ToolChain::Msvc,
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
                "/EHsc",
                "/std:c++20",
                "/c",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                "/MDd",
                "/Od",
                "/Zi",
                "/Fd:bin/debug/obj/vc143.pdb",
                "/FS",
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
                "/EHsc",
                "/std:c++latest",
                "/c",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                "/MDd",
                "/Od",
                "/Zi",
                "/Fd:bin/debug/obj/vc143.pdb",
                "/FS",
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
            lang: Lang::Cpp(123),
            crtstatic: false,
            outdir: &out,
            defines: &vec![],
            incdirs: &vec![ "src/".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/EHsc",
                "/std:c++latest",
                "/c",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
                "/Isrc/",
                "/DUNICODE",
                "/D_UNICODE",
                "/MD",
                "/O2",
                "/Oi",
                "/GL",
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
            lang: Lang::Cpp(123),
            crtstatic: true,
            outdir: &out,
            defines: &vec![],
            incdirs: &vec![ "src".into() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/EHsc",
                "/std:c++latest",
                "/c",
                src.to_str().unwrap(),
                &format!("/Fo:{}", obj.to_str().unwrap()),
                "/Isrc",
                "/DUNICODE",
                "/D_UNICODE",
                "/MT",
                "/O2",
                "/Oi",
                "/GL",
            ]
        );
    }
}

