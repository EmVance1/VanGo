use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use crate::{
    input::BuildSwitches,
    config::{BuildFile, BuildSettings, ProjKind, ToolChain, WarnLevel},
    exec::{self, BuildInfo},
    fetch,
    error::Error,
};


pub fn build(build: &BuildFile, switches: &BuildSwitches, recursive: bool) -> Result<bool, Error> {
    let profile = build.get(&switches.profile)?.to_owned();
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
    deps.defines.push(format!("VANGO_PKG_NAME=\"{}\"", build.name));
    deps.defines.push(format!("VANGO_PKG_VERSION=\"{}\"", build.version));
    deps.defines.push(format!("VANGO_PKG_VERSION_MAJOR={}", build.version.major));
    deps.defines.push(format!("VANGO_PKG_VERSION_MINOR={}", build.version.minor));
    deps.defines.push(format!("VANGO_PKG_VERSION_PATCH={}", build.version.patch));
    deps.incdirs.extend(profile.include);

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

    let info = BuildInfo{
        changed:   settings_cache_changed(deps.defines.clone(), &profile.settings, switches, &outdir),
        projkind:  build.kind,
        toolchain: switches.toolchain,
        lang:      build.lang,
        cpprt:     build.runtime.as_ref().map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),
        settings:  profile.settings,

        defines:  deps.defines,

        srcdir:   PathBuf::from("src"),
        incdirs:  deps.incdirs,
        libdirs:  deps.libdirs,
        outdir,

        pch:      profile.pch,
        sources:  fetch::source_files(&Path::new("src"), build.lang.src_ext()).unwrap(),
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
        Ok(rebuilt) => Ok(deps.rebuilt || rebuilt),
    }
}


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
struct BuildCache {
    defines:       Vec<String>,
    opt_level:     u32,
    opt_size:      bool,
    opt_speed:     bool,
    opt_linktime:  bool,
    iso_compliant: bool,
    warn_level:    WarnLevel,
    warn_as_error: bool,
    debug_info:    bool,
    runtime:       crate::config::Runtime,
    pthreads:      bool,
    aslr:          bool,
    rtti:          bool,
    is_test:       bool,
}

fn settings_cache_changed(defines: Vec<String>, settings: &BuildSettings, switches: &BuildSwitches, outdir: &std::path::Path) -> bool {
    let newcache = BuildCache{
        defines,
        opt_level:     settings.opt_level,
        opt_size:      settings.opt_size,
        opt_speed:     settings.opt_speed,
        opt_linktime:  settings.opt_linktime,
        iso_compliant: settings.iso_compliant,
        warn_level:    settings.warn_level,
        warn_as_error: settings.warn_as_error,
        debug_info:    settings.debug_info,
        runtime:       settings.runtime,
        pthreads:      settings.pthreads,
        aslr:          settings.aslr,
        rtti:          settings.rtti,
        is_test:       switches.is_test,
    };
    let cachepath = outdir.join("build_cache.json");
    if let Ok(cachefile) = std::fs::read_to_string(&cachepath) {
        let _ = std::fs::write(&cachepath, serde_json::to_string(&newcache).unwrap());
        let Ok(oldcache) = serde_json::from_str::<BuildCache>(&cachefile) else {
            return true
        };

        newcache.defines       != oldcache.defines ||
        newcache.opt_level     != oldcache.opt_level ||
        newcache.opt_size      != oldcache.opt_size ||
        newcache.opt_speed     != oldcache.opt_speed ||
        newcache.opt_linktime  != oldcache.opt_linktime ||
        (newcache.iso_compliant && !oldcache.iso_compliant) ||
        ((newcache.warn_level > oldcache.warn_level) && newcache.warn_as_error) ||
        (newcache.warn_as_error && !oldcache.warn_as_error) ||
        newcache.debug_info    != oldcache.debug_info ||
        newcache.runtime       != oldcache.runtime ||
        newcache.pthreads      != oldcache.pthreads ||
        newcache.aslr          != oldcache.aslr ||
        newcache.rtti          != oldcache.rtti ||
        newcache.is_test       != oldcache.is_test
    } else {
        let _ = std::fs::write(&cachepath, serde_json::to_string(&newcache).unwrap());
        true
    }
}

