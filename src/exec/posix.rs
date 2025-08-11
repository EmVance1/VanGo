use super::{BuildInfo, CompileInfo};
use crate::{Error, fetch::FileInfo, log_info};
use std::{io::Write, process::Command};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo, verbose: bool) -> std::process::Command {
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

    if info.pch.is_some() {
        cmd.arg(format!("-Ibin/{}/pch/", info.config));
    }

    if info.config.is_release() {
        cmd.args(args.opt_profile_high());
    } else {
        cmd.args(args.opt_profile_none());
    }

    cmd.args(info.comp_args);
    cmd.stderr(std::process::Stdio::piped());
    if verbose {
        cmd.stdout(std::process::Stdio::piped());
    } else {
        cmd.stdout(std::process::Stdio::null());
    };
    if verbose { print_command(&cmd); }
    cmd
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolchain.archiver());
    cmd.arg("rcs");
    cmd.arg(&info.outfile.repr);
    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.args(info.link_args);
    if verbose { print_command(&cmd); }
    let output = cmd.output().unwrap();
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

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolchain.linker(info.lang.is_cpp()));
    cmd.args(objs.into_iter().map(|fi| fi.repr));
    cmd.args(["-o", &info.outfile.repr]);
    cmd.args(info.libdirs.iter().map(|l| format!("-L{l}")));
    cmd.args(info.links.iter().map(|l| format!("-l{l}")));
    cmd.args(info.link_args);
    if verbose { print_command(&cmd); }
    let output = cmd.output().unwrap();
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

pub(super) fn precompile_header(header: &str, info: &BuildInfo, verbose: bool) -> Option<std::process::Command> {
    let infile = format!("{}{}", info.srcdir, header);
    let outfile = format!("bin/{}/pch/{}.gch", info.config, header);

    if !std::fs::exists(&outfile).unwrap() ||
        (std::fs::metadata(&infile).unwrap().modified().unwrap() > std::fs::metadata(&outfile).unwrap().modified().unwrap())
    {
        log_info!("compiling precompiled header: {}{}", info.srcdir, header);

        let mut cmd = Command::new(info.toolchain.compiler(info.lang.is_cpp()));
        let args = info.toolchain.args();

        if info.lang.is_cpp() {
            cmd.arg(args.force_cpp());
            cmd.args(args.eh_default_cpp());
        }

        cmd.arg(args.std(&info.lang));
        cmd.arg(args.no_link());
        cmd.arg(&infile);
        cmd.arg(args.output(&outfile));

        cmd.args(info.incdirs.iter().map(|i| format!("{}{i}", args.I())));
        cmd.args(info.defines.iter().map(|d| format!("{}{d}", args.D())));

        if info.config.is_release() {
            cmd.args(args.opt_profile_high());
        } else {
            cmd.args(args.opt_profile_none());
        }

        if verbose {
            cmd.stdout(std::process::Stdio::piped());
        } else {
            cmd.stdout(std::process::Stdio::null());
        };
        if verbose { print_command(&cmd); }
        Some(cmd)
    } else {
        None
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

    #[test]
    pub fn compile_cmd_gcc_1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.o";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Debug,
            toolchain: ToolChain::Gnu,
            lang: Lang::Cpp(120),
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        }, false);

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
    pub fn compile_cmd_clang_1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.o";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Debug,
            toolchain: ToolChain::Clang,
            lang: Lang::Cpp(123),
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        }, false);

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
}
