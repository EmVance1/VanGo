use crate::log_info_ln;
use std::path::{Path, PathBuf};

pub fn pull_package(url: &Path, tag: &Option<String>, install_loc: &Path) {
    let branch: Vec<PathBuf> = if let Some(tag) = tag {
        vec!["--branch".into(), tag.into(), "--depth".into(), "1".into(), url.into()]
    } else {
        vec![url.into()]
    };
    log_info_ln!("{:-<80}", format!("cloning project dependency to: {} ", install_loc.display()));
    std::process::Command::new("git")
        .arg("clone")
        .args(branch)
        .arg(install_loc)
        .output()
        .unwrap();
}
