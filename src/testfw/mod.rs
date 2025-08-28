use std::{io::Write, path::PathBuf, process::Command};
use crate::{exec::{self, BuildInfo}, input::BuildSwitches, fetch, config::{BuildFile, BuildProfile, Lang}, Error, log_info};


struct TestInfo {
    defines: Vec<String>,
    incdirs: Vec<PathBuf>,
}


fn inherited(build: &BuildFile, profile: &BuildProfile, switches: &BuildSwitches, lang: Lang) -> TestInfo {
    let mut deps = fetch::libraries(build.dependencies.clone(), switches, lang).unwrap();
    let mut defines = profile.defines.clone();
    defines.extend(deps.defines);
    deps.incdirs.extend(profile.include.clone());
    TestInfo {
        defines,
        incdirs: deps.incdirs,
    }
}

pub fn test_lib(mut build: BuildFile, switches: BuildSwitches, args: Vec<String>) -> Result<(), Error> {
    if !std::fs::exists("test").unwrap_or_default() { return Err(Error::MissingTests); }

    let inc = std::env::current_exe()?
        .parent()
        .unwrap()
        .to_owned();

    let profile = build.take(&switches.profile)?;
    let mut partial = inherited(&build, &profile, &switches, build.build.lang);
    partial.defines.push("VANGO_TEST".to_string());
    partial.incdirs.extend([ "test".into(), inc.join("testframework") ]);
    let mut headers = fetch::source_files(&PathBuf::from(&profile.include_pub), "h")?;
    headers.push(inc.join("testframework/vangotest/asserts.h"));
    headers.push(inc.join("testframework/vangotest/casserts.h"));
    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let relink = vec![
        outdir.join(format!("{}{}", switches.toolchain.lib_prefix(), build.build.package)).with_extension(switches.toolchain.lib_ext())
    ];

    let sources = fetch::source_files(&PathBuf::from("test"), build.build.lang.src_ext()).unwrap();
    let outfile = outdir.join(format!("test_{}.exe", build.build.package));
    let info = BuildInfo {
        projkind: crate::config::ProjKind::App,
        toolchain: switches.toolchain,
        profile:   switches.profile.clone(),
        lang:      build.build.lang,
        crtstatic: switches.crtstatic,
        cpprt: build.build.runtime.map(|rt| rt == "C++").unwrap_or_default(),

        defines: partial.defines,

        srcdir: "test".into(),
        incdirs: partial.incdirs,
        libdirs: vec![ PathBuf::from("bin").join(switches.profile.to_string()) ],
        outdir,

        pch: None,
        sources,
        headers,
        archives: vec![ PathBuf::from(&build.build.package).with_extension("lib") ],
        relink,
        outfile: outfile.clone(),

        comp_args: vec![],
        link_args: vec![],
    };
    exec::run_build(info, switches.echo, false)?;
    log_info!(
        "running tests for project {:=<57}",
        format!("\"{}\" ", build.build.package)
    );
    Command::new(PathBuf::from(".").join(outfile))
        .args(args)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
    Ok(())
}

