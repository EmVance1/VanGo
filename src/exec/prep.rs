use std::{io::Write, path::{Path, PathBuf}, process::Command};
use crate::fetch::FileInfo;
use super::BuildInfo;


pub fn assert_out_dirs(sdir: &str, odir: &str) {
    if !std::fs::exists("./bin/").unwrap() {
        std::fs::create_dir("./bin/").unwrap();
        std::fs::create_dir("./bin/debug/").unwrap();
        std::fs::create_dir("./bin/release/").unwrap();
    } else {
        if !std::fs::exists("./bin/debug").unwrap() {
            std::fs::create_dir("./bin/debug/").unwrap();
        }
        if !std::fs::exists("./bin/release").unwrap() {
            std::fs::create_dir("./bin/release/").unwrap();
        }
    }
    assert_out_dirs_rec(&PathBuf::from(sdir), sdir, odir);
}

pub fn assert_out_dirs_rec(root: &Path, sdir: &str, odir: &str) {
    let obj = root.to_string_lossy().replace(sdir, odir);
    if !std::fs::exists(&obj).unwrap() {
        std::fs::create_dir(obj).unwrap();
    }
    for e in std::fs::read_dir(root).ok().unwrap() {
        let e = e.ok().unwrap();
        if e.path().is_dir() {
            assert_out_dirs_rec(&e.path(), sdir, odir);
        }
    }
}

pub fn precompile_header(header: &str, info: &BuildInfo) {
    let head_with_dir = format!("{}{}", info.src_dir, header);
    let cppf = format!("{}{}", info.src_dir, header.replace(".h", ".cpp"));
    let objt = format!("{}{}", info.out_dir, header.replace(".h", ".obj"));
    let cmpd = format!("{}{}.pch", info.out_dir, header.replace(&info.src_dir, &info.out_dir));
    let infile = FileInfo::from_path(&PathBuf::from(&head_with_dir));
    let outfile = FileInfo::from_path(&PathBuf::from(&cmpd));

    if !outfile.exists() || infile.modified().unwrap() > outfile.modified().unwrap() {
        let mut cmd = Command::new("cl");
        cmd.args([
            cppf.clone(),
            "/c".to_string(),
            "/EHsc".to_string(),
            format!("/Yc{}", header),
            format!("/Fp{}", cmpd),
            format!("/std:{}", info.cppstd),
            format!("/Fo:{}", objt),
//            "/Gy".to_string(),
//            "/GL".to_string(),
//            "/Oi".to_string(),
        ]);
        cmd.args(info.incdirs.iter().map(|i| format!("/I{}", i)));
        cmd.args(info.defines.iter().map(|d| format!("/D{}", d)));
        if info.config.is_release() {
            cmd.args(["/MD", "/O2"]);
        } else {
            cmd.args(["/MDd", "/Od"]);
        }
        println!("[mscmp:  info] compiling precompiled header: {}", header);
        std::io::stdout().write_all(&cmd.output().unwrap().stdout).unwrap();
        println!();
    }
}

