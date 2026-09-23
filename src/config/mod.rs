mod build;
mod elems;
mod lib;

use crate::error::Error;
pub use build::*;
pub use elems::*;
pub use lib::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VangoFile {
    Build(PackageManifest),
    Lib(LibManifest),
}

#[allow(dead_code)]
impl VangoFile {
    pub fn from_str(value: &str) -> Result<VangoFile, Error> {
        let table: toml::Table = toml::from_str(value)?;
        if table.contains_key("package") {
            Ok(VangoFile::Build(PackageManifest::from_table(table)?))
        } else if table.contains_key("staticlib") {
            Ok(VangoFile::Lib(LibManifest::from_table(table)?))
        } else {
            Err(Error::InvalidPkgHeader(std::env::current_dir()?))
        }
    }

    pub fn get_build(self) -> Option<PackageManifest> {
        if let Self::Build(b) = self { Some(b) } else { None }
    }

    pub fn get_lib(self) -> Option<LibManifest> {
        if let Self::Lib(l) = self { Some(l) } else { None }
    }

    pub fn unwrap_build(self) -> PackageManifest {
        self.get_build().unwrap()
    }

    pub fn unwrap_lib(self) -> LibManifest {
        self.get_lib().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, str::FromStr};

    #[test]
    fn parse_buildfile() {
        let file = r#"
[package]
name = "Shimmy"
version = "0.1.0"
lang = "C++20"
include = [ "headers" ]

[profile.debug]
include = [ "dbg_headers" ]

[dependencies]
engine  = { src="../engine" }
NavMesh = { src="../../NavMesh" }
SFML    = { git="https://github.com/SFML/SFML.git",     recipe="recipes/SFML.bat",  features=[ "graphics" ] }
LuaJIT  = { git="https://github.com/LuaJIT/LuaJIT.git", recipe="recipes/LuaJIT.bat" }
"#;

        let mut dependencies = Vec::new();
        dependencies.push((
            "engine".into(),
            Dependency::Package {
                src: "../engine".into(),
                targets: vec![],
                features: vec![],
            },
        ));
        dependencies.push((
            "NavMesh".into(),
            Dependency::Package {
                src: "../../NavMesh".into(),
                targets: vec![],
                features: vec![],
            },
        ));
        dependencies.push((
            "SFML".into(),
            Dependency::Git {
                git: "https://github.com/SFML/SFML.git".into(),
                tag: None,
                features: vec!["graphics".into()],
            },
        ));
        dependencies.push((
            "LuaJIT".into(),
            Dependency::Git {
                git: "https://github.com/LuaJIT/LuaJIT.git".into(),
                tag: None,
                features: vec![],
            },
        ));

        let mut profiles = HashMap::new();
        profiles.insert(
            Profile::Debug,
            BuildProfile {
                include: vec!["src".into(), "headers".into(), "dbg_headers".into()],
                defines: vec!["VANGO_DEBUG".into()],
                ..BuildProfile::default_debug()
            },
        );
        profiles.insert(
            Profile::Release,
            BuildProfile {
                include: vec!["src".into(), "headers".into()],
                defines: vec!["VANGO_RELEASE".into()],
                ..BuildProfile::default_release()
            },
        );

        assert_eq!(
            VangoFile::from_str(file).unwrap(),
            VangoFile::Build(PackageManifest {
                name: "Shimmy".to_string(),
                version: "0.1.0".parse().unwrap(),
                lang: Language::Cpp(120),
                artefact: Artefact::Executable,
                implib: false,
                toolchain: None,
                interface: Language::Cpp(120),
                runtime: None,
                vcpkg: VcpkgConfig {
                    triplet: "x64-linux".to_string()
                },
                dependencies,
                profiles,
            })
        );
    }

    #[test]
    fn parse_libfile() {
        let file = r#"
[staticlib]
name = "SFML"
version = "3.0.1"
lang = "C++17"
include = "include"
defines = [ "SFML_STATIC" ]

[profile.debug]
libdir = "bin/debug"
binaries = [ "sfml-network-s", "sfml-audio-s", "sfml-graphics-s", "sfml-window-s", "sfml-system-s" ]

[profile.release]
libdir = "bin/release"
binaries = [ "sfml-network-s", "sfml-audio-s", "sfml-graphics-s", "sfml-window-s", "sfml-system-s" ]
"#;

        let mut profiles = HashMap::new();
        profiles.insert(
            Profile::Debug,
            LibProfile {
                include: "include".into(),
                libdir: "bin/debug".into(),
                binaries: vec![
                    "sfml-network-s".into(),
                    "sfml-audio-s".into(),
                    "sfml-graphics-s".into(),
                    "sfml-window-s".into(),
                    "sfml-system-s".into(),
                ],
                defines: vec!["VANGO_DEBUG".into(), "SFML_STATIC".into()],
            },
        );
        profiles.insert(
            Profile::Release,
            LibProfile {
                include: "include".into(),
                libdir: "bin/release".into(),
                binaries: vec![
                    "sfml-network-s".into(),
                    "sfml-audio-s".into(),
                    "sfml-graphics-s".into(),
                    "sfml-window-s".into(),
                    "sfml-system-s".into(),
                ],
                defines: vec!["VANGO_RELEASE".into(), "SFML_STATIC".into()],
            },
        );

        assert_eq!(
            VangoFile::from_str(file).unwrap(),
            VangoFile::Lib(LibManifest {
                name: "SFML".to_string(),
                version: "3.0.1".parse().unwrap(),
                lang: Language::from_str("C++17").unwrap(),
                profiles,
            })
        );
    }
}
