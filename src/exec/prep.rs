use std::path::{Path, PathBuf};

pub fn assert_out_dirs(sdir: &str, odir: &str) {
    if !std::fs::exists("./bin/").unwrap() {
        std::fs::create_dir("./bin/").unwrap();
        std::fs::create_dir("./bin/debug/").unwrap();
        std::fs::create_dir("./bin/release/").unwrap();
    } else {
        if !std::fs::exists("./bin/debug").unwrap() {
            std::fs::create_dir("./bin/debug/").unwrap();
        }
        if !std::fs::exists("./bin/release").unwrap() {
            std::fs::create_dir("./bin/release/").unwrap();
        }
    }
    assert_out_dirs_rec(&PathBuf::from(sdir), sdir, odir);
}

pub fn assert_out_dirs_rec(root: &Path, sdir: &str, odir: &str) {
    let obj = root.to_string_lossy().replace(sdir, odir);
    if !std::fs::exists(&obj).unwrap() {
        std::fs::create_dir(obj).unwrap();
    }
    for e in std::fs::read_dir(root).ok().unwrap() {
        let e = e.ok().unwrap();
        if e.path().is_dir() {
            assert_out_dirs_rec(&e.path(), sdir, odir);
        }
    }
}
