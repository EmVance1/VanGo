use std::{io::Write, path::PathBuf, process::Command};
use crate::{config::BuildFile, exec::{self, BuildInfo}, fetch, input::BuildSwitches, Error, log_info_ln};


pub fn test_lib(mut build: BuildFile, switches: &BuildSwitches, args: Vec<String>) -> Result<(), Error> {
    if !std::fs::exists("test").unwrap_or_default() { return Err(Error::MissingTests); }

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

    let mut inherited = fetch::libraries(build.dependencies.clone(), switches, build.lang).unwrap();
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
        cpprt:     build.runtime.map(|rt| rt == "C++").unwrap_or_default(),
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

