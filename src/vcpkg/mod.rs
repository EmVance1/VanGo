use crate::deps::Dependencies;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct VcpkgDependency {
    pub name: String,
    pub features: Vec<String>,
}

pub fn pull_packages(packages: Vec<VcpkgDependency>, triplet: &str, deps: &mut Dependencies, debug: bool) {
    if packages.is_empty() {
        return;
    }
    let _ = std::fs::create_dir("bin");
    std::env::set_current_dir("bin").unwrap();
    let mut data = HashMap::new();
    data.insert("dependencies".to_string(), packages);
    std::fs::write("vcpkg.json", serde_json::to_string_pretty(&data).unwrap()).unwrap();

    // log_info_ln!("{:-<80}", "pulling vcpkg dependencies");
    // std::process::Command::new("vcpkg")
    //     .arg("install")
    //     .arg("--triplet")
    //     .arg(triplet)
    //     .output()
    //     .unwrap();

    std::env::set_current_dir("..").unwrap();

    deps.incdirs.push(format!("bin/vcpkg_installed/{}/include", triplet).into());
    if debug {
        deps.libdirs.push(format!("bin/vcpkg_installed/{}/debug/lib", triplet).into());
        deps.rpaths.push(format!("bin/vcpkg_installed/{}/debug/lib", triplet).into());
    } else {
        deps.libdirs.push(format!("bin/vcpkg_installed/{}/lib", triplet).into());
        deps.rpaths.push(format!("bin/vcpkg_installed/{}/lib", triplet).into());
    }
}

pub fn run_pkgconfig(packages: Vec<&PathBuf>, debug: bool) -> Vec<String> {
    if packages.is_empty() {
        return vec![];
    }

    let pkg_config_path = PathBuf::from("bin/vcpkg_installed/x64-linux/")
        .join(if debug { "debug/lib/pkgconfig" } else { "lib/pkgconfig" })
        .canonicalize()
        .unwrap();

    let output = std::process::Command::new("pkg-config")
        .arg("--libs-only-l")
        .args(packages)
        .env("PKG_CONFIG_PATH", pkg_config_path)
        .output()
        .unwrap();

    String::from_utf8_lossy(&output.stdout)
        .split(' ')
        .map(|l| l.trim().strip_prefix("-l").unwrap_or(l).to_owned())
        .collect()
}
