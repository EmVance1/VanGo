use std::{io::Write, path::PathBuf, process::Command};
use crate::{exec::{self, BuildInfo}, input::BuildSwitches, fetch, BuildFile, Error, Lang, log_info};


struct TestInfo {
    defines: Vec<String>,
    incdirs: Vec<PathBuf>,
}


fn inherited(build: &BuildFile, switches: BuildSwitches, lang: Lang) -> TestInfo {
    let mut deps = fetch::libraries(build.dependencies.clone(), switches, lang).unwrap();
    let mut defines = build.defines.clone();
    defines.extend(deps.defines);
    deps.incdirs.extend(build.incdirs.clone());
    TestInfo {
        defines,
        incdirs: deps.incdirs,
    }
}

pub fn test_lib(build: BuildFile, switches: BuildSwitches, args: Vec<String>) -> Result<(), Error> {
    if !std::fs::exists("test").unwrap_or_default() { return Err(Error::MissingTests); }

    let inc = std::env::current_exe()?
        .parent()
        .unwrap()
        .to_owned();

    let lang: Lang = build.lang.parse()?;
    let mut partial = inherited(&build, switches, lang);
    partial.defines.extend([ switches.config.as_define().to_string(), "VANGO_TEST".to_string() ]);
    partial.incdirs.extend([ "test".into(), inc.join("testframework") ]);
    let mut headers = if let Some(inc) = build.include_public {
        fetch::source_files(&PathBuf::from(&inc), ".h")?
    } else {
        fetch::source_files(&PathBuf::from(&build.srcdir), ".h")?
    };
    headers.push(inc.join("testframework/vangotest/asserts.h"));
    headers.push(inc.join("testframework/vangotest/casserts.h"));
    let outdir = PathBuf::from("bin").join(switches.config.to_string());
    let relink = vec![
        outdir.join(format!("{}{}", switches.toolchain.lib_prefix(), build.project)).with_extension(switches.toolchain.lib_ext())
    ];

    let sources = fetch::source_files(&PathBuf::from("test"), lang.src_ext()).unwrap();
    let outfile = outdir.join(format!("test_{}.exe", build.project));
    let info = BuildInfo {
        projkind: crate::repr::ProjKind::App,
        toolchain: switches.toolchain,
        config: switches.config,
        lang,
        crtstatic: switches.crtstatic,

        defines: partial.defines,

        srcdir: "test".into(),
        incdirs: partial.incdirs,
        libdirs:  vec![ PathBuf::from("bin").join(switches.config.to_string()) ],
        outdir,

        pch: None,
        sources,
        headers,
        archives: vec![ PathBuf::from(&build.project).with_extension("lib") ],
        relink,
        outfile: outfile.clone(),

        comp_args: vec![],
        link_args: vec![],
    };
    exec::run_build(info, switches.echo, false)?;
    log_info!(
        "running tests for project {:=<57}",
        format!("\"{}\" ", build.project)
    );
    Command::new(PathBuf::from(".").join(outfile))
        .args(args)
        .current_dir(std::env::current_dir().unwrap())
        .status()
        .unwrap();
    Ok(())
}

