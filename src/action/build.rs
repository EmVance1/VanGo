use std::path::PathBuf;
use crate::{
    input::BuildSwitches,
    exec::{self, BuildInfo},
    config::{BuildFile, ProjKind, ToolChain},
    fetch,
    error::Error,
};


pub fn build(mut build: BuildFile, switches: &BuildSwitches) -> Result<(bool, PathBuf), Error> {
    let profile = build.take(&switches.profile)?;
    let srcdir = PathBuf::from("src");
    let mut headers = fetch::source_files(&profile.include_pub, "h").unwrap();
    if build.lang.is_cpp() {
        headers.extend(fetch::source_files(&profile.include_pub, "hpp").unwrap());
    }
    for incdir in &profile.include {
        headers.extend(fetch::source_files(incdir, "h").unwrap());
        if build.lang.is_cpp() {
            headers.extend(fetch::source_files(incdir, "hpp").unwrap());
        }
    }
    let sources = fetch::source_files(&srcdir, build.lang.src_ext()).unwrap();

    let mut deps = fetch::libraries(build.dependencies, switches, build.lang)?;
    deps.defines.extend(profile.defines);
    if switches.is_test { deps.defines.push("VANGO_TEST".to_string()); }
    if cfg!(windows) {
        deps.defines.push("UNICODE".to_string());
        deps.defines.push("_UNICODE".to_string());
        if let ProjKind::SharedLib{..} = build.kind {
            deps.defines.push("VANGO_EXPORT_SHARED".to_string());
        }
    }
    deps.incdirs.extend(profile.include);

    let rebuilt_dep = deps.rebuilt;
    let outdir = PathBuf::from("bin").join(switches.profile.to_string());
    let (outfile, implib) = match build.kind {
        ProjKind::App => {
            (outdir.join(build.name).with_extension(switches.toolchain.app_ext()), None)
        }
        ProjKind::SharedLib{implib: false} => {
            (outdir.join(format!("{}{}", ToolChain::shared_lib_prefix(), build.name))
             .with_extension(ToolChain::shared_lib_ext()), None)
        }
        ProjKind::SharedLib{implib: true} => {
            (outdir.join(format!("{}{}", ToolChain::shared_lib_prefix(), build.name))
             .with_extension(ToolChain::shared_lib_ext()),
             Some(outdir.join(format!("{}{}", switches.toolchain.static_lib_prefix(), build.name))
             .with_extension(switches.toolchain.static_lib_ext())))
        }
        ProjKind::StaticLib => {
            (outdir.join(format!("{}{}", switches.toolchain.static_lib_prefix(), build.name))
             .with_extension(switches.toolchain.static_lib_ext()), None)
        }
    };

    let info = BuildInfo{
        projkind:  build.kind,
        toolchain: switches.toolchain,
        lang:      build.lang,
        crtstatic: switches.crtstatic,
        cpprt:     build.runtime.map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),
        settings:  profile.settings,

        defines:  deps.defines,

        srcdir,
        incdirs:  deps.incdirs,
        libdirs:  deps.libdirs,
        outdir,

        pch:      profile.pch,
        sources,
        headers,
        archives: deps.archives,
        relink:   deps.relink,
        outfile:  outfile.clone(),
        implib,

        comp_args: profile.compiler_options,
        link_args: profile.linker_options,
    };
    match exec::run_build(info, switches.echo, switches.verbose) {
        Err(e) => Err(e),
        Ok(rebuilt) => Ok((rebuilt_dep || rebuilt, outfile)),
    }
}

