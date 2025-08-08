use crate::{Error, fetch::FileInfo, log_info};
use super::{CompileInfo, BuildInfo};
use std::{
    process::Command,
    io::Write,
};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo) -> Vec<String> {
    let mut args = vec![];
    if !info.is_c {
        args.push("-xc++".to_string());
    }
    args.extend([
        format!("-std={}", info.cppstd),
        "-c".to_string(),
        src.to_string(),
        "-o".to_string(),
        obj.to_string(),
    ]);
    args.extend(info.incdirs.iter().map(|i| format!("-I{}", i)));
    args.extend(info.defines.iter().map(|d| format!("-D{}", d)));
    if info.config.is_release() {
        args.push("-O2".to_string());
        // args.push("/MD".to_string());
    } else {
        args.push("-O0".to_string());
        args.push("-g".to_string());
        // args.push("/MDd".to_string());
        // args.push(format!("/Fd:{}/vc143.pdb", info.outdir));
        // args.push("/FS".to_string());
    }
    /*
    if let Some(outfile) = info.pch {
        let cmpd = format!("{}/{}.gch", info.outdir, outfile);
        args.push(format!("/Yu{}", outfile));
        args.push(format!("/Fp{}", cmpd));
    }
    */
    args
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolset.archiver());
    cmd.arg("rcs");
    cmd.arg(format!("{}", info.outfile.repr));
    cmd.args(objs.into_iter().map(|o| o.repr));
    let output = cmd.output().unwrap();
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
    println!();
    if !output.status.success() { Err(Error::LinkerFail(info.outfile.repr)) } else { Ok(true) }
}

pub(super) fn link_exe(objs: Vec<FileInfo>, info: BuildInfo) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolset.linker(info.is_c));
    cmd.args(objs.into_iter().map(|fi| fi.repr));
    cmd.args([
        "-o",
        &info.outfile.repr,
    ]);
    cmd.args(info.libdirs.iter().map(|l| format!("-L{}", l)));
    cmd.args(info.links.iter().map(|l| format!("-l{}", l)));
    let output = cmd.output().unwrap();
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
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
        let cmpd = format!("{}.gch", head_with_dir);
        let infile = FileInfo::from_path(&PathBuf::from(&head_with_dir));
        let outfile = FileInfo::from_path(&PathBuf::from(&cmpd));

        if !outfile.exists() || infile.modified().unwrap() > outfile.modified().unwrap() {
            let mut cmd = Command::new(info.toolset.compiler(info.is_c));
            if !info.is_c {
                cmd.arg("-xc++-header");
            }
            cmd.args([
                format!("-std={}", info.cppstd),
                head_with_dir,
                // "-o".to_string(),
                // obj.to_string(),
            ]);
            cmd.args(info.incdirs.iter().map(|i| format!("-I{}", i)));
            cmd.args(info.defines.iter().map(|d| format!("-D{}", d)));
            if info.config.is_release() {
                cmd.arg("-O2");
            } else {
                cmd.arg("-O0");
                cmd.arg("-g");
            }
            log_info_noline!("compiling precompiled header: {}\n", cmpd);
            let output = cmd.output().unwrap();
            if !output.status.success() {
                std::io::stderr().write_all(&output.stderr).unwrap();
            }
            println!();
        }
    }
}

