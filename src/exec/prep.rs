use std::path::{Path, PathBuf};


pub fn ensure_out_dirs(sdir: &Path, odir: &Path) {
    let _ = std::fs::create_dir_all(odir);
    let _ = std::fs::create_dir(odir.join("pch"));
    ensure_out_dirs_rec(&PathBuf::from(sdir), sdir, &odir.join("obj"));
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


pub fn cull_zombies(sdir: &Path, odir: &Path, ext: &str) {
    cull_zombies_rec(&odir.join("obj"), sdir, &odir.join("obj"), ext);
}

fn cull_zombies_rec(root: &Path, sdir: &Path, odir: &Path, ext: &str) {
    for e in std::fs::read_dir(root).ok().unwrap() {
        let e = e.ok().unwrap();
        if e.path().is_file() && e.path().extension().unwrap() == "obj" {
            let src = sdir.join(e.path().file_stem().unwrap()).with_extension(ext);
            if !src.exists() {
                let _ = std::fs::remove_file(e.path());
            }
        } else if e.path().is_dir() {
            ensure_out_dirs_rec(&e.path(), sdir, odir);
        }
    }
}

