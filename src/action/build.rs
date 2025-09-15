use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::{
    config::{BuildFile, BuildSettings, ProjKind, ToolChain, Version}, error::Error, exec::{self, BuildInfo}, fetch, input::BuildSwitches
};


pub fn build(build: &BuildFile, switches: &BuildSwitches, recursive: bool) -> Result<bool, Error> {
    let profile = build.get(&switches.profile)?.to_owned();
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

    let mut deps = fetch::libraries(build, &profile.baseprof, switches)?;
    deps.defines.extend(profile.defines);
    if switches.is_test { deps.defines.push("VANGO_TEST".to_string()); }
    if cfg!(windows) {
        deps.defines.push("UNICODE".to_string());
        deps.defines.push("_UNICODE".to_string());
        if let ProjKind::SharedLib{..} = build.kind {
            deps.defines.push("VANGO_EXPORT_SHARED".to_string());
        }
    }
    deps.defines.push(format!("VANGO_PKG_VERSION=\"{}\"", build.version));
    deps.defines.push(format!("VANGO_PKG_VERSION_MAJOR={}", build.version.major));
    deps.defines.push(format!("VANGO_PKG_VERSION_MINOR={}", build.version.minor));
    deps.defines.push(format!("VANGO_PKG_VERSION_PATCH={}", build.version.patch));
    deps.incdirs.extend(profile.include);

    let rebuilt_dep = deps.rebuilt;
    let outdir = if switches.toolchain == ToolChain::system_default() {
        PathBuf::from("bin").join(switches.profile.to_string())
    } else {
        PathBuf::from("bin").join(switches.toolchain.as_directory()).join(switches.profile.to_string())
    };
    let (outfile, implib) = match build.kind {
        ProjKind::App => {
            (outdir.join(&build.name).with_extension(switches.toolchain.app_ext()), None)
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

    let changed = cache_changed_and_update(&profile.settings, switches, build.version, &outdir);

    let info = BuildInfo{
        projkind:  build.kind,
        toolchain: switches.toolchain,
        lang:      build.lang,
        crtstatic: switches.crtstatic,
        cpprt:     build.runtime.as_ref().map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),
        settings:  profile.settings,
        changed,

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
    match exec::run_build(info, switches.echo, switches.verbose, recursive) {
        Err(e) => Err(e),
        Ok(rebuilt) => Ok(rebuilt_dep || rebuilt),
    }
}


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct BuildCache {
    opt_level:     u32,
    opt_size:      bool,
    opt_speed:     bool,
    opt_linktime:  bool,
    iso_compliant: bool,
    warn_as_error: bool,
    debug_info:    bool,
    runtime:       crate::config::Runtime,
    pthreads:      bool,
    aslr:          bool,
    rtti:          bool,
    crtstatic:     bool,
    install:       bool,
    echo:          bool,
    verbose:       bool,
    is_test:       bool,
    version:       Version
}

fn cache_changed_and_update(settings: &BuildSettings, switches: &BuildSwitches, version: Version, outdir: &PathBuf) -> bool {
    let newcache = BuildCache{
        opt_level:     settings.opt_level,
        opt_size:      settings.opt_size,
        opt_speed:     settings.opt_speed,
        opt_linktime:  settings.opt_linktime,
        iso_compliant: settings.iso_compliant,
        warn_as_error: settings.warn_as_error,
        debug_info:    settings.debug_info,
        runtime:       settings.runtime,
        pthreads:      settings.pthreads,
        aslr:          settings.aslr,
        rtti:          settings.rtti,
        crtstatic:     switches.crtstatic,
        install:       switches.install,
        echo:          switches.echo,
        verbose:       switches.verbose,
        is_test:       switches.is_test,
        version,
    };
    let cachepath = outdir.join("build_cache.json");
    if !cachepath.exists() {
        let _ = std::fs::write(&cachepath, serde_json::to_string(&newcache).unwrap());
        false
    } else {
        let oldcache: BuildCache = serde_json::from_str(&std::fs::read_to_string(&cachepath).unwrap()).unwrap();
        let _ = std::fs::write(&cachepath, serde_json::to_string(&newcache).unwrap());
        oldcache != newcache
    }
}

