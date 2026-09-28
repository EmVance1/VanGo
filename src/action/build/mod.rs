mod cache;

use crate::{
    cli::BuildSwitches,
    config::{PackageManifest, Artefact, Language, Platform, BuildSettings, PrecompiledHeader},
    deps,
    error::Error,
    exec::{Toolchain, fsutil},
};
use std::path::{Path, PathBuf};


#[derive(Debug)]
pub struct BuildInfo {
    pub artefact: Artefact,
    pub toolchain: Toolchain,
    pub lang: Language,
    pub cpprt: bool,
    pub settings: BuildSettings,
    pub changed: bool,
    pub is_testexe: bool,

    pub defines: Vec<String>,

    pub srcdir: PathBuf,
    pub incdirs: Vec<PathBuf>,
    pub libdirs: Vec<PathBuf>,
    pub rpaths: Vec<PathBuf>,
    pub outdir: PathBuf,

    // (re-)compilation sensitive files, exhaustive
    pub pch: Vec<PrecompiledHeader>,
    pub sources: Vec<PathBuf>,
    pub headers: Vec<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub relink: Vec<PathBuf>,
    pub outfile: PathBuf,
    pub implib: Option<PathBuf>,

    // raw forwarded args
    pub comp_args: Vec<String>,
    pub link_args: Vec<String>,
}


