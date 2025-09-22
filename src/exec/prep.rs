use std::path::Path;


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
            if e.path().to_string_lossy().ends_with(".h.obj") { continue }
            let src = sdir.join(e.path().strip_prefix(odir).unwrap()).with_extension(ext);
            if !src.exists() {
                let _ = std::fs::remove_file(e.path());
            }
        } else if e.path().is_dir() {
            cull_zombies_rec(&e.path(), sdir, odir, ext);
        }
    }
}

