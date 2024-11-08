use std::path::{Path, PathBuf};
use crate::{error::Error, repr::{Dependencies, ProjKind, Config}, BuildDef, LibDef};
use crate::log_info;
use std::io::Write;


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

pub fn get_project_kind(srcs: &[FileInfo]) -> Result<ProjKind, Error> {
    for s in srcs {
        if s.file_name() == "main.cpp" {
            return Ok(ProjKind::App)
        } else if s.file_name() == "lib.cpp" {
            return Ok(ProjKind::Lib)
        }
    }
    Err(Error::MissingEntryPoint)
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
        let (name, cfg) = if let Some((n, c)) = get_cfg(&dep) {
            (n, Some(c))
        } else {
            (dep.as_str(), None)
        };
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
    let libdef: LibDef = serde_json::from_str(src).map_err(|e| Error::JsonParse(e))?;
    if u32_from_cppstd(&libdef.minstd) > u32_from_cppstd(cpp) {
        return Err(Error::IncompatibleCpp(libdef.library))
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


pub fn u32_from_cppstd(cpp: &str) -> u32 {
    let cpp: u32 = cpp.to_ascii_lowercase()
        .strip_prefix("c++")
        .unwrap()
        .parse()
        .unwrap();
    if cpp < 50 {
        100 + cpp
    } else {
        cpp
    }
}


fn get_cfg(s: &str) -> Option<(&str, &str)> {
    for (i, c) in s.chars().rev().enumerate() {
        if c == '/' || c == '\\' {
            return None
        } else if c == '.' {
            let l = s.len();
            return Some((&s[..(l-i-1)], &s[(l-i)..]))
        }
    }
    None
}

