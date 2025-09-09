use std::{io::Write, path::PathBuf, process::Command};
use crate::{exec::{self, BuildInfo}, input::BuildSwitches, fetch, config::{BuildFile, BuildProfile, Lang}, Error, log_info_ln};


struct TestInfo {
    defines: Vec<String>,
    incdirs: Vec<PathBuf>,
    archives: Vec<PathBuf>,
}


fn inherited(build: &BuildFile, profile: &BuildProfile, switches: &BuildSwitches, lang: Lang) -> TestInfo {
    let mut deps = fetch::libraries(build.dependencies.clone(), switches, lang).unwrap();
    let mut defines = profile.defines.clone();
    defines.extend(deps.defines);
    deps.incdirs.extend(profile.include.clone());
    TestInfo {
        defines,
        incdirs: deps.incdirs,
        archives: deps.archives,
    }
}

pub fn test_lib(mut build: BuildFile, switches: BuildSwitches, args: Vec<String>) -> Result<(), Error> {
    if !std::fs::exists("test").unwrap_or_default() { return Err(Error::MissingTests); }

    let inc = std::env::current_exe()?
        .parent()
        .unwrap()
        .to_owned();

    let profile = build.take(&switches.profile)?;
    let mut partial = inherited(&build, &profile, &switches, build.lang);
    partial.defines.push("VANGO_TEST".to_string());
    partial.incdirs.extend([ "test".into(), inc.join("testframework") ]);
    let mut headers = fetch::source_files(&PathBuf::from(&profile.include_pub), "h")?;
    headers.extend(fetch::source_files(&PathBuf::from(&profile.include_pub), "hpp")?);
    headers.push(inc.join("testframework/vangotest/asserts.h"));
    headers.push(inc.join("testframework/vangotest/casserts.h"));
    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let relink = [
        outdir.join(format!("{}{}", switches.toolchain.static_lib_prefix(), build.name)).with_extension(switches.toolchain.static_lib_ext())
    ].into_iter().collect();
    if switches.toolchain.is_msvc() {
        partial.archives.insert(0, PathBuf::from(&build.name).with_extension("lib"))
    } else {
        partial.archives.insert(0, PathBuf::from(&build.name))
    };

    let sources = fetch::source_files(&PathBuf::from("test"), build.lang.src_ext()).unwrap();
    let outfile = outdir.join(format!("test_{}.exe", build.name));
    let info = BuildInfo {
        projkind:  crate::config::ProjKind::App,
        toolchain: switches.toolchain,
        lang:      build.lang,
        crtstatic: switches.crtstatic,
        cpprt:     build.runtime.map(|rt| rt == "C++").unwrap_or_default(),
        settings:  profile.settings,

        defines: partial.defines,

        srcdir: "test".into(),
        incdirs: partial.incdirs,
        libdirs: [ PathBuf::from("bin").join(switches.profile.to_string()) ].into_iter().collect(),
        outdir,

        pch: None,
        sources,
        headers,
        archives: partial.archives,
        relink,
        outfile: outfile.clone(),
        implib: None,

        comp_args: vec![],
        link_args: vec![],
    };
    exec::run_build(info, switches.echo, false)?;
    log_info_ln!(
        "running tests for project {:=<57}",
        format!("\"{}\" ", build.name)
    );
    Command::new(PathBuf::from(".").join(outfile))
        .args(args)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
    Ok(())
}

