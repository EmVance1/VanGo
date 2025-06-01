use crate::{Error, fetch::FileInfo, log_info};
use super::{CompileInfo, BuildInfo};
use std::{
    process::Command,
    io::Write,
};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo) -> Vec<String> {
    let mut args = vec![];
    args.extend(info.incdirs.iter().map(|i| format!("-I{}", i)));
    args.extend(info.defines.iter().map(|d| format!("-D{}", d)));
    if info.config.is_release() {
        args.push("-O2".to_string());
    }
    args.extend([
        format!("-std={}", info.cppstd),
        format!("-o {}", obj),
        "-c".to_string(),
        src.to_string(),
    ]);
    // if let Some(outfile) = info.pch {
    //     let cmpd = format!("{}/{}.pch", info.out_dir, outfile);
    //     args.push(format!("/Yu{}", outfile));
    //     args.push(format!("/Fp{}", cmpd));
    // }
    args
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo) -> Result<bool, Error> {
    let mut cmd = Command::new("ar rcs");
    cmd.args(objs.into_iter().map(|o| o.repr));
    cmd.args(info.links.iter().map(|l| format!("-l{}", l)));
    cmd.args(info.libdirs.iter().map(|l| format!("-L{}", l)));
    cmd.args([
        format!("-o {}", info.outfile.repr),
    ]);
    let output = cmd.output().unwrap();
    std::io::stdout().write_all(&output.stdout).unwrap();
    println!();
    if !output.status.success() { Err(Error::LinkerFail(info.outfile.repr)) } else { Ok(true) }
}

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo) -> Result<bool, Error> {
    let mut cmd = Command::new("g++");
    cmd.args(objs.into_iter().map(|fi| fi.repr));
    cmd.args(info.links.iter().map(|l| format!("-l{}", l)));
    cmd.args(info.libdirs.iter().map(|l| format!("-L{}", l)));
    cmd.args([
        format!("-o {}", info.outfile.repr),
    ]);
    let output = cmd.output().unwrap();
    std::io::stdout().write_all(&output.stdout).unwrap();
    println!();
    if !output.status.success() {
        Err(Error::LinkerFail(info.outfile.repr))
    } else {
        log_info!("successfully built project {}", info.outfile.repr);
        Ok(true)
    }
}


pub mod prep {
    use crate::{fetch::FileInfo, log_info_noline};
    use super::BuildInfo;
    use std::{
        process::Command,
        path::PathBuf,
        io::Write,
    };

    pub fn precompile_header(header: &str, info: &BuildInfo) {
        let head_with_dir = format!("{}{}", info.srcdir, header);
        let cmpd = format!("{}{}.gch", info.outdir, header.replace(&info.srcdir, &info.outdir));
        let infile = FileInfo::from_path(&PathBuf::from(&head_with_dir));
        let outfile = FileInfo::from_path(&PathBuf::from(&cmpd));

        if !outfile.exists() || infile.modified().unwrap() > outfile.modified().unwrap() {
            let mut cmd = Command::new("g++");
            cmd.args([
                "-c".to_string(),
                format!("-Yc{}", header),
                format!("-Fp{}", cmpd),
                format!("-std:{}", info.cppstd),
            ]);
            cmd.args(info.incdirs.iter().map(|i| format!("-I{}", i)));
            cmd.args(info.defines.iter().map(|d| format!("-D{}", d)));
            if info.config.is_release() {
                cmd.args(["/O2"]);
            }
            log_info_noline!("compiling precompiled header: ");
            std::io::stdout().write_all(&cmd.output().unwrap().stdout).unwrap();
            println!();
        }
    }
}

