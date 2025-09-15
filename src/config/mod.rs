mod build;
mod lib;
mod elems;

pub use build::*;
pub use lib::*;
pub use elems::*;
use crate::error::Error;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VangoFile {
    Build(BuildFile),
    Lib(LibFile),
}

#[allow(dead_code)]
impl VangoFile {
    pub fn from_str(value: &str) -> Result<VangoFile, Error> {
        let table: toml::Table = toml::from_str(value)?;
        if table.contains_key("package") {
            Ok(VangoFile::Build(BuildFile::from_table(table)?))
        } else if table.contains_key("staticlib") {
            Ok(VangoFile::Lib(LibFile::from_table(table)?))
        } else {
            Err(Error::InvalidPkgHeader(std::env::current_dir()?))
        }
    }

    pub fn get_build(self) -> Option<BuildFile> {
        if let Self::Build(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn get_lib(self) -> Option<LibFile> {
        if let Self::Lib(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn unwrap_build(self) -> BuildFile {
        self.get_build().unwrap()
    }

    pub fn unwrap_lib(self) -> LibFile {
        self.get_lib().unwrap()
    }
}



#[cfg(test)]
mod tests {
    use super::{build::*, lib::*, VangoFile, ProjKind, Lang};
    use std::{collections::HashMap, str::FromStr};

    #[test]
    fn parse_buildfile() {
        let file =
r#"
[package]
name = "Shimmy"
version = "0.1.0"
lang = "C++20"
include = [ "headers" ]

[profile.debug]
include = [ "dbg_headers" ]

[dependencies]
engine  = { path="../engine" }
NavMesh = { path="../../NavMesh" }
SFML    = { git="https://github.com/SFML/SFML.git",     recipe="recipes/SFML.bat",  features=[ "graphics" ] }
LuaJIT  = { git="https://github.com/LuaJIT/LuaJIT.git", recipe="recipes/LuaJIT.bat" }
"#;

        let mut dependencies = Vec::new();
        dependencies.push(Dependency::Local{ path: "../engine".into(),     features: vec![] });
        dependencies.push(Dependency::Local{ path: "../../NavMesh".into(), features: vec![] });
        dependencies.push(Dependency::Git{
            git: "https://github.com/SFML/SFML.git".into(), tag: None, recipe: Some("recipes/SFML.bat".into()), features: vec![ "graphics".into() ]
        });
        dependencies.push(Dependency::Git{
            git: "https://github.com/LuaJIT/LuaJIT.git".into(), tag: None, recipe: Some("recipes/LuaJIT.bat".into()), features: vec![]
        });

        let mut profiles: HashMap<String, BuildProfile> = HashMap::new();
        profiles.insert("debug".into(), BuildProfile{
            include: vec![ "headers".into(), "dbg_headers".into(), "src".into() ],
            defines: vec![ "VANGO_DEBUG".into() ],
            ..BuildProfile::debug(&Default::default())
        });
        profiles.insert("release".into(), BuildProfile{
            include: vec![ "headers".into(), "src".into()  ],
            defines: vec![ "VANGO_RELEASE".into() ],
            ..BuildProfile::release(&Default::default())
        });

        assert_eq!(VangoFile::from_str(file).unwrap(), VangoFile::Build(BuildFile{
            name: "Shimmy".to_string(),
            version: "0.1.0".to_string(),
            lang: Lang::Cpp(120),
            kind: ProjKind::App,
            interface: Lang::Cpp(120),
            runtime: None,
            dependencies,
            profiles,
        }));
    }

    #[test]
    fn parse_libfile() {
        let file =
r#"
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

        let mut profiles: HashMap<String, LibProfile> = HashMap::new();
        profiles.insert("debug".into(),
            LibProfile{
                include:  "include".into(),
                libdir:   "bin/debug".into(),
                binaries: vec![ "sfml-network-s".into(), "sfml-audio-s".into(), "sfml-graphics-s".into(), "sfml-window-s".into(), "sfml-system-s".into() ],
                defines:  vec![ "VANGO_DEBUG".into(), "SFML_STATIC".into() ],
            }
        );
        profiles.insert("release".into(),
            LibProfile{
                include:  "include".into(),
                libdir:   "bin/release".into(),
                binaries: vec![ "sfml-network-s".into(), "sfml-audio-s".into(), "sfml-graphics-s".into(), "sfml-window-s".into(), "sfml-system-s".into() ],
                defines:  vec![ "VANGO_RELEASE".into(), "SFML_STATIC".into() ],
            }
        );

        assert_eq!(VangoFile::from_str(file).unwrap(), VangoFile::Lib(LibFile{
            name:    "SFML".to_string(),
            version: "3.0.1".to_string(),
            lang:    Lang::from_str("C++17").unwrap(),
            profiles,
        }));
    }
}

