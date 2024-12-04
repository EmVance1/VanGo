use std::{
    path::{Path, PathBuf},
    io::Write,
};
use crate::{
    BuildDef, LibDef, ProjKind, Config,
    error::Error,
    log_info,
};


#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub path: PathBuf,
    pub repr: String,
    pub exists: bool,
    pub modified: Option<std::time::SystemTime>
}

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

        Self{
            path,
            repr,
            exists,
            modified,
        }
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


pub fn get_source_files(sdir: &Path, ext: &str) -> Option<Vec<FileInfo>> {
    let mut res = Vec::new();

    for e in std::fs::read_dir(sdir).ok()? {
        let e = e.ok()?;
        if e.path().is_dir() {
            res.extend(get_source_files(&e.path(), ext)?);
        } else {
            let filename = e.path().file_name()?.to_str()?.to_string();
            if filename.ends_with(ext) && filename != "pch.cpp" {
                res.push(FileInfo::from_path(&e.path()));
            }
        }
    }

    Some(res)
}

pub fn get_project_kind(sdir: &Path) -> Result<ProjKind, Error> {
    for e in std::fs::read_dir(sdir).map_err(|_| Error::MissingEntryPoint)? {
        let e = e.map_err(|_| Error::MissingEntryPoint)?;
        if e.path().is_file() {
            let filename = e.path().file_name().unwrap().to_str().unwrap().to_string();
            if filename.ends_with("main.cpp") {
                return Ok(ProjKind::App)
            }
            if filename.ends_with("main.c") {
                return Ok(ProjKind::App)
            }
            if filename.ends_with("lib.hpp") {
                return Ok(ProjKind::Lib)
            }
            if filename.ends_with("lib.h") {
                return Ok(ProjKind::Lib)
            }
        }
    }
    Err(Error::MissingEntryPoint)
}

pub fn _get_project_kind(srcs: &[FileInfo], headers: &[FileInfo]) -> Result<ProjKind, Error> {
    for s in srcs {
        if s.file_name() == "main.cpp" || s.file_name() == "main.c" {
            return Ok(ProjKind::App)
        }
    }
    for s in headers {
        if s.file_name() == "lib.hpp" || s.file_name() == "lib.h" {
            return Ok(ProjKind::Lib)
        }
    }
    Err(Error::MissingEntryPoint)
}


#[derive(Debug, Clone)]
pub struct Dependencies {
    pub incdirs: Vec<String>,
    pub headers: Vec<FileInfo>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
    pub defines: Vec<String>,
}


pub fn get_dependencies(incs: Vec<String>, deps: Vec<String>, config: Config, cpp: &str) -> Result<Dependencies, Error> {
    let mut incdirs = Vec::new();
    let mut headers = Vec::new();
    let mut libdirs = Vec::new();
    let mut links = Vec::new();
    let mut defines = Vec::new();

    for inc in incs {
        headers.extend(get_source_files(&PathBuf::from(&inc), ".h").unwrap());
        incdirs.push(inc);
    }

    for dep in deps {
        let (name, cfg) = get_cfg(&dep);

        if let Ok(build) = std::fs::read_to_string(format!("lib/{}/lib.json", name)) {
            let libinfo = get_lib_info(&build, cfg, config, cpp)?;
            incdirs.push(format!("lib/{}/{}", name, libinfo.incdir));
            libdirs.push(format!("lib/{}/{}", name, libinfo.libdir));
            links.extend(libinfo.links);
            defines.extend(libinfo.defines);
        } else if let Ok(build) = std::fs::read_to_string(format!("{}/lib.json", name)) {
            let libinfo = get_lib_info(&build, cfg, config, cpp)?;
            incdirs.push(format!("{}/{}", name, libinfo.incdir));
            libdirs.push(format!("{}/{}", name, libinfo.libdir));
            links.extend(libinfo.links);
            defines.extend(libinfo.defines);
        } else if let Ok(build) = std::fs::read_to_string(format!("{}/build.json", name)) {
            let lib: BuildDef = serde_json::from_str(&build).unwrap();
            log_info!("building project dependency: {}", lib.project);
            let save = std::env::current_dir().unwrap();
            std::env::set_current_dir(name).unwrap();
            std::process::Command::new("mscmp")
                .arg("build")
                .arg(format!("-{}", config))
                .status()
                .unwrap();
            std::env::set_current_dir(&save).unwrap();
            incdirs.push(format!("{}/include", name));
            libdirs.push(format!("{}/bin/{}", name, config));
            links.push(format!("{}.lib", lib.project));
            println!();
        }
    }

    Ok(Dependencies{
        incdirs,
        headers,
        libdirs,
        links,
        defines,
    })
}


