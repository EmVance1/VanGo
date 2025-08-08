use std::{io::Write, path::PathBuf, process::Command};
use crate::{
    BuildFile,
    Config,
    exec::BuildInfo,
    fetch::FileInfo,
    repr::ToolSet,
    Error,
    log_info,
};


struct TestInfo{
    defines: Vec<String>,
    incdirs: Vec<String>,
}

fn inherited(build: &BuildFile, config: Config, mingw: bool) -> TestInfo {
    let mut deps = crate::fetch::get_libraries(build.dependencies.clone(), config, mingw, &build.cpp).unwrap();
    let mut defines = build.defines.clone();
    defines.extend(deps.defines);
    deps.incdirs.extend(build.incdirs.clone());
    TestInfo{
        defines,
        incdirs: deps.incdirs,
    }
}

pub fn test_lib(build: BuildFile, config: Config, mingw: bool, args: Vec<String>) -> Result<(), Error> {
    if !std::fs::exists("test/").unwrap() {
        return Err(Error::MissingTests)
    }

    let toolset = if cfg!(target_os = "windows") && !mingw {
        ToolSet::MSVC
    } else if cfg!(target_os = "linux") || mingw {
        ToolSet::GNU{ mingw }
    } else {
        ToolSet::CLANG
    };

    let inc = std::env::current_exe().unwrap() // ./target/release/mscmp.exe
                       .parent().unwrap()      // ./target/release/
                       .parent().unwrap()      // ./target/
                       .parent().unwrap()      // ./
                       .to_string_lossy().to_string();

    let mut partial = inherited(&build, config, mingw);
    partial.defines.extend([ config.as_arg(), "TEST".to_string() ]);
    partial.incdirs.extend([ "test/".to_string(), format!("{}/testframework/", inc) ]);
    let mut headers = if let Some(inc) = build.inc_public {
        crate::fetch::get_source_files(&PathBuf::from(&inc), ".h").unwrap()
    } else {
        crate::fetch::get_source_files(&PathBuf::from(&build.srcdir), ".h").unwrap()
    };
    headers.push(FileInfo::from_str(&format!("{}/testframework/mscmptest/asserts.h", inc)));
    headers.push(FileInfo::from_str(&format!("{}/testframework/mscmptest/casserts.h", inc)));
    let relink = vec![ FileInfo::from_str(&format!("bin/{}/{}.lib", config, build.project)) ];

    let cppstd = build.cpp.to_ascii_lowercase();
    let is_c = !build.cpp.starts_with("c++");

    let sources = crate::fetch::get_source_files(&PathBuf::from("test/"), if is_c { ".c" } else { ".cpp" }).unwrap();
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
        cppstd,
        is_c,
        config,
        toolset,
        projkind: crate::repr::ProjKind::App,
        defines: partial.defines,
        comp_args: vec![],
        link_args: vec![],
    };
    crate::exec::run_build(info)?;
    log_info!("running tests for project {:=<57}", format!("\"{}\" ", build.project));
    Command::new(format!("./{}", &outpath))
        .args(args)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
    Ok(())
}

