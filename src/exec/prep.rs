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

