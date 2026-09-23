use crate::Error;
use std::path::{Path, PathBuf};

pub fn scan_dirs_for_filetype(dirs: &[PathBuf], extensions: &[String]) -> Result<Vec<PathBuf>, Error> {
    let mut res = Vec::new();
    for dir in dirs {
        res.extend(scan_for_filetype(dir, extensions)?);
    }
    Ok(res)
}

pub fn scan_for_filetype(dir: &Path, extensions: &[String]) -> Result<Vec<PathBuf>, Error> {
    let mut res = Vec::new();
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        if e.path().is_dir() {
            res.extend(scan_for_filetype(&e.path(), extensions)?);
        } else if e.path().is_file() {
            let ext = e.path().extension().unwrap_or_default().to_owned();
            for alt in extensions {
                if ext == alt.as_str() {
                    res.push(e.path());
                }
            }
        }
    }
    Ok(res)
}

pub fn ensure_out_dirs(sdir: &Path, odir: &Path) {
    let _ = std::fs::create_dir_all(odir);
    ensure_out_dirs_rec(sdir, sdir, &odir.join("obj"));
}

fn ensure_out_dirs_rec(root: &Path, sdir: &Path, odir: &Path) {
    let _ = std::fs::create_dir(odir.join(root.strip_prefix(sdir).unwrap()));
    for e in std::fs::read_dir(root).ok().unwrap() {
        let e = e.ok().unwrap();
        if e.path().is_dir() {
            ensure_out_dirs_rec(&e.path(), sdir, odir);
        }
    }
}

#[allow(dead_code)]
pub fn cull_zombies(sdir: &Path, odir: &Path, ext: &str) {
    cull_zombies_rec(&odir.join("obj"), sdir, &odir.join("obj"), ext);
}

#[allow(dead_code)]
fn cull_zombies_rec(root: &Path, sdir: &Path, odir: &Path, ext: &str) {
    for e in std::fs::read_dir(root).ok().unwrap() {
        let e = e.ok().unwrap();
        if e.path().is_file() && e.path().extension().unwrap() == "obj" {
            if e.path().to_string_lossy().ends_with(".h.obj") {
                continue;
            }
            let src = sdir.join(e.path().strip_prefix(odir).unwrap()).with_extension(ext);
            if !src.exists() {
                let _ = std::fs::remove_file(e.path());
            }
        } else if e.path().is_dir() {
            cull_zombies_rec(&e.path(), sdir, odir, ext);
        }
    }
}