#[derive(Debug, Clone)]
pub struct LibInfo {
    pub incdir: String,
    pub libdir: String,
    pub links: Vec<String>,
    pub defines: Vec<String>,
}

fn get_lib_info(src: &str, cfg: Option<&str>, config: Config, cpp: &str) -> Result<LibInfo, Error> {
    let libdef: LibDef = serde_json::from_str(src).map_err(Error::JsonParse)?;
    if u32_from_cppstd(&libdef.minstd)? > u32_from_cppstd(cpp)? {
        return Err(Error::IncompatibleCppStd(libdef.library))
    }
    if let Some(cfg) = cfg {
        for (n, c) in libdef.configs {
            if n == cfg {
                return Ok(LibInfo {
                    incdir: libdef.include,
                    links: c.links,
                    defines: c.defines,
                    libdir: if config.is_release() { c.binary_release } else { c.binary_debug }
                })
            }
        }
        Err(Error::ConfigUnavailable(libdef.library, cfg.to_string()))
    } else {
        if let Some(defconf) = libdef.all {
            return Ok(LibInfo {
                incdir: libdef.include,
                links: defconf.links,
                defines: defconf.defines,
                libdir: if config.is_release() { defconf.binary_release } else { defconf.binary_debug }
            })
        }
        Err(Error::ConfigUnavailable(libdef.library, "default".to_string()))
    }
}


pub fn u32_from_cppstd(cpp: &str) -> Result<u32, Error> {
    if cpp.to_ascii_lowercase() == "c" {
        return Ok(0)
    }

    let num: u32 = cpp.to_ascii_lowercase()
        .strip_prefix("c++")
        .ok_or(Error::InvalidCppStd(cpp.to_string()))?
        .parse()
        .map_err(|_| Error::InvalidCppStd(cpp.to_string()))?;
    if !matches!(num, 98|3|11|14|17|20|23) {
        Err(Error::InvalidCppStd(cpp.to_string()))
    } else if num < 50 {
        Ok(100 + num)
    } else {
        Ok(num)
    }
}


fn get_cfg(s: &str) -> (&str, Option<&str>) {
    for (i, c) in s.chars().rev().enumerate() {
        if c == '/' || c == '\\' {
            return (s, None)
        } else if c == '.' {
            let l = s.len();
            return (&s[..(l-i-1)], Some(&s[(l-i)..]))
        }
    }
    (s, None)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_u32_from_cppstd() {
        assert_eq!(u32_from_cppstd("c++98").unwrap(), 98);
        assert_eq!(u32_from_cppstd("c++03").unwrap(), 103);
        assert_eq!(u32_from_cppstd("c++11").unwrap(), 111);
        assert_eq!(u32_from_cppstd("c++14").unwrap(), 114);
        assert_eq!(u32_from_cppstd("c++17").unwrap(), 117);
        assert_eq!(u32_from_cppstd("c++20").unwrap(), 120);
        assert_eq!(u32_from_cppstd("c++23").unwrap(), 123);

        assert!(u32_from_cppstd("c++24").is_err());
        assert!(u32_from_cppstd("c++").is_err());
        assert!(u32_from_cppstd("c23").is_err());
        assert!(u32_from_cppstd("3").is_err());
    }

    #[test]
    pub fn test_get_cfg() {
        assert_eq!(get_cfg("SFML"),             ("SFML", None));
        assert_eq!(get_cfg("SFML.static"),      ("SFML", Some("static")));
        assert_eq!(get_cfg("SF.ML.static"),     ("SF.ML", Some("static")));
        assert_eq!(get_cfg("../Rusty"),         ("../Rusty", None));
        assert_eq!(get_cfg("../Rusty.static"),  ("../Rusty", Some("static")));
        assert_eq!(get_cfg("../Ru.sty.static"), ("../Ru.sty", Some("static")));
    }
}

