use std::{ io::Write, path::{Path, PathBuf} };
use crate::{error::Error, input::BuildSwitches, repr::Lang, BuildFile, LibFile, log_info};


pub fn source_files(sdir: &Path, ext: &str) -> Result<Vec<PathBuf>, Error> {
    let mut res = Vec::new();

    for e in std::fs::read_dir(sdir)? {
        let e = e?;
        if e.path().is_dir() {
            res.extend(source_files(&e.path(), ext)?);
        } else if e.path().is_file() {
            let filename = e.path().file_name().unwrap().to_string_lossy().to_string();
            if filename.ends_with(ext) {
                res.push(e.path());
            }
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

pub fn libraries(libraries: Vec<String>, switches: &BuildSwitches, lang: Lang) -> Result<Dependencies, Error> {
    let home = std::env::home_dir().unwrap();

    let mut incdirs  = Vec::new();
    let mut libdirs  = Vec::new();
    let mut archives = Vec::new();
    let mut relink   = Vec::new();
    let mut defines  = Vec::new();
    let mut rebuilt  = false;

    for lib in libraries {
        let (root, _version) = split_version(&lib);

        let path = if root.ends_with(".git") {
            let url = std::path::Path::new(root);
            let stem = url.file_stem().unwrap().to_string_lossy();
            let dir = home.join(format!(".vango/packages/{stem}"));
            if !std::fs::exists(&dir).unwrap() {
                log_info!("cloning project dependency to: {:-<52}", format!("$ENV/packages/{stem} "));
                std::process::Command::new("git")
                    .arg("clone")
                    .arg(format!("{}", url.to_string_lossy()))
                    .arg(&dir)
                    .output()
                    .unwrap();
            }
            dir
        } else {
            PathBuf::from(root)
        };

        if !std::fs::exists(&path).unwrap() {
            return Err(Error::DirectoryNotFound(path))
        }

        if let Some(build) = if cfg!(target_os = "windows") && std::fs::exists(path.join("win.lib.json"))? {
            std::fs::read_to_string(path.join("win.lib.json")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(path.join("lnx.lib.json"))? {
            std::fs::read_to_string(path.join("lnx.lib.json")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(path.join("mac.lib.json"))? {
            std::fs::read_to_string(path.join("mac.lib.json")).ok()
        } else {
            std::fs::read_to_string(path.join("lib.json")).ok()
        } {
            let mut libinfo = LibFile::from_str(&build)?.validate(lang)?;
            let profile = libinfo.take(&switches.profile)?;
            incdirs.push(path.join(profile.include));
            libdirs.push(path.join(profile.libdir));
            if switches.toolchain.is_msvc() {
                archives.extend(profile.binaries.into_iter().map(|l| l.with_extension("lib")));
            } else {
                archives.extend(profile.binaries);
            }
            defines.extend(profile.defines);
        } else if let Some(build) = if cfg!(target_os = "windows") && std::fs::exists(path.join("win.build.json"))? {
            std::fs::read_to_string(path.join("win.build.json")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(path.join("lnx.build.json"))? {
            std::fs::read_to_string(path.join("lnx.build.json")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(path.join("mac.build.json"))? {
            std::fs::read_to_string(path.join("mac.build.json")).ok()
        } else {
            std::fs::read_to_string(path.join("build.json")).ok()
        } {
            let build = BuildFile::from_str(&build)?;
            log_info!("building project dependency: {:-<54}", format!("{} ", build.project));
            let save = std::env::current_dir().unwrap();
            std::env::set_current_dir(&path).unwrap();
            let output = std::process::Command::new("vango")
                .arg("build")
                .arg(switches.profile.as_arg())
                .arg(switches.toolchain.as_arg())
                .args(if switches.crtstatic { Some("--crtstatic") } else { None })
                .args(if switches.verbose { Some("-v") } else { None })
                .args(if switches.echo { Some("--echo") } else { None })
                .status()
                .unwrap();
            if output.code() == Some(8) {
                rebuilt = true;
            } else {
                println!();
            }
            std::env::set_current_dir(&save).unwrap();
            let mut libinfo = LibFile::from(build).validate(lang)?;
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
            defines.extend(profile.defines);
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

