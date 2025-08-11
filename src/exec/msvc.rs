use super::{BuildInfo, CompileInfo, PreCompHead};
use crate::{Error, fetch::FileInfo, log_info};
use std::{io::Write, process::Command};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo, pch: PreCompHead, verbose: bool) -> std::process::Command {
    let mut cmd = std::process::Command::new(info.toolchain.compiler(info.lang.is_cpp()));
    let args = info.toolchain.args();

    if info.lang.is_cpp() {
        cmd.arg(args.force_cpp());
        cmd.args(args.eh_default_cpp());
    }

    cmd.arg(args.std(&info.lang));
    cmd.arg(args.no_link());
    cmd.arg(src);
    cmd.arg(args.output(obj));

    cmd.args(info.incdirs.iter().map(|i| format!("{}{i}", args.I())));
    cmd.args(info.defines.iter().map(|d| format!("{}{d}", args.D())));

    if info.config.is_release() {
        cmd.args(args.opt_profile_high());
    } else {
        cmd.args(args.opt_profile_none());
    }

    match pch {
        PreCompHead::Create(h) => {
            cmd.arg(format!("/Yc{h}"));
            cmd.arg(format!("/Fp:{}pch/{}.pch", info.outdir, h));
        }
        PreCompHead::Use(h) => {
            cmd.arg(format!("/Yu{h}"));
            cmd.arg(format!("/Fp:{}pch/{}.pch", info.outdir, h));
        }
        _ => ()
    }

    cmd.args(info.comp_args);
    cmd.stdout(std::process::Stdio::piped());
    if verbose {
        cmd.stderr(std::process::Stdio::piped());
    } else {
        cmd.stderr(std::process::Stdio::null());
    };
    if verbose { print_command(&cmd); }
    cmd
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new("lib");
    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.arg(format!("/OUT:{}", info.outfile.repr));
    cmd.arg("/MACHINE:X64");
    cmd.args(info.link_args);
    if verbose { print_command(&cmd); }
    let output = cmd.output().unwrap();
    if !output.status.success() {
        std::io::stderr().write_all(&output.stdout).unwrap();
        eprintln!();
        Err(Error::ArchiverFail(info.outfile.repr))
    } else {
        log_info!("successfully built project {}\n", info.outfile.repr);
        Ok(true)
    }
}

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new("link");
    cmd.args(objs.into_iter().map(|fi| fi.repr));
    cmd.args(&info.links);
    cmd.args(DEFAULT_LIBS.iter().map(|l| format!("/DEFAULTLIB:{l}")));
    cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{l}")));
    cmd.arg(format!("/OUT:{}", info.outfile.repr));
    cmd.arg("/MACHINE:X64");
        // "/LTCG".to_string(),
        // format!("/{}", info.config.as_arg()),
        // "/OPT:REF".to_string(),
    if info.config.is_debug() {
        cmd.arg("/DEBUG");
    }
    cmd.args(info.link_args);
    if verbose { print_command(&cmd); }
    let output = cmd.output().unwrap();
    if !output.status.success() {
        std::io::stderr().write_all(&output.stdout).unwrap();
        eprintln!();
        Err(Error::LinkerFail(info.outfile.repr))
    } else {
        log_info!("successfully built project {}\n", info.outfile.repr);
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
    use crate::repr::{ToolChain, Config, Lang};

    #[test]
    pub fn compile_cmd_msvc_1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Debug,
            toolchain: ToolChain::Msvc,
            lang: Lang::Cpp(120),
            outdir: "bin/debug/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        }, crate::exec::PreCompHead::None, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/TP",
                "/EHsc",
                "/std:c++20",
                "/c",
                src,
                &format!("/Fo:{obj}"),
                "/Isrc/",
                "/Od",
                "/Zi",
                "/MDd",
                "/Fd:bin/debug/obj/vc143.pdb",
                "/FS",
            ]
        );
    }

    #[test]
    pub fn compile_cmd_msvc_2() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Debug,
            toolchain: ToolChain::Msvc,
            lang: Lang::Cpp(123),
            outdir: "bin/debug/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        }, crate::exec::PreCompHead::None, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/TP",
                "/EHsc",
                "/std:c++latest",
                "/c",
                src,
                &format!("/Fo:{obj}"),
                "/Isrc/",
                "/Od",
                "/Zi",
                "/MDd",
                "/Fd:bin/debug/obj/vc143.pdb",
                "/FS",
            ]
        );
    }
}

