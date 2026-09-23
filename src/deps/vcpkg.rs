use super::Dependencies;
use crate::log_info_ln;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct VcpkgDependency {
    pub name: String,
    pub features: Vec<String>,
}

pub fn pull_package(packages: Vec<VcpkgDependency>, triplet: &str, deps: &mut Dependencies) {
    if packages.is_empty() {
        return;
    }
    let _ = std::fs::create_dir("bin");
    std::env::set_current_dir("bin").unwrap();
    let mut data = HashMap::new();
    data.insert("dependencies".to_string(), packages);
    std::fs::write("vcpkg.json", serde_json::to_string_pretty(&data).unwrap()).unwrap();

    log_info_ln!("{:-<80}", "pulling vcpkg dependencies");
    // std::process::Command::new("vcpkg")
    //     .arg("install")
    //     .arg("--triplet")
    //     .arg(triplet)
    //     .output()
    //     .unwrap();

    std::env::set_current_dir("..").unwrap();

    deps.incdirs.push(format!("bin/vcpkg_installed/{}/include", triplet).into());
    deps.libdirs.push(format!("bin/vcpkg_installed/{}/lib", triplet).into());
    deps.rpaths.push(format!("bin/vcpkg_installed/{}/lib", triplet).into());
}
