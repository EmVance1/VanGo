use std::{ffi::OsStr, io::Write, path::{Path, PathBuf}};
use crate::{config::{Dependency, Lang, LibFile, VangoFile}, error::Error, input::BuildSwitches, log_info_ln};


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


#[derive(Debug, Clone)]
pub struct Dependencies {
    pub incdirs:  Vec<PathBuf>,
    pub libdirs:  Vec<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub relink:   Vec<PathBuf>,
    pub defines:  Vec<String>,
    pub rebuilt:  bool,
}

pub fn libraries(libraries: Vec<Dependency>, switches: &BuildSwitches, lang: Lang) -> Result<Dependencies, Error> {
    let home = std::env::home_dir().unwrap();

    let mut incdirs  = Vec::new();
    let mut libdirs  = Vec::new();
    let mut archives = Vec::new();
    let mut relink   = Vec::new();
    let mut defines  = Vec::new();
    let mut rebuilt  = false;

    for lib in libraries {
        let path = match lib {
            #[allow(unused)]
            Dependency::Git { git, tag, recipe, features } => {
                let url = std::path::Path::new(&git);
                let stem = url.file_stem().unwrap().to_string_lossy();
                let dir = home.join(format!(".vango/packages/{stem}"));
                if !std::fs::exists(&dir).unwrap() {
                    let version: Vec<PathBuf> = if let Some(tag) = tag {
                        vec![ "--branch".into(), tag.into(), "--depth".into(), "1".into() ]
                    } else {
                        vec![]
                    };
                    log_info_ln!("cloning project dependency to: {:-<52}", format!("$ENV/packages/{stem} "));
                    std::process::Command::new("git")
                        .arg("clone")
                        .args(version)
                        .arg(format!("{}", url.to_string_lossy()))
                        .arg(&dir)
                        .output()
                        .unwrap();
                    if let Some(recipe) = recipe {
                        log_info_ln!("building project dependency according to '{}'", recipe.display());
                        std::process::Command::new(PathBuf::from(".").join(recipe))
                            .current_dir(&dir)
                            .output()
                            .unwrap();
                    }
                }
                dir
            }
            #[allow(unused)]
            Dependency::Local { path, features } => {
                path
            }
            #[allow(unused)]
            Dependency::Headers { headers, features } => {
                incdirs.push(headers);
                continue;
            }
        };

        if !std::fs::exists(&path).unwrap() {
            return Err(Error::DirectoryNotFound(path))
        }

        if let Some(build) = if cfg!(windows) && std::fs::exists(path.join("win.vango.toml"))? {
            std::fs::read_to_string(path.join("win.vango.toml")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(path.join("lnx.vango.toml"))? {
            std::fs::read_to_string(path.join("lnx.vango.toml")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(path.join("mac.vango.toml"))? {
            std::fs::read_to_string(path.join("mac.vango.toml")).ok()
        } else {
            std::fs::read_to_string(path.join("vango.toml")).ok()
        } {
            match VangoFile::from_str(&build)? {
                VangoFile::Build(build) => {
                    log_info_ln!("building project dependency: {:-<54}", format!("{} ", build.name));
                    let save = std::env::current_dir().unwrap();
                    std::env::set_current_dir(&path).unwrap();
                    let (_rebuilt, _) = crate::action::build(build.clone(), switches.clone())?;
                    if _rebuilt {
                        rebuilt = true;
                    } else {
                        println!();
                    }
                    std::env::set_current_dir(&save).unwrap();
                    let mut libinfo = LibFile::try_from(build)?.validate(lang)?;
                    let profile = libinfo.take(&switches.profile)?;
                    incdirs.push(path.join(profile.include));
                    libdirs.push(path.join(&profile.libdir));
                    if switches.toolchain.is_msvc() {
                        for l in profile.binaries {
                            relink.push(path.join(&profile.libdir).join(&l).with_extension("lib"));
                            archives.push(l.with_extension("lib"));
                        }
                    } else {
                        for l in profile.binaries {
                            relink.push(path.join(&profile.libdir).join(format!("lib{}", l.display())).with_extension("a"));
                            archives.push(l);
                        }
                    }
                    defines.extend(profile.defines.into_iter().filter(|d| !d.starts_with("VANGO_")));
                }
                VangoFile::Lib(mut lib) => {
                    let profile = lib.take(&switches.profile)?;
                    incdirs.push(path.join(profile.include));
                    libdirs.push(path.join(profile.libdir));
                    if switches.toolchain.is_msvc() {
                        archives.extend(profile.binaries.into_iter().map(|l| l.with_extension("lib")));
                    } else {
                        archives.extend(profile.binaries);
                    }
                    defines.extend(profile.defines.into_iter().filter(|d| !d.starts_with("VANGO_")));
                }
            }
        } else {
            return Err(Error::MissingBuildScript(path))
        }
    }

    Ok(Dependencies {
        incdirs,
        libdirs,
        archives,
        relink,
        defines,
        rebuilt,
    })
}


/*
fn split_version(s: &str) -> (&str, Option<&str>) {
    for (i, c) in s.chars().rev().enumerate() {
        if c == '/' || c == '\\' {
            return (s, None);
        } else if c == ':' {
            let l = s.len();
            return (&s[..(l - i - 1)], Some(&s[(l - i)..]));
        }
    }
    (s, None)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_version() {
        assert_eq!(split_version("SFML"),             ("SFML",       None));
        assert_eq!(split_version("SFML:static"),      ("SFML",       Some("static")));
        assert_eq!(split_version("SF.ML:static"),     ("SF.ML",      Some("static")));
        assert_eq!(split_version("SFML-2.6.1"),       ("SFML-2.6.1", None));
        assert_eq!(split_version("../Rusty"),         ("../Rusty",   None));
        assert_eq!(split_version("../Rusty:static"),  ("../Rusty",   Some("static")));
        assert_eq!(split_version("../Ru.sty:static"), ("../Ru.sty",  Some("static")));
    }
}
*/

