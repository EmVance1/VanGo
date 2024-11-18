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

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo) -> Result<(), Error> {
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
    if !output.status.success() { Err(Error::LinkerFail(info.outfile.repr)) } else { Ok(()) }
}

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo) -> Result<(), Error> {
    let mut cmd = Command::new("link");
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
        Ok(())
    }
}

