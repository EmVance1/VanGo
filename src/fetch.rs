use std::{ffi::OsStr, path::{Path, PathBuf}};
use crate::{config::{VangoFile, BuildFile, LibFile, Dependency, Profile}, input::BuildSwitches, error::Error, log_info_ln};


pub fn source_files(sdir: &Path, ext: &str) -> Result<Vec<PathBuf>, Error> {
    let mut res = Vec::new();

    for e in std::fs::read_dir(sdir)? {
        let e = e?;
        if e.path().is_dir() {
            res.extend(source_files(&e.path(), ext)?);
        } else if e.path().is_file() && e.path().extension().unwrap_or(OsStr::new("")) == ext {
            res.push(e.path());
        }
    }

    Ok(res)
}


pub fn pull_git_repo(url: &Path, tag: &Option<String>, recipe: &Option<PathBuf>, install_loc: &Path) {
    let branch: Vec<PathBuf> = if let Some(tag) = tag {
        vec![ "--branch".into(), tag.into(), "--depth".into(), "1".into(), url.into() ]
    } else {
        vec![ url.into() ]
    };
    log_info_ln!("{:-<80}", format!("cloning project dependency to: {} ", install_loc.display()));
    std::process::Command::new("git")
        .arg("clone")
        .args(branch)
        .arg(install_loc)
        .output()
        .unwrap();
    if let Some(recipe) = recipe {
        log_info_ln!("building project dependency according to '{}'", recipe.display());
        std::process::Command::new(PathBuf::from(".").join(recipe))
            .current_dir(install_loc)
            .output()
            .unwrap();
    }
}


#[derive(Debug, Default, Clone)]
pub struct Dependencies {
    pub incdirs:  Vec<PathBuf>,
    pub libdirs:  Vec<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub relink:   Vec<PathBuf>,
    pub defines:  Vec<String>,
}

pub fn libraries(info: &BuildFile, profile: &Profile, switches: &BuildSwitches) -> Result<Dependencies, Error> {
    let mut deps = Dependencies::default();
    let home = std::env::home_dir().unwrap();

    // recursive builds only forward base (inherited) profile, custom profiles ignored
    let switches = if let Profile::Custom(..) = switches.profile {
        BuildSwitches{ profile: profile.clone(), ..switches.clone() }
    } else {
        switches.clone()
    };

    for lib in &info.dependencies {
        // get path to library root, pull repo if necessary
        let path = match lib {
            Dependency::Git { git, tag, recipe, features: _ } => {
                let git = Path::new(&git);
                let stem = git.file_stem().unwrap().to_string_lossy();
                let path = home.join(format!(".vango/packages/{stem}"));
                if !std::fs::exists(&path).unwrap() {
                    pull_git_repo(git, tag, recipe, &path);
                }
                path
            }
            Dependency::Local { path, features: _ } => {
                path.clone()
            }
            Dependency::Headers { headers, features: _ } => {
                deps.incdirs.push(headers.clone());
                continue;
            }
            Dependency::System { system, target } => {
                if let Some(target) = target &&
                    (target == "windows" && !cfg!(windows) ||
                    target == "unix"  && !cfg!(unix) ||
                    target == "linux" && !cfg!(target_os="linux") ||
                    target == "macos" && !cfg!(target_os="macos")) {
                    continue;
                }
                if switches.toolchain.is_msvc() {
                    deps.archives.push(system.with_extension("lib"));
                } else {
                    deps.archives.push(system.clone());
                }
                continue;
            }
        };

        if !std::fs::exists(&path).unwrap() {
            return Err(Error::DirectoryNotFound(path))
        }

        let mut srcpkg = false;
        let save = std::env::current_dir().unwrap();
        std::env::set_current_dir(&path).unwrap();
        let mut library = match VangoFile::from_str(&crate::read_manifest()?)? {
            VangoFile::Build(build) => {
                // could use .validate(), but prefer checking *before* build to save user time
                if build.interface > info.lang {
                    return Err(Error::IncompatibleCppStd(build.name, build.interface, info.name.clone(), info.lang))
                }
                srcpkg = true;
                crate::action::build(&build, &switches, true)?;
                LibFile::from_build(build, switches.toolchain)?
            }
            VangoFile::Lib(lib) => lib.validate(&info.name, info.lang)?,
        };
        std::env::set_current_dir(&save).unwrap();

        // collect all dependency artefacts (includes, definitions, libraries, libdirs) into SOA
        let profile = library.take(&switches.profile)?;
        deps.incdirs.push(path.join(profile.include));
        deps.libdirs.push(path.join(&profile.libdir));
        if switches.toolchain.is_msvc() {
            for l in profile.binaries {
                if srcpkg { deps.relink.push(path.join(&profile.libdir).join(&l).with_extension("lib")); }
                deps.archives.push(l.with_extension("lib"));
            }
        } else {
            for l in profile.binaries {
                if srcpkg { deps.relink.push(path.join(&profile.libdir).join(format!("lib{}", l.display())).with_extension("a")); }
                deps.archives.push(l);
            }
        }

        // no vango generated definitions are propagated - all such defs are tailored to the project being built
        deps.defines.extend(profile.defines.into_iter().filter(|d| !d.starts_with("VANGO_")));
    }

    Ok(deps)
}

