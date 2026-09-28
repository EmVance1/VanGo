use crate::{
    cli::BuildSwitches,
    config::{BuildSettings, WarnLevel},
};
use serde::{Deserialize, Serialize};


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
struct BuildCache {
    defines: Vec<String>,
    opt_level: u32,
    opt_size: bool,
    opt_speed: bool,
    opt_linktime: bool,
    iso_compliant: bool,
    warn_level: WarnLevel,
    warn_as_error: bool,
    debug_info: bool,
    runtime: crate::config::Runtime,
    aslr: bool,
    no_rtti: bool,
    no_except: bool,
    is_test: bool,

    pthreads: bool,
    asan: bool,
    tsan: bool,
    lsan: bool,
    ubsan: bool,
}

pub fn settings_changed(defines: Vec<String>, settings: &BuildSettings, switches: &BuildSwitches, outdir: &std::path::Path) -> bool {
    let newcache = BuildCache {
        defines,
        opt_level: settings.opt_level,
        opt_size: settings.opt_size,
        opt_speed: settings.opt_speed,
        opt_linktime: settings.opt_linktime,
        iso_compliant: settings.iso_compliant,
        warn_level: settings.warn_level,
        warn_as_error: settings.warn_as_error,
        debug_info: settings.debug_info,
        runtime: settings.runtime,
        aslr: settings.aslr,
        no_rtti: settings.no_rtti,
        no_except: settings.no_except,
        is_test: switches.is_test,

        pthreads: settings.pthreads,
        asan: settings.asan,
        tsan: settings.tsan,
        lsan: settings.lsan,
        ubsan: settings.ubsan,
    };
    let cachepath = outdir.join("build_cache.json");
    if let Ok(cachefile) = std::fs::read_to_string(&cachepath) {
        let _ = std::fs::write(&cachepath, serde_json::to_string(&newcache).unwrap());
        let Ok(oldcache) = serde_json::from_str::<BuildCache>(&cachefile) else {
            return true;
        };

        // settings that may or may not trigger project rebuilds
        newcache.defines != oldcache.defines
            || newcache.opt_level != oldcache.opt_level
            || newcache.opt_size != oldcache.opt_size
            || newcache.opt_speed != oldcache.opt_speed
            || newcache.opt_linktime != oldcache.opt_linktime
            || (newcache.iso_compliant && !oldcache.iso_compliant)
            || ((newcache.warn_level > oldcache.warn_level) && newcache.warn_as_error)
            || (newcache.warn_as_error && !oldcache.warn_as_error)
            || newcache.debug_info != oldcache.debug_info
            || newcache.runtime != oldcache.runtime
            || newcache.aslr != oldcache.aslr
            || newcache.no_rtti != oldcache.no_rtti
            || newcache.no_except != oldcache.no_except
            || newcache.is_test != oldcache.is_test
            || newcache.pthreads != oldcache.pthreads
            || newcache.asan != oldcache.asan
            || newcache.tsan != oldcache.tsan
            || newcache.lsan != oldcache.lsan
            || newcache.ubsan != oldcache.ubsan
    } else {
        let _ = std::fs::write(&cachepath, serde_json::to_string(&newcache).unwrap());
        true
    }
}
