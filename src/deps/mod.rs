pub mod git;
mod vcpkg;

use crate::{
    Error,
    cli::BuildSwitches,
    config::{Dependency, LibManifest, PackageManifest, Profile, VangoFile},
    exec::Toolchain,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct Dependencies {
    pub incdirs: Vec<PathBuf>,
    pub libdirs: Vec<PathBuf>,
    pub rpaths: Vec<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub relink: Vec<PathBuf>,
    pub defines: Vec<String>,
}

pub fn libraries(info: &PackageManifest, profile: &Profile, switches: &BuildSwitches) -> Result<Dependencies, Error> {
    let mut deps = Dependencies::default();
    let home = std::env::home_dir().unwrap();

    // recursive builds only forward base (inherited) profile, custom profiles ignored
    let switches = if let Profile::Custom(..) = switches.profile {
        BuildSwitches {
            profile: profile.clone(),
            ..switches.clone()
        }
    } else {
        switches.clone()
    };

    // select toolchain in order of descending priority
    let toolchain = switches.toolchain.or(info.toolchain).unwrap_or(Toolchain::user_default()?);

    let mut vcpkg = Vec::new();

    for lib in &info.dependencies {
        // get path to library root, pull repo if necessary
        let path = match &lib.1 {
            Dependency::Git { git, tag, features: _ } => {
                let git = Path::new(&git);
                let stem = git.file_stem().unwrap().to_string_lossy();
                let path = home.join(format!(".vango/packages/{stem}"));
                if !std::fs::exists(&path).unwrap() {
                    git::pull_package(git, tag, &path);
                }
                path
            }
            Dependency::Package { src, targets, features } => {
                if src == "vcpkg" {
                    vcpkg.push(vcpkg::VcpkgDependency {
                        name: lib.0.to_ascii_lowercase(),
                        features: features.clone(),
                    });
                    for tar in targets {
                        if toolchain.is_msvc_compatible() {
                            deps.archives.push(tar.with_extension("lib"));
                        } else {
                            deps.archives.push(tar.clone());
                        }
                    }
                    continue;
                } else {
                    src.clone()
                }
            }
            Dependency::Headers { headers, features: _ } => {
                deps.incdirs.push(headers.clone());
                continue;
            }
            Dependency::System { system } => {
                if toolchain.is_msvc_compatible() {
                    deps.archives.push(system.with_extension("lib"));
                } else {
                    deps.archives.push(system.clone());
                }
                continue;
            }
        };

        if !std::fs::exists(&path).unwrap() {
            return Err(Error::DirectoryNotFound(path));
        }

        let mut srcpkg = false;
        let save = std::env::current_dir().unwrap();
        std::env::set_current_dir(&path).unwrap();
        let mut library = match VangoFile::from_str(&crate::read_manifest()?)? {
            VangoFile::Build(build) => {
                // could use .validate(), but prefer checking *before* build to save user time
                if build.interface > info.lang {
                    return Err(Error::IncompatibleCppStd(build.name, build.interface, info.name.clone(), info.lang));
                }
                srcpkg = true;
                crate::action::build(&build, &switches, 0)?;
                LibManifest::from_build(build, toolchain)?
            }
            VangoFile::Lib(lib) => lib.validate(&info.name, info.lang)?,
        };
        std::env::set_current_dir(&save).unwrap();

        // collect all dependency artefacts (includes, definitions, libraries, libdirs) into SOA
        let profile = library.remove(&switches.profile)?;
        deps.incdirs.push(path.join(profile.include));
        deps.libdirs.push(path.join(&profile.libdir));
        if toolchain.is_msvc_compatible() {
            for l in profile.binaries {
                if srcpkg {
                    deps.relink.push(path.join(&profile.libdir).join(&l).with_extension("lib"));
                }
                deps.archives.push(l.with_extension("lib"));
            }
        } else {
            for l in profile.binaries {
                if srcpkg {
                    deps.relink
                        .push(path.join(&profile.libdir).join(format!("lib{}", l.display())).with_extension("a"));
                }
                deps.archives.push(l);
            }
        }

        // no vango generated definitions are propagated - all such defs are tailored to the project being built
        deps.defines
            .extend(profile.defines.into_iter().filter(|d| !d.starts_with("VANGO_")));
    }

    vcpkg::pull_package(vcpkg, &info.vcpkg.triplet, &mut deps);

    Ok(deps)
}
