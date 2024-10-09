mod incremental;
mod prep;

use std::{io::Write, path::PathBuf, process::Command};
use crate::{input::Config, fetch::FileInfo};
use incremental::IncrementalBuild;


#[derive(Debug)]
pub struct BuildInfo {
    pub cppstd: String,
    pub config: Config,
    pub sdir: String,
    pub odir: String,
    pub defines: Vec<String>,
    pub sources: Vec<FileInfo>,
    pub headers: Vec<FileInfo>,
    pub incdirs: Vec<String>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
    pub pch: Option<String>,
    pub oplevel: String,
    pub outfile: FileInfo,
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
];

pub fn run_build(info: BuildInfo) -> Result<(), ()> {
    println!("[mscmp:  info] starting build for \"{}\":", info.outfile.repr);
    prep::assert_out_dirs(&PathBuf::from(&info.sdir), &info.sdir, &info.odir);
    let pch = if let Some(pch) = &info.pch {
        Some(prep::precompile_header(&PathBuf::from(pch), &info))
    } else {
        None
    };

    match IncrementalBuild::calc(&info) {
        IncrementalBuild::NoBuild => {
            println!("[mscmp:  info] build up to date for \"{}\"", info.outfile.repr);
            return Ok(())
        }
        IncrementalBuild::BuildSelective(elems) => {
            for (src, obj) in elems {
                println!("[mscmp:  info] compiling: {}", src.repr);
                let output = compile_cmd(&src.repr, &obj.repr, &info, &pch).output().unwrap();
                std::io::stdout().write_all(&output.stdout).unwrap();
                println!();
                if !output.status.success() { return Err(()) }
            }
        }
        IncrementalBuild::BuildAll => {
            for src in &info.sources {
                let obj = src.repr.replace(&info.sdir, &info.odir).replace(".cpp", ".obj");
                println!("[mscmp:  info] compiling: {}", src.repr);
                let output = compile_cmd(&src.repr, &obj, &info, &pch).output().unwrap();
                std::io::stdout().write_all(&output.stdout).unwrap();
                println!();
                if !output.status.success() { return Err(()) }
            }
        }
    }

    let all_objs = crate::fetch::get_source_files(&PathBuf::from(&info.odir), ".obj").unwrap();
    println!("[mscmp:  info]   linking: {}", info.outfile.repr);
    if info.outfile.repr.ends_with(".lib") {
        let mut cmd = Command::new("lib");
        cmd.args(all_objs.into_iter().map(|fi| fi.repr));
        cmd.args(&info.links);
        cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{}", l)));
        cmd.args([
            format!("/OUT:{}", info.outfile.repr),
            "/MACHINE:X64".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
//            "/LTCG".to_string(),
        ]);
        let output = cmd.output().unwrap();
        std::io::stdout().write_all(&output.stdout).unwrap();
        println!();
        if !output.status.success() { Err(()) } else { Ok(()) }
    } else {
        let mut cmd = Command::new("link");
        cmd.args(all_objs.into_iter().map(|fi| fi.repr));
        cmd.args(&info.links);
        cmd.args(DEFAULT_LIBS);
        cmd.args(info.libdirs.iter().map(|l| format!("/LIBPATH:{}", l)));
        cmd.args([
            format!("/OUT:{}", info.outfile.repr),
            "/DEBUG".to_string(),
            "/MACHINE:X64".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
//            format!("/{}", info.config),
//            "/DYNAMICBASE".to_string(),
//            "/OPT:REF".to_string(),
//            "/LTCG".to_string(),
        ]);
        let output = cmd.output().unwrap();
        std::io::stdout().write_all(&output.stdout).unwrap();
        println!();
        if !output.status.success() { Err(()) } else { Ok(()) }
    }
}

pub fn run_app(outfile: FileInfo, runargs: Vec<String>) {
    println!("[mscmp:  info] running application \"{}\"...", outfile.repr);
    let mut cmd = Command::new(format!("./{}", outfile.repr));
    cmd.args(runargs);
    std::io::stdout().write_all(&cmd.output().unwrap().stdout).unwrap();
}



fn compile_cmd(src: &str, obj: &str, info: &BuildInfo, pch: &Option<String>) -> Command {
    let mut cmd = Command::new("cl");
    cmd.args([
        src.to_string(),
        "/c".to_string(),
        "/EHsc".to_string(),
//        "/Gy".to_string(),
//        "/GL".to_string(),
//        "/Oi".to_string(),
        format!("/std:{}", info.cppstd),
        format!("/Fo:{}", obj),
        info.oplevel.clone(),
    ]);
    if info.config.is_release() {
        cmd.arg("/MD".to_string());
    } else {
        cmd.arg("/MDd".to_string());
    }
    cmd.args(info.incdirs.iter().map(|i| format!("/I{}", i)));
    cmd.args(info.defines.iter().map(|d| format!("/D{}", d)));
    if let Some(outfile) = pch {
        cmd.arg(format!("/Yu{}", outfile));
        let cmpd = format!("{}/{}.pch", info.odir, outfile);
        cmd.arg(format!("/Fp{}", cmpd));
    }
    cmd
}

