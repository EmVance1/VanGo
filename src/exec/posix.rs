use super::{BuildInfo, CompileInfo};
use crate::{Error, fetch::FileInfo, log_info};
use std::{io::Write, path::PathBuf, process::Command};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo, verbose: bool) -> std::process::Command {
    let mut cmd = std::process::Command::new(info.toolchain.compiler(info.lang.is_cpp()));
    if info.lang.is_cpp() {
        cmd.arg("-xc++");
    }
    cmd.args([
        format!("-std={}", info.lang),
        "-c".to_string(),
        src.to_string(),
        "-o".to_string(),
        obj.to_string(),
    ]);
    cmd.args(info.incdirs.iter().map(|i| format!("-I{i}")));
    cmd.args(info.defines.iter().map(|d| format!("-D{d}")));
    if info.config.is_release() {
        cmd.arg("-O2");
        // args.push("/MD".to_string());
    } else {
        cmd.args(["-O0", "-g"]);
        // args.push("/MDd".to_string());
        // args.push(format!("/Fd:{}/vc143.pdb", info.outdir));
        // args.push("/FS".to_string());
    }
    cmd.args(info.comp_args);
    cmd.stderr(std::process::Stdio::piped());
    if verbose {
        cmd.stdout(std::process::Stdio::piped());
    } else {
        cmd.stdout(std::process::Stdio::null());
    };
    if verbose { print_command(info.toolchain.compiler(info.lang.is_cpp()), &cmd); }
    cmd
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolchain.archiver());
    cmd.arg("rcs");
    cmd.arg(&info.outfile.repr);
    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.args(info.link_args);
    if verbose { print_command(info.toolchain.archiver(), &cmd); }
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
    if verbose { print_command(info.toolchain.linker(info.lang.is_cpp()), &cmd); }
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
    let head_with_dir = format!("{}{}", info.srcdir, header);
    let cmpd = format!("{head_with_dir}.gch");
    let infile = FileInfo::from_path(&PathBuf::from(&head_with_dir));
    let outfile = FileInfo::from_path(&PathBuf::from(&cmpd));

    if !outfile.exists() || infile.modified().unwrap() > outfile.modified().unwrap() {
        let mut cmd = Command::new(info.toolchain.compiler(info.lang.is_cpp()));
        if info.lang.is_cpp() {
            cmd.arg("-xc++-header");
        }
        cmd.args([
            format!("-std={}", info.lang),
            head_with_dir,
            // "-o".to_string(),
            // obj.to_string(),
        ]);
        cmd.args(info.incdirs.iter().map(|i| format!("-I{i}")));
        cmd.args(info.defines.iter().map(|d| format!("-D{d}")));
        if info.config.is_release() {
            cmd.arg("-O2");
        } else {
            cmd.arg("-O0");
            cmd.arg("-g");
        }
        if verbose {
            cmd.stdout(std::process::Stdio::piped());
        } else {
            cmd.stdout(std::process::Stdio::null());
        };
        if verbose { print_command(info.toolchain.compiler(info.lang.is_cpp()), &cmd); }
        Some(cmd)
    } else {
        None
    }
}


fn print_command(exe: &str, cmd: &std::process::Command) {
    print!("{exe} ");
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
            outdir: "bin/debug/obj/",
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
                "-o",
                obj,
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
            outdir: "bin/debug/obj/",
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
                "-o",
                obj,
                "-Isrc/",
                "-O0",
                "-g",
            ]
        );
    }
}