impl BuildInfo {
    pub fn from_build_cfg(build: &PackageManifest, switches: &BuildSwitches) -> Result<BuildInfo, Error> {
        if !std::fs::exists("src").unwrap_or_default() {
            return Err(Error::MissingSource(build.name.clone()));
        }

        // select toolchain in order of descending priority
        let toolchain = switches.toolchain.or(build.toolchain).unwrap_or(Toolchain::user_default()?);

        // extract settings for current profile
        let mut profile = build.get(&switches.profile)?.to_owned();

        // BANDAID: collect all headers from all (direct) include directories (for incremental builds)
        let mut headers = Vec::new();
        if build.artefact.is_lib() {
            if !std::fs::exists("include").unwrap_or_default() {
                return Err(Error::MissingInclude(build.name.clone()));
            }
            profile.include.push("include".into());
        }
        for incdir in &profile.include {
            headers.extend(fsutil::scan_for_filetype(incdir, &["h".to_string()])?);
            if build.lang.is_cpp() {
                headers.extend(fsutil::scan_for_filetype(incdir, &["hpp".to_string()])?);
            }
        }

        // collect and flatten all dependency information into single SOA
        let mut deps = deps::libraries(build, &profile.baseprof, switches)?;
        deps.defines.extend(profile.defines);
        if switches.is_test {
            deps.defines.push("VANGO_TEST".to_string());
        }
        if cfg!(windows) {
            deps.defines.push("UNICODE".to_string());
            deps.defines.push("_UNICODE".to_string());
            if let Artefact::SharedLib = build.artefact {
                deps.defines.push("VANGO_EXPORT_SHARED".to_string());
            }
        }
        deps.defines.push(format!("VANGO_PKG_NAME=\"{}\"", build.name));
        deps.defines.push(format!("VANGO_PKG_VERSION=\"{}\"", build.version));
        deps.defines.push(format!("VANGO_PKG_VERSION_MAJOR={}", build.version.major));
        deps.defines.push(format!("VANGO_PKG_VERSION_MINOR={}", build.version.minor));
        deps.defines.push(format!("VANGO_PKG_VERSION_PATCH={}", build.version.patch));
        deps.incdirs.extend(profile.include);

        // scope all output to correct directory
        let outdir = if toolchain == Toolchain::system_default()? {
            PathBuf::from("bin").join(switches.profile.to_string())
        } else {
            PathBuf::from("bin")
                .join(toolchain.as_directory())
                .join(switches.profile.to_string())
        };

        // determine output filenames, depends on project type, toolchain and platform (see elems::{Toolchain, Artefact})
        let outfile = match build.artefact {
            Artefact::Executable => outdir.join(toolchain.fmt_executable(&build.name)),
            Artefact::StaticLib => outdir.join(toolchain.fmt_static_lib(&build.name)),
            Artefact::SharedLib => outdir.join(Platform::current()?.fmt_shared_lib(&build.name)),
            Artefact::Module    => outdir.join(Platform::current()?.fmt_module(&build.name)),
        };
        let implib = if build.artefact == Artefact::SharedLib && Platform::current()? == Platform::Windows {
            Some(outdir.join(toolchain.fmt_static_lib(&build.name)))
        } else {
            None
        };

        // replicate source directory hierarchy in output directory
        fsutil::ensure_out_dirs(Path::new("src"), &outdir);

        Ok(BuildInfo {
            changed: cache::settings_changed(deps.defines.clone(), &profile.settings, switches, &outdir),
            artefact: build.artefact,
            toolchain,
            lang: build.lang,
            cpprt: build.runtime.as_ref().map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),
            settings: profile.settings,
            is_testexe: false,

            defines: deps.defines,

            srcdir: PathBuf::from("src"),
            incdirs: deps.incdirs,
            libdirs: deps.libdirs,
            rpaths: deps.rpaths,
            outdir,

            pch: profile.pch,
            sources: fsutil::scan_for_filetype(Path::new("src"), &[build.lang.src_ext().to_string()]).unwrap(),
            headers,
            archives: deps.archives,
            relink: deps.relink,
            outfile: outfile.clone(),
            implib,

            comp_args: profile.compiler_options,
            link_args: profile.linker_options,
        })
    }

    pub fn from_test_cfg(build: &PackageManifest, switches: &BuildSwitches) -> Result<BuildInfo, Error> {
        if !std::fs::exists("test").unwrap_or_default() {
            return Err(Error::MissingTests(build.name.clone()));
        }

        // select toolchain in order of descending priority
        let toolchain = switches.toolchain.or(build.toolchain).unwrap_or(Toolchain::user_default()?);

        let include = std::env::current_exe()?.parent().unwrap().to_owned().join("testframework");

        let profile = build.get(&switches.profile)?;
        let headers = fsutil::scan_dirs_for_filetype(
            &["src".into(), "include".into(), include.join("vangotest")],
            &["h".into(), "hpp".into()],
        )?;

        let mut inherited = deps::libraries(&build, &profile.baseprof, switches)?;
        inherited.defines.push("VANGO_TEST".to_string());
        if cfg!(windows) {
            inherited.defines.push("UNICODE".to_string());
            inherited.defines.push("_UNICODE".to_string());
        }
        inherited.incdirs.extend(["test".into(), include, "src".into(), "include".into()]);

        let base_outdir = if toolchain == Toolchain::system_default()? {
            PathBuf::from("bin").join(switches.profile.to_string())
        } else {
            PathBuf::from("bin")
                .join(toolchain.as_directory())
                .join(switches.profile.to_string())
        };
        inherited.libdirs.push(base_outdir.clone());

        let outdir = base_outdir.join("test");
        let outfile = outdir.join(format!("test_{}.exe", build.name));
        let mut relink = Vec::new();
        if toolchain.is_msvc_compatible() {
            inherited.archives.insert(0, PathBuf::from(&build.name).with_extension("lib"));
            relink.push(base_outdir.join(&build.name).with_extension("lib"));
        } else {
            inherited.archives.insert(0, PathBuf::from(&build.name));
            relink.push(base_outdir.join(format!("lib{}", build.name)).with_extension("a"));
        }

        // replicate source directory hierarchy in output directory
        fsutil::ensure_out_dirs(Path::new("test"), &outdir);

        Ok(BuildInfo {
            artefact: crate::config::Artefact::Executable,
            toolchain,
            lang: build.lang,
            cpprt: build.runtime.clone().map(|rt| rt.eq_ignore_ascii_case("c++")).unwrap_or_default(),
            settings: profile.settings,
            changed: false,
            is_testexe: true,

            defines: inherited.defines,

            srcdir: "test".into(),
            incdirs: inherited.incdirs,
            libdirs: inherited.libdirs,
            rpaths: inherited.rpaths,
            outdir,

            pch: vec![],
            sources: fsutil::scan_for_filetype(&PathBuf::from("test"), &[build.lang.src_ext().to_string()]).unwrap(),
            headers,
            archives: inherited.archives,
            relink,
            outfile: outfile.clone(),
            implib: None,

            comp_args: vec![],
            link_args: vec![],
        })
    }
}
