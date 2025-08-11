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
            if filename.ends_with(ext) && filename != "pch.cpp" {
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

pub fn libraries(libraries: Vec<String>, config: Config, toolchain: ToolChain, maxcpp: &str) -> Result<Dependencies, Error> {
    let mut incdirs = Vec::new();
    let mut libdirs = Vec::new();
    let mut links = Vec::new();
    let mut relink = Vec::new();
    let mut defines = Vec::new();
    let mut rebuilt = false;

    for lib in libraries {
        let (name, version) = get_version(&lib);

        if name.ends_with(".git") {
            let url = std::path::Path::new(name);
            let home = std::env::home_dir().unwrap().to_string_lossy().to_string();
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

            if let Ok(build) = std::fs::read_to_string(format!("{dir}/lib.json")) {
                let libinfo = LibFile::from_str(&build)?
                    .validate(maxcpp)?
                    .linearise(config, version)?;
                incdirs.push(format!("{dir}/{}", libinfo.incdir));
                libdirs.push(format!("{dir}/{}", libinfo.libdir));
                links.extend(libinfo.links);
                defines.extend(libinfo.defines);
            } else if let Ok(build) = std::fs::read_to_string(format!("{dir}/build.json")) {
                let build: BuildFile = serde_json::from_str(&build)?;
                log_info!("building project dependency: {:-<54}", format!("{} ", build.project));
                let save = std::env::current_dir().unwrap();
                std::env::set_current_dir(&dir).unwrap();
                let mut cmd = std::process::Command::new("vango");
                cmd.arg("build")
                    .arg(config.as_arg())
                    .arg(toolchain.as_arg());
                let output = cmd.status().unwrap();
                if output.code() == Some(8) {
                    rebuilt = true;
                } else {
                    println!();
                }
                std::env::set_current_dir(&save).unwrap();
                let libinfo = LibFile::from(build)
                    .validate(maxcpp)?
                    .linearise(config, version)?;
                incdirs.push(format!("{dir}/{}", libinfo.incdir));
                libdirs.push(format!("{dir}/{}", libinfo.libdir));
                if toolchain.is_msvc() {
                    for l in &libinfo.links {
                        relink.push(FileInfo::from_str(&format!("{dir}/{}/{}", libinfo.libdir, l)));
                        links.push(format!("{l}.lib"));
                    }
                } else {
                    for l in &libinfo.links {
                        relink.push(FileInfo::from_str(&format!("{name}/{}/lib{}.a", libinfo.libdir, l)));
                        links.push(l.to_string());
                    }
                }
                defines.extend(libinfo.defines);
            }
        }

        if let Some(build) = if cfg!(target_os = "windows") && std::fs::exists(format!("{name}/win.lib.json")).unwrap() {
            std::fs::read_to_string(format!("{name}/win.lib.json")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(format!("{name}/linux.lib.json")).unwrap() {
            std::fs::read_to_string(format!("{name}/linux.lib.json")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(format!("{name}/macos.lib.json")).unwrap() {
            std::fs::read_to_string(format!("{name}/macos.lib.json")).ok()
        } else {
            std::fs::read_to_string(format!("{name}/lib.json")).ok()
        } {
            let libinfo = LibFile::from_str(&build)?
                .validate(maxcpp)?
                .linearise(config, version)?;
            incdirs.push(format!("{name}/{}", libinfo.incdir));
            libdirs.push(format!("{name}/{}", libinfo.libdir));
            links.extend(libinfo.links);
            defines.extend(libinfo.defines);
        } else if let Some(build) = if cfg!(target_os = "windows") && std::fs::exists(format!("{name}/win.build.json")).unwrap() {
            std::fs::read_to_string(format!("{name}/win.build.json")).ok()
        } else if cfg!(target_os = "linux") && std::fs::exists(format!("{name}/linux.build.json")).unwrap() {
            std::fs::read_to_string(format!("{name}/linux.build.json")).ok()
        } else if cfg!(target_os = "macos") && std::fs::exists(format!("{name}/macos.build.json")).unwrap() {
            std::fs::read_to_string(format!("{name}/macos.build.json")).ok()
        } else {
            std::fs::read_to_string(format!("{name}/build.json")).ok()
        } {
            let build: BuildFile = serde_json::from_str(&build)?;
            log_info!("building project dependency: {:-<54}", format!("{} ", build.project));
            let save = std::env::current_dir().unwrap();
            std::env::set_current_dir(name).unwrap();
            let mut cmd = std::process::Command::new("vango");
            cmd.arg("build")
                .arg(config.as_arg())
                .arg(toolchain.as_arg());
            let output = cmd.status().unwrap();
            if output.code() == Some(8) {
                rebuilt = true;
            } else {
                println!();
            }
            std::env::set_current_dir(&save).unwrap();
            let libinfo = LibFile::from(build)
                .validate(maxcpp)?
                .linearise(config, version)?;
            incdirs.push(format!("{name}/{}", libinfo.incdir));
            libdirs.push(format!("{name}/{}", libinfo.libdir));
            if toolchain.is_msvc() {
                for l in &libinfo.links {
                    relink.push(FileInfo::from_str(&format!("{name}/{}/{}.lib", libinfo.libdir, l)));
                    links.push(format!("{l}.lib"));
                }
            } else {
                for l in &libinfo.links {
                    relink.push(FileInfo::from_str(&format!("{name}/{}/lib{}.a", libinfo.libdir, l)));
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


fn get_version(s: &str) -> (&str, Option<&str>) {
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
        assert_eq!(get_version("SFML"), ("SFML", None));
        assert_eq!(get_version("SFML:static"), ("SFML", Some("static")));
        assert_eq!(get_version("SF.ML:static"), ("SF.ML", Some("static")));
        assert_eq!(get_version("SFML-2.6.1"), ("SFML-2.6.1", None));
        assert_eq!(get_version("../Rusty"), ("../Rusty", None));
        assert_eq!(get_version("../Rusty:static"), ("../Rusty", Some("static")));
        assert_eq!(
            get_version("../Ru.sty:static"),
            ("../Ru.sty", Some("static"))
        );
    }
}
