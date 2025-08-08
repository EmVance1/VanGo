use crate::{Error, fetch::FileInfo, log_info};
use super::{CompileInfo, BuildInfo};
use std::{
    process::Command,
    io::Write,
};


pub(super) fn compile_cmd(src: &str, obj: &str, info: CompileInfo) -> std::process::Command {
    let mut cmd = std::process::Command::new(info.toolset.compiler(info.is_c));
    if !info.is_c {
        cmd.arg("-xc++");
    }
    cmd.args([
        format!("-std={}", info.cppstd),
        "-c".to_string(),
        src.to_string(),
        "-o".to_string(),
        obj.to_string(),
    ]);
    cmd.args(info.incdirs.iter().map(|i| format!("-I{}", i)));
    cmd.args(info.defines.iter().map(|d| format!("-D{}", d)));
    if info.config.is_release() {
        cmd.arg("-O2");
        // args.push("/MD".to_string());
    } else {
        cmd.args([ "-O0", "-g", ]);
        // args.push("/MDd".to_string());
        // args.push(format!("/Fd:{}/vc143.pdb", info.outdir));
        // args.push("/FS".to_string());
    }
    cmd.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    cmd
}

pub(super) fn link_lib(objs: Vec<FileInfo>, info: BuildInfo) -> Result<bool, Error> {
    let mut cmd = Command::new(info.toolset.archiver());
    cmd.arg("rcs");
    cmd.arg(format!("{}", info.outfile.repr));
    cmd.args(objs.into_iter().map(|o| o.repr));
    println!();
    let output = cmd.output().unwrap();
    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr).unwrap();
        Err(Error::LinkerFail(info.outfile.repr))
    } else {
        log_info!("successfully built project {}", info.outfile.repr);
        Ok(true)
    }
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
    println!();
    let output = cmd.output().unwrap();
    if !output.status.success() {
        std::io::stderr().write_all(&output.stderr).unwrap();
        Err(Error::LinkerFail(info.outfile.repr))
    } else {
        log_info!("successfully built project {}", info.outfile.repr);
        Ok(true)
    }
}


pub mod prep {
    use crate::{fetch::FileInfo, log_error, log_info_noline};
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
            let output = cmd.output().unwrap_or_else(|_| { println!(); log_error!("compiler not found for current target"); std::process::exit(1) });
            if !output.status.success() {
                std::io::stderr().write_all(&output.stderr).unwrap();
            }
            println!();
        }
    }
}

