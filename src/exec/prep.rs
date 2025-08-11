use std::path::{Path, PathBuf};


pub fn ensure_out_dirs(sdir: &str, odir: &str) {
    let _ = std::fs::create_dir_all("bin/debug/obj");
    let _ = std::fs::create_dir("bin/debug/pch");
    let _ = std::fs::create_dir_all("bin/release/obj");
    let _ = std::fs::create_dir("bin/release/pch");
    let objdir = format!("{odir}obj/");
    ensure_out_dirs_rec(&PathBuf::from(sdir), sdir, &objdir);
}

fn ensure_out_dirs_rec(root: &Path, sdir: &str, odir: &str) {
    let _ = std::fs::create_dir(root.to_string_lossy().replace(sdir, odir));
    for e in std::fs::read_dir(root).ok().unwrap() {
        let e = e.ok().unwrap();
        if e.path().is_dir() {
            ensure_out_dirs_rec(&e.path(), sdir, odir);
        }
    }
}

