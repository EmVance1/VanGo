use crate::{error::Error, log_info, repr::ToolChain, BuildFile, Config, LibFile};
use std::{
    io::Write,
    path::{Path, PathBuf},
};


#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub path: PathBuf,
    pub repr: String,
    pub exists: bool,
    pub modified: Option<std::time::SystemTime>,
}

#[allow(dead_code)]
impl FileInfo {
    pub fn from_path(path: &Path) -> Self {
        let exists = path.exists();
        let modified = if exists {
            Some(std::fs::metadata(path).unwrap().modified().unwrap())
        } else {
            None
        };
        let path = path.to_owned();
        let repr = path.to_string_lossy().to_string();

        Self {
            path,
            repr,
            exists,
            modified,
        }
    }

    pub fn from_str(path: &str) -> Self {
        Self::from_path(&PathBuf::from(path))
    }

    pub fn file_name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }
    pub fn exists(&self) -> bool {
        self.exists
    }
    pub fn modified(&self) -> Option<std::time::SystemTime> {
        self.modified
    }
}


pub fn source_files(sdir: &Path, ext: &str) -> Option<Vec<FileInfo>> {
    let mut res = Vec::new();

    for e in std::fs::read_dir(sdir).ok()? {
        let e = e.ok()?;
        if e.path().is_dir() {
            res.extend(source_files(&e.path(), ext)?);
        } else {
            let filename = e.path().file_name()?.to_str()?.to_string();
            if filename.ends_with(ext) {
                res.push(FileInfo::from_path(&e.path()));
            }
        }
    }

    Some(res)
}


#[derive(Debug, Clone)]
pub struct Dependencies {
    pub incdirs: Vec<String>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
    pub relink: Vec<FileInfo>,
    pub defines: Vec<String>,
    pub rebuilt: bool,
}

pub fn libraries(libraries: Vec<String>, config: Config, toolchain: ToolChain, verbose: bool, maxcpp: &str) -> Result<Dependencies, Error> {
    let home = std::env::home_dir().unwrap().to_string_lossy().to_string();

    let mut incdirs = Vec::new();
    let mut libdirs = Vec::new();
    let mut links = Vec::new();
    let mut relink = Vec::new();
    let mut defines = Vec::new();
    let mut rebuilt = false;

    for lib in libraries {
        let (root, version) = split_version(&lib);

        let path = if root.ends_with(".git") {
            let url = std::path::Path::new(root);
            let stem = url.file_stem().unwrap().to_string_lossy();
            let dir = format!("{home}/.vango/packages/{stem}");
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
            root.to_string()
        };

        if let Some(build) = if cfg!(target_os = "windows") && std::fs::exists(format!("{path}/win.lib.json")).unwrap() {
            std::fs::read_to_string(format!("{path}/win.lib.json")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(format!("{path}/linux.lib.json")).unwrap() {
            std::fs::read_to_string(format!("{path}/linux.lib.json")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(format!("{path}/macos.lib.json")).unwrap() {
            std::fs::read_to_string(format!("{path}/macos.lib.json")).ok()
        } else {
            std::fs::read_to_string(format!("{path}/lib.json")).ok()
        } {
            let libinfo = LibFile::from_str(&build)?
                .validate(maxcpp)?
                .linearise(config, version)?;
            incdirs.push(format!("{path}/{}", libinfo.incdir));
            if let Some(libdir) = libinfo.libdir {
                libdirs.push(format!("{path}/{libdir}"));
            }
            links.extend(libinfo.links);
            defines.extend(libinfo.defines);
        } else if let Some(build) = if cfg!(target_os = "windows") && std::fs::exists(format!("{path}/win.build.json")).unwrap() {
            std::fs::read_to_string(format!("{path}/win.build.json")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(format!("{path}/linux.build.json")).unwrap() {
            std::fs::read_to_string(format!("{path}/linux.build.json")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(format!("{path}/macos.build.json")).unwrap() {
            std::fs::read_to_string(format!("{path}/macos.build.json")).ok()
        } else {
            std::fs::read_to_string(format!("{path}/build.json")).ok()
        } {
            let build: BuildFile = serde_json::from_str(&build)?;
            log_info!("building project dependency: {:-<54}", format!("{} ", build.project));
            let save = std::env::current_dir().unwrap();
            std::env::set_current_dir(&path).unwrap();
            let output = std::process::Command::new("vango")
                .arg("build")
                .arg(config.as_arg())
                .arg(toolchain.as_arg())
                .args(if verbose { Some("-v") } else { None })
                .status()
                .unwrap();
            if output.code() == Some(8) {
                rebuilt = true;
            } else {
                println!();
            }
            std::env::set_current_dir(&save).unwrap();
            let libinfo = LibFile::from(build)
                .validate(maxcpp)?
                .linearise(config, version)?;
            incdirs.push(format!("{path}/{}", libinfo.incdir));
            if let Some(libdir) = &libinfo.libdir {
                libdirs.push(format!("{path}/{libdir}"));
            }
            if toolchain.is_msvc() {
                for l in &libinfo.links {
                    relink.push(FileInfo::from_str(&format!("{path}/{}/{}.lib", libinfo.libdir.as_ref().unwrap(), l)));
                    links.push(format!("{l}.lib"));
                }
            } else {
                for l in &libinfo.links {
                    relink.push(FileInfo::from_str(&format!("{path}/{}/lib{}.a", libinfo.libdir.as_ref().unwrap(), l)));
                    links.push(l.to_string());
                }
            }
            defines.extend(libinfo.defines);
        }
    }

    Ok(Dependencies {
        incdirs,
        libdirs,
        links,
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
