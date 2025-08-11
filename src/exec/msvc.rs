use super::{BuildInfo, CompileInfo};
use crate::{Error, fetch::FileInfo, log_info};
use std::{io::Write, process::Command};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo, verbose: bool) -> std::process::Command {
    let mut cmd = std::process::Command::new("cl");
    if info.lang.is_latest() {
        if info.lang.is_cpp() {
            cmd.arg("/std:c++latest");
        } else {
            cmd.arg("/std:clatest");
        }
    } else {
        cmd.arg(format!("/std:{}", info.lang));
    }
    cmd.args([
        "/c".to_string(),
        src.to_string(),
        format!("/Fo:{obj}"),
        "/EHsc".to_string(),
        // "/Gy".to_string(),
        // "/GL".to_string(),
        // "/Oi".to_string(),
    ]);
    cmd.args(info.incdirs.iter().map(|i| format!("/I{i}")));
    cmd.args(info.defines.iter().map(|d| format!("/D{d}")));
    if info.config.is_release() {
        cmd.args(["/MD", "/O2"]);
    } else {
        cmd.args([
            "/MDd".to_string(),
            "/Od".to_string(),
            "/Zi".to_string(),
            format!("/Fd:{}vc143.pdb", info.outdir),
            "/FS".to_string(),
        ]);
    }
    if let Some(infile) = info.pch {
        let outfile = format!("bin/{}/pch/{}.pch", info.config, infile);
        cmd.arg(format!("/Yu{infile}"));
        cmd.arg(format!("/Fp{outfile}"));
    }
    cmd.args(info.comp_args);
    cmd.stdout(std::process::Stdio::piped());
    if verbose {
        cmd.stderr(std::process::Stdio::piped());
    } else {
        cmd.stderr(std::process::Stdio::null());
    };
    if verbose { print_command("cl.exe", &cmd); }
    cmd
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo, verbose: bool) -> Result<bool, Error> {
    let mut cmd = Command::new("lib");
    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.arg(format!("/OUT:{}", info.outfile.repr));
    cmd.arg("/MACHINE:X64");
    cmd.args(info.link_args);
    if verbose { print_command("lib.exe", &cmd); }
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
    if verbose { print_command("link.exe", &cmd); }
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

pub(super) fn precompile_header(header: &str, info: &BuildInfo, verbose: bool) -> Option<std::process::Command> {
    let cppf = format!("bin/{}/pch/{}", info.config, header.replace(".h", ".cpp"));
    if !std::fs::exists(&cppf).unwrap() {
        std::fs::write(&cppf, format!("#include \"{}\"", header)).unwrap();
    }

    let infile = format!("{}{}", info.srcdir, header);
    let outpch = format!("bin/{}/pch/{}.pch", info.config, header);
    let outobj = format!("{}{}", info.outdir, header.replace(".h", ".obj"));

    if !std::fs::exists(&outpch).unwrap() ||
        (std::fs::metadata(&infile).unwrap().modified().unwrap() > std::fs::metadata(&outpch).unwrap().modified().unwrap())
    {
        let mut cmd = Command::new("cl");
        cmd.args([
            cppf,
            "/c".to_string(),
            "/EHsc".to_string(),
            format!("/Yc{header}"),
            format!("/Fp:{outpch}"),
            format!("/Fo:{outobj}"),
            // "/Gy".to_string(),
            // "/GL".to_string(),
            // "/Oi".to_string(),
        ]);
        if info.lang.is_latest() {
            if info.lang.is_cpp() {
                cmd.arg("/std:c++latest");
            } else {
                cmd.arg("/std:clatest");
            }
        } else {
            cmd.arg(format!("/std:{}", info.lang));
        }
        cmd.args(info.incdirs.iter().map(|i| format!("/I{i}")));
        cmd.args(info.defines.iter().map(|d| format!("/D{d}")));
        if info.config.is_release() {
            cmd.args(["/MD", "/O2"]);
        } else {
            cmd.args(["/MDd", "/Od", "/Zi", "/FS"]);
            cmd.arg(format!("/Fd:{}vc143.pdb", info.outdir));
        }
        cmd.stdout(std::process::Stdio::piped());
        if verbose {
            cmd.stderr(std::process::Stdio::piped());
        } else {
            cmd.stderr(std::process::Stdio::null());
        };
        if verbose { print_command("cl.exe", &cmd); }
        Some(cmd)
    } else {
        None
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
    pub fn compile_cmd_msvc_1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            config: Config::Debug,
            toolchain: ToolChain::Msvc,
            lang: Lang::Cpp(120),
            outdir: "bin/debug/obj/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        }, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/std:c++20",
                "/c",
                src,
                &format!("/Fo:{obj}"),
                "/EHsc",
                "/Isrc/",
                "/MDd",
                "/Od",
                "/Zi",
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
            outdir: "bin/debug/obj/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        }, false);

        let cmd: Vec<_> = cmd.get_args().collect();
        assert_eq!(cmd, [
                "/std:c++latest",
                "/c",
                src,
                &format!("/Fo:{obj}"),
                "/EHsc",
                "/Isrc/",
                "/MDd",
                "/Od",
                "/Zi",
                "/Fd:bin/debug/obj/vc143.pdb",
                "/FS",
            ]
        );
    }
}

