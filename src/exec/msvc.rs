use super::{BuildInfo, CompileInfo};
use crate::{Error, fetch::FileInfo, log_info};
use std::{io::Write, path::PathBuf, process::Command};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo) -> std::process::Command {
    let mut cmd = std::process::Command::new("cl");
    if info.cppstd.ends_with("23") {
        if info.cppstd.starts_with("c++") {
            cmd.arg("/std:c++latest");
        } else {
            cmd.arg("/std:clatest");
        }
    } else {
        cmd.arg(format!("/std:{}", info.cppstd.to_ascii_lowercase()));
    }
    cmd.args([
        "/c".to_string(),
        src.to_string(),
        format!("/Fo:{}", obj),
        "/EHsc".to_string(),
        // "/Gy".to_string(),
        // "/GL".to_string(),
        // "/Oi".to_string(),
    ]);
    cmd.args(info.incdirs.iter().map(|i| format!("/I{}", i)));
    cmd.args(info.defines.iter().map(|d| format!("/D{}", d)));
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
    if let Some(outfile) = info.pch {
        let cmpd = format!("{}/{}.pch", info.outdir, outfile);
        cmd.arg(format!("/Yu{}", outfile));
        cmd.arg(format!("/Fp{}", cmpd));
    }
    cmd.args(info.comp_args.iter().map(|s| s.to_string()));
    cmd
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo) -> Result<bool, Error> {
    let mut cmd = Command::new("lib");
    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.args([
        format!("/OUT:{}", info.outfile.repr),
        "/MACHINE:X64".to_string(),
    ]);
    cmd.args(info.link_args);
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

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo) -> Result<bool, Error> {
    let mut cmd = Command::new("link");
    cmd.args(objs.into_iter().map(|fi| fi.repr));
    cmd.args(&info.links);
    cmd.args(DEFAULT_LIBS);
    cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{}", l)));
    cmd.args([
        format!("/OUT:{}", info.outfile.repr),
        "/MACHINE:X64".to_string(),
        // "/LTCG".to_string(),
        // format!("/{}", info.config.as_arg()),
        // "/OPT:REF".to_string(),
    ]);
    cmd.args(info.link_args);
    if info.config.is_debug() {
        cmd.arg("/DEBUG");
    }
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

pub(super) fn precompile_header(header: &str, info: &BuildInfo) -> Option<std::process::Command> {
    let head_with_dir = format!("{}{}", info.srcdir, header);
    let cppf = format!("{}{}", info.srcdir, header.replace(".h", ".cpp"));
    let objt = format!("{}{}", info.outdir, header.replace(".h", ".obj"));
    let cmpd = format!(
        "{}{}.pch",
        info.outdir,
        header.replace(&info.srcdir, &info.outdir)
    );
    let infile = FileInfo::from_path(&PathBuf::from(&head_with_dir));
    let outfile = FileInfo::from_path(&PathBuf::from(&cmpd));

    if !outfile.exists() || infile.modified().unwrap() > outfile.modified().unwrap() {
        let mut cmd = Command::new("cl");
        cmd.args([
            cppf.clone(),
            "/c".to_string(),
            "/EHsc".to_string(),
            format!("/Yc{}", header),
            format!("/Fp:{}", cmpd),
            format!("/Fo:{}", objt),
            // "/Gy".to_string(),
            // "/GL".to_string(),
            // "/Oi".to_string(),
        ]);
        if info.cppstd.ends_with("23") {
            if info.cppstd.starts_with("c++") {
                cmd.arg("/std:c++latest");
            } else {
                cmd.arg("/std:clatest");
            }
        } else {
            cmd.arg(format!("/std:{}", info.cppstd.to_ascii_lowercase()));
        }
        cmd.args(info.incdirs.iter().map(|i| format!("/I{}", i)));
        cmd.args(info.defines.iter().map(|d| format!("/D{}", d)));
        if info.config.is_release() {
            cmd.args(["/MD", "/O2"]);
        } else {
            cmd.args(["/MDd", "/Od", "/Zi", "/FS"]);
            cmd.arg(format!("/Fd:{}vc143.pdb", info.outdir));
        }
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


#[cfg(test)]
mod tests {
    use crate::repr::{ToolSet, Config};

    #[test]
    pub fn compile_cmd_msvc_1() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";

        let cmd = super::compile_cmd(src, obj, super::CompileInfo {
            cppstd: "c++20",
            is_c: false,
            config: Config::Debug,
            toolset: ToolSet::MSVC,
            outdir: "bin/debug/obj/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        });

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
            cppstd: "c++23",
            is_c: false,
            config: Config::Debug,
            toolset: ToolSet::MSVC,
            outdir: "bin/debug/obj/",
            defines: &vec![],
            incdirs: &vec![ "src/".to_string() ],
            pch: &None,
            comp_args: &vec![],
        });

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
