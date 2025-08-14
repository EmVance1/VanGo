use crate::{exec::BuildInfo, fetch::FileInfo, input::BuildSwitches, BuildFile, Error, Lang, log_info};
use std::{io::Write, path::PathBuf, process::Command};


struct TestInfo {
    defines: Vec<String>,
    incdirs: Vec<String>,
}


fn inherited(build: &BuildFile, switches: BuildSwitches, lang: Lang) -> TestInfo {
    let mut deps = crate::fetch::libraries(build.dependencies.clone(), switches, lang).unwrap();
    let mut defines = build.defines.clone();
    defines.extend(deps.defines);
    deps.incdirs.extend(build.incdirs.clone());
    TestInfo {
        defines,
        incdirs: deps.incdirs,
    }
}

pub fn test_lib(build: BuildFile, switches: BuildSwitches, args: Vec<String>) -> Result<(), Error> {
    if !std::fs::exists("test/").unwrap() { return Err(Error::MissingTests); }

    let inc = std::env::current_exe()
        .unwrap() // ./target/release/vango.exe
        .parent()
        .unwrap() // ./target/release/
        .parent()
        .unwrap() // ./target/
        .parent()
        .unwrap() // ./
        .to_string_lossy()
        .to_string();

    let lang: Lang = build.lang.parse()?;
    let mut partial = inherited(&build, switches, lang);
    partial.defines.extend([ switches.config.as_define().to_string(), "TEST".to_string() ]);
    partial.incdirs.extend([ "test/".to_string(), format!("{inc}/testframework/") ]);
    let mut headers = if let Some(inc) = build.include_public {
        crate::fetch::source_files(&PathBuf::from(&inc), ".h").unwrap()
    } else {
        crate::fetch::source_files(&PathBuf::from(&build.srcdir), ".h").unwrap()
    };
    headers.push(FileInfo::from_str(&format!("{inc}/testframework/vangotest/asserts.h")));
    headers.push(FileInfo::from_str(&format!("{inc}/testframework/vangotest/casserts.h")));
    let relink = vec![
        FileInfo::from_str(&format!( "bin/{}/{}{}{}", switches.config, switches.toolchain.lib_prefix(), build.project, switches.toolchain.lib_ext())) ];

    let sources = crate::fetch::source_files(&PathBuf::from("test/"), lang.src_ext()).unwrap();
    let outpath = format!("bin/{}/test_{}.exe", switches.config, build.project);
    let outfile = FileInfo::from_str(&outpath);
    let info = BuildInfo {
        projkind: crate::repr::ProjKind::App,
        toolchain: switches.toolchain,
        config: switches.config,
        lang,
        crtstatic: switches.crtstatic,

        sources,
        headers,
        relink,
        srcdir: "test/".to_string(),
        outdir: format!("bin/{}/obj/", switches.config),
        outfile,
        defines: partial.defines,
        incdirs: partial.incdirs,
        libdirs: vec![format!("bin/{}/", switches.config)],
        links: vec![format!("{}.lib", build.project)],
        pch: None,

        comp_args: vec![],
        link_args: vec![],
    };
    crate::exec::run_build(info, switches.echo, false)?;
    log_info!(
        "running tests for project {:=<57}",
        format!("\"{}\" ", build.project)
    );
    Command::new(format!("./{}", &outpath))
        .args(args)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
    Ok(())
}

