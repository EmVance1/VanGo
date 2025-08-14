use std::{io::Write, process::Command};
use super::{BuildInfo, CompileInfo, PreCompHead};
use crate::{Error, fetch::FileInfo, log_info};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo, echo: bool, verbose: bool) -> std::process::Command {
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
    cmd.arg(args.comp_output(obj));

    cmd.args(info.incdirs.iter().map(|i| format!("{}{i}", args.I())));
    cmd.args(info.defines.iter().map(|d| format!("{}{d}", args.D())));

    match info.pch {
        PreCompHead::Use(_) => { cmd.arg(format!("-I{}pch/", info.outdir)); }
        _ => (),
    }

    if info.crtstatic {
        cmd.arg(args.crt_static(info.config));
    } else {
        cmd.args(args.crt_dynamic(info.config));
    }

    if info.config.is_release() {
        cmd.args(args.opt_profile_high());
    } else {
        cmd.args(args.opt_profile_none());
    }

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


pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolchain.archiver());
    // let args = info.toolchain.args();
    cmd.args(info.toolchain.archiver_as_arg());
    if verbose {
        cmd.arg("rcsv");
    } else {
        cmd.arg("rcs");
    }
    cmd.args(info.link_args);

    cmd.arg(&info.outfile.repr);
    cmd.args(objs.into_iter().map(|o| o.repr));
    if echo { print_command(&cmd); }
    let output = cmd.output().map_err(|_| Error::MissingArchiver(info.toolchain.to_string()))?;
    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr).unwrap();
        if verbose { std::io::stderr().write_all(&output.stdout).unwrap(); }
        eprintln!();
        Err(Error::ArchiverFail(info.outfile.repr))
    } else {
        log_info!("successfully built project {}\n", info.outfile.repr);
        Ok(true)
    }
}

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo, echo: bool, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolchain.linker(info.lang.is_cpp()));
    let args = info.toolchain.args();
    cmd.args(info.toolchain.linker_as_arg(info.lang.is_cpp()));
    cmd.args(info.link_args);

    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.arg(args.link_output(&info.outfile.repr));
    cmd.args(info.libdirs.iter().map(|l| format!("{}{}", args.L(), l)));
    cmd.args(info.links.iter().map(|l| format!("{}{}", args.l(), l)));

    if echo { print_command(&cmd); }
    if verbose { cmd.arg("--verbose"); }
    let output = cmd.output().map_err(|_| Error::MissingLinker(info.toolchain.to_string()))?;
    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr).unwrap();
        if verbose { std::io::stderr().write_all(&output.stdout).unwrap(); }
        eprintln!();
        Err(Error::LinkerFail(info.outfile.repr))
    } else {
        log_info!("successfully built project {}\n", info.outfile.repr);
        Ok(true)
    }
}


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
    use super::*;

    #[test]
    pub fn compile_cmd_gcc_dbg1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.o";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Debug,
            toolchain: ToolChain::Gnu,
            lang: Lang::Cpp(120),
            crtstatic: false,
            outdir: "bin/debug/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-xc++",
                "-std=c++20",
                "-c",
                src,
                &format!("-o{obj}"),
                "-Isrc/",
                "-O0",
                "-g",
            ]
        );
    }

    #[test]
    pub fn compile_cmd_clang_dbg1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.o";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Debug,
            toolchain: ToolChain::Clang,
            lang: Lang::Cpp(123),
            crtstatic: false,
            outdir: "bin/debug/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-xc++",
                "-std=c++23",
                "-c",
                src,
                &format!("-o{obj}"),
                "-Isrc/",
                "-O0",
                "-g",
            ]
        );
    }

    #[test]
    pub fn compile_cmd_gcc_rel1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.o";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Release,
            toolchain: ToolChain::Gnu,
            lang: Lang::Cpp(120),
            crtstatic: false,
            outdir: "bin/debug/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-xc++",
                "-std=c++20",
                "-c",
                src,
                &format!("-o{obj}"),
                "-Isrc/",
                "-O2",
            ]
        );
    }

    #[test]
    pub fn compile_cmd_gcc_rel2() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.o";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Release,
            toolchain: ToolChain::Gnu,
            lang: Lang::Cpp(120),
            crtstatic: true,
            outdir: "bin/debug/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &PreCompHead::None,
            comp_args: &vec![],
        }, false, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "-xc++",
                "-std=c++20",
                "-c",
                src,
                &format!("-o{obj}"),
                "-Isrc/",
                "-static",
                "-O2",
            ]
        );
    }
}
