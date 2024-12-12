use std::{io::Write, path::PathBuf, process::Command};
use crate::{exec::BuildInfo, fetch::FileInfo, BuildFile, Config, log_info};


struct TestInfo{
    defines: Vec<String>,
    incdirs: Vec<String>,
}

fn inherited(build: &BuildFile, config: Config) -> TestInfo {
    let mut deps = crate::fetch::get_libraries(build.dependencies.clone(), config, &build.cpp).unwrap();
    let mut defines = build.defines.clone();
    defines.extend(deps.defines);
    deps.incdirs.extend(build.incdirs.clone());
    TestInfo{
        defines,
        incdirs: deps.incdirs,
    }
}

pub fn test_lib(build: BuildFile, config: Config) {
    let inc = std::env::current_exe().unwrap() // ./target/release/mscmp.exe
                       .parent().unwrap()      // ./target/release/
                       .parent().unwrap()      // ./target/
                       .parent().unwrap()      // ./
                       .to_string_lossy().to_string();

    let mut partial = inherited(&build, config);
    partial.defines.extend([ config.as_arg(), "TEST".to_string() ]);
    partial.incdirs.extend([ "test/".to_string(), format!("{}/testframework/", inc) ]);
    let mut headers = crate::fetch::get_source_files(&PathBuf::from(&build.inc_public), ".h").unwrap();
    headers.push(FileInfo::from_str(&format!("{}/testframework/mscmptest/asserts.h", inc)));
    let relink = vec![ FileInfo::from_str(&format!("bin/{}/{}.lib", config, build.project)) ];

    let sources = crate::fetch::get_source_files(&PathBuf::from("test/"), ".cpp").unwrap();
    let outpath = format!("bin/{}/test_{}.exe", config, build.project);
    let outfile = FileInfo::from_str(&outpath);
    let info = BuildInfo{
        sources,
        headers,
        relink,
        srcdir: "test/".to_string(),
        outdir: format!("bin/{}/obj/", config),
        outfile,
        incdirs: partial.incdirs,
        libdirs: vec![ format!("bin/{}/", config) ],
        links: vec![ format!("{}.lib", build.project) ],
        pch: None,
        cppstd: "c++20".to_string(),
        config,
        mingw: false,
        defines: partial.defines,
    };
    if let Err(_) = crate::exec::run_build(info) {
        // Err(e)
    } else {
        println!();
        log_info!("running tests for project: {}", build.project);
        println!();
        Command::new(format!("./{}", &outpath))
            .current_dir(std::env::current_dir().unwrap())
            .status()
            .unwrap();
        // Ok(outpath)
    }
}

