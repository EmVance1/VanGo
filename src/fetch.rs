use std::{path::{Path, PathBuf}, collections::HashMap};
use crate::{repr::{Dependencies, ProjKind}, LibDef};


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

pub fn get_dependencies(incs: Vec<String>, deps: HashMap<String, LibDef>) -> Dependencies {
    let mut incdirs = Vec::new();
    let mut headers = Vec::new();
    let mut libdirs = Vec::new();
    let mut links = Vec::new();

    for inc in incs {
        headers.extend(get_source_files(&PathBuf::from(&inc), ".h").unwrap());
        incdirs.push(inc);
    }

    for (_, data) in deps {
        incdirs.push(data.include);
        if let Some(mut binary) = data.binary {
            let link = data.link.unwrap();
            libdirs.push(binary.swap_remove(0));
            links.extend(link);
        }
    }

    Dependencies{
        incdirs,
        headers,
        libdirs,
        links,
    }
}

pub fn get_project_kind(srcs: &[FileInfo]) -> Option<ProjKind> {
    for s in srcs {
        if s.file_name() == "main.cpp" {
            return Some(ProjKind::App)
        } else if s.file_name() == "lib.cpp" {
            return Some(ProjKind::Lib)
        }
    }
    None
}

