mod incremental;
mod prep;

use std::{io::Write, path::PathBuf, process::Command};
use crate::{repr::Config, fetch::FileInfo, error::Error, log_info};
use incremental::IncrementalBuild;


#[derive(Debug)]
pub struct BuildInfo {
    pub cppstd: String,
    pub config: Config,
    pub src_dir: String,
    pub out_dir: String,
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

pub fn run_build(info: BuildInfo) -> Result<(), Error> {
    log_info!("starting build for \"{}\":", info.outfile.repr);
    prep::assert_out_dirs(&info.src_dir, &info.out_dir);
    let pch = if let Some(pch) = &info.pch {
        Some(prep::precompile_header(&PathBuf::from(pch), &info))
    } else {
        None
    };

    match IncrementalBuild::calc(&info) {
        IncrementalBuild::NoBuild => {
            log_info!("build up to date for \"{}\"", info.outfile.repr);
            return Ok(())
        }
        IncrementalBuild::BuildSelective(elems) => {
            for (src, obj) in elems {
                log_info!("compiling: {}", src.repr);
                let output = std::process::Command::new("cl")
                    .args(compile_cmd(&src.repr, &obj.repr, &info, &pch))
                    .output()
                    .unwrap();
                std::io::stdout().write_all(&output.stdout).unwrap();
                println!();
                if !output.status.success() { return Err(Error::CompilerFail(src.repr.clone())) }
            }
        }
        IncrementalBuild::BuildAll => {
            for src in &info.sources {
                let obj = src.repr.replace(&info.src_dir, &info.out_dir).replace(".cpp", ".obj");
                log_info!("compiling: {}", src.repr);
                let output = std::process::Command::new("cl")
                    .args(compile_cmd(&src.repr, &obj, &info, &pch))
                    .output()
                    .unwrap();
                std::io::stdout().write_all(&output.stdout).unwrap();
                println!();
                if !output.status.success() { return Err(Error::CompilerFail(src.repr.clone())) }
            }
        }
    }

    let all_objs = crate::fetch::get_source_files(&PathBuf::from(&info.out_dir), ".obj").unwrap();
    log_info!("linking: {}", info.outfile.repr);
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
        if !output.status.success() { Err(Error::LinkerFail(info.outfile.repr)) } else { Ok(()) }
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
//            format!("/{}", info.config.as_arg()),
//            "/DYNAMICBASE".to_string(),
//            "/OPT:REF".to_string(),
//            "/LTCG".to_string(),
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
}

pub fn run_app(outfile: FileInfo,  runargs: Vec<String>) {
    log_info!("running application \"{}\"...", outfile.repr);
    let mut cmd = Command::new(format!("./{}", outfile.repr));
    cmd.args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
}


fn compile_cmd(src: &str, obj: &str, info: &BuildInfo, pch: &Option<String>) -> Vec<String> {
    let mut args = vec![
        src.to_string(),
        "/c".to_string(),
        "/EHsc".to_string(),
//        "/Gy".to_string(),
//        "/GL".to_string(),
//        "/Oi".to_string(),
        format!("/std:{}", info.cppstd),
        format!("/Fo:{}", obj),
        info.oplevel.clone(),
    ];
    if info.config.is_release() {
        args.push("/MD".to_string());
    } else {
        args.push("/MDd".to_string());
    }
    args.extend(info.incdirs.iter().map(|i| format!("/I{}", i)));
    args.extend(info.defines.iter().map(|d| format!("/D{}", d)));
    if let Some(outfile) = pch {
        args.push(format!("/Yu{}", outfile));
        let cmpd = format!("{}/{}.pch", info.out_dir, outfile);
        args.push(format!("/Fp{}", cmpd));
    }
    args
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


#[cfg(test)]
mod tests {

    #[test]
    pub fn test_compile_cmd() {
        /*
        let src = "src/main.cpp";
        let obj = "bin/obj/main.obj";
        let info = BuildInfo{
            cppstd: "c++20".to_string(),
            config: Config::Debug,
            src_dir: "src/".to_string(),
            out_dir: format!("bin/debug/obj/"),
            defines: vec![],
            sources: Vec<FileInfo>,
            headers: vec![],
            incdirs: vec![ "src/".to_string() ],
            libdirs: vec![],
            links: vec![],
            pch: None,
            oplevel: "/Od".to_string(),
            outfile: FileInfo,
        };

        let args = compile_cmd(src, obj, &info, &None);
        */
    }
}

