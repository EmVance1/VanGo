use std::{path::PathBuf, process::ExitCode};
use crate::{
    config::BuildFile,
    input::BuildSwitches,
    fetch,
    exec::{self, BuildInfo},
    Error,
    log_info_ln
};


pub fn test(mut build: BuildFile, switches: &BuildSwitches, args: Vec<String>) -> Result<ExitCode, Error> {
    if !std::fs::exists("test").unwrap_or_default() { return Err(Error::MissingTests(build.name)); }

    let include = std::env::current_exe()?
        .parent()
        .unwrap()
        .to_owned()
        .join("testframework");

    let profile = build.take(&switches.profile)?;
    let mut headers = fetch::source_files(&PathBuf::from(&profile.include_pub), "h")?;
    headers.extend(fetch::source_files(&PathBuf::from(&profile.include_pub), "hpp")?);
    headers.push(include.join("vangotest/asserts.h"));
    headers.push(include.join("vangotest/casserts.h"));
    headers.push(include.join("vangotest/asserts2.h"));
    headers.push(include.join("vangotest/casserts2.h"));

    let mut inherited = fetch::libraries(&build, &profile.baseprof, switches)?;
    inherited.defines.push("VANGO_TEST".to_string());
    inherited.incdirs.extend([ "test".into(), include, profile.include_pub ]);
    inherited.libdirs.push(PathBuf::from("bin").join(switches.profile.to_string()));

    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let mut relink = Vec::new();
    if switches.toolchain.is_msvc() {
        inherited.archives.insert(0, PathBuf::from(&build.name).with_extension("lib"));
        relink.push(outdir.join(&build.name).with_extension("lib"));
    } else {
        inherited.archives.insert(0, PathBuf::from(&build.name));
        relink.push(outdir.join(format!("lib{}", build.name)).with_extension("a"));
    }

    let sources = fetch::source_files(&PathBuf::from("test"), build.lang.src_ext()).unwrap();
    let outfile = outdir.join(format!("test_{}.exe", build.name));
    let info = BuildInfo {
        projkind:  crate::config::ProjKind::App,
        toolchain: switches.toolchain,
        lang:      build.lang,
        crtstatic: switches.crtstatic,
        cpprt:     build.runtime.map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),
        settings:  profile.settings,

        defines:   inherited.defines,

        srcdir:    "test".into(),
        incdirs:   inherited.incdirs,
        libdirs:   inherited.libdirs,
        outdir,

        pch: None,
        sources,
        headers,
        archives: inherited.archives,
        relink    ,
        outfile: outfile.clone(),
        implib: None,

        comp_args: vec![],
        link_args: vec![],
    };
    exec::run_build(info, switches.echo, false, false)?;
    log_info_ln!("{:=<80}", format!("running tests for project: {} ", build.name));
    let code: u8 = std::process::Command::new(PathBuf::from(".").join(&outfile))
        .args(args)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap()
        .code()
        .ok_or(Error::ExeKilled(outfile))?
        .try_into()
        .unwrap_or(1);
    Ok(code.into())
}

