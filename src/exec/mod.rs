mod incremental;
mod prep;

use std::{io::Write, path::PathBuf, process::Command};
use crate::{repr::Config, fetch::FileInfo, error::Error, log_info};


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
    pub outfile: FileInfo,
}

impl BuildInfo {
    fn compile_info(&self) -> CompileInfo<'_> {
        CompileInfo{
            cppstd: &self.cppstd,
            config: self.config,
            out_dir: &self.out_dir,
            defines: &self.defines,
            incdirs: &self.incdirs,
            pch: &self.pch,
        }
    }
}

#[derive(Debug)]
struct CompileInfo<'a> {
    cppstd: &'a str,
    config: Config,
    out_dir: &'a str,
    defines: &'a [String],
    incdirs: &'a [String],
    pch: &'a Option<String>,
}


pub fn run_build(info: BuildInfo) -> Result<(), Error> {
    log_info!("starting build for \"{}\":", info.outfile.repr);
    prep::assert_out_dirs(&info.src_dir, &info.out_dir);
    if let Some(pch) = &info.pch {
        prep::precompile_header(pch, &info)
    }

    match incremental::get_outdated(&info) {
        None => {
            log_info!("build up to date for \"{}\"", info.outfile.repr);
            return Ok(())
        }
        Some(elems) => {
            for (src, obj) in elems {
                log_info!("compiling: {}", src);
                let output = std::process::Command::new("cl")
                    .args(compile_cmd(src, &obj, info.compile_info()))
                    .output()
                    .unwrap();
                std::io::stdout().write_all(&output.stdout).unwrap();
                println!();
                if !output.status.success() { return Err(Error::CompilerFail(src.to_string())) }
            }
        }
    }

    let all_objs = crate::fetch::get_source_files(&PathBuf::from(&info.out_dir), ".obj").unwrap();
    log_info!("linking: {}", info.outfile.repr);
    if info.outfile.repr.ends_with(".lib") {
        let mut cmd = Command::new("lib");
        cmd.args(all_objs.into_iter().map(|o| o.repr));
        cmd.args(&info.links);
        cmd.args(DEFAULT_LIBS);
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
            "/MACHINE:X64".to_string(),
            "/SUBSYSTEM:CONSOLE".to_string(),
//            "/LTCG".to_string(),
//            "/DEBUG".to_string(),
//            format!("/{}", info.config.as_arg()),
//            "/OPT:REF".to_string(),
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

pub fn run_app(outfile: PathBuf,  runargs: Vec<String>) {
    log_info!("running application \"{}\"...", outfile.to_str().unwrap());
    Command::new(format!("./{}", outfile.to_str().unwrap()))
        .args(runargs)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
}


fn compile_cmd(src: &str, obj: &str, info: CompileInfo) -> Vec<String> {
    let mut args = vec![
        src.to_string(),
        "/c".to_string(),
        "/EHsc".to_string(),
        format!("/std:{}", info.cppstd),
        format!("/Fo:{}", obj),
//        "/Gy".to_string(),
//        "/GL".to_string(),
//        "/Oi".to_string(),
    ];
    args.extend(info.incdirs.iter().map(|i| format!("/I{}", i)));
    args.extend(info.defines.iter().map(|d| format!("/D{}", d)));
    if info.config.is_release() {
        args.push("/MD".to_string());
        args.push("/O2".to_string());
    } else {
        args.push("/MDd".to_string());
        args.push("/Od".to_string());
    }
    if let Some(outfile) = info.pch {
        let cmpd = format!("{}/{}.pch", info.out_dir, outfile);
        args.push(format!("/Yu{}", outfile));
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
    use super::*;

    #[test]
    pub fn test_compile_cmd() {
        let src = "src/main.cpp";
        let obj = "bin/debug/obj/main.obj";
        let defines = vec![];
        let incdirs = vec![ "src/".to_string() ];
        let pch = None;
        let info = CompileInfo{
            cppstd: "c++20",
            config: Config::Debug,
            out_dir: "bin/debug/obj/",
            defines: &defines,
            incdirs: &incdirs,
            pch: &pch,
        };

        let args = compile_cmd(src, obj, info);
        assert_eq!(args, vec![
            "src/main.cpp".to_string(),
            "/c".to_string(),
            "/EHsc".to_string(),
            "/std:c++20".to_string(),
            "/Fo:bin/debug/obj/main.obj".to_string(),
            "/Isrc/".to_string(),
            "/MDd".to_string(),
            "/Od".to_string(),
        ]);
    }
}

