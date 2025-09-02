#![allow(dead_code)]
mod build;
mod lib;
mod settings;

pub use build::*;
pub use lib::*;
pub use settings::*;
use crate::error::Error;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VangoFile {
    Build(BuildFile),
    Lib(LibFile),
}

impl VangoFile {
    pub fn from_str(value: &str) -> Result<VangoFile, Error> {
        let table: toml::Table = toml::from_str(value)?;
        if table.contains_key("build") {
            Ok(VangoFile::Build(BuildFile::from_table(table)?))
        } else if table.contains_key("library") {
            Ok(VangoFile::Lib(LibFile::from_table(table)?))
        } else {
            Err(Error::Unknown)
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
[build]
package = "Shimmy"
version = "0.1.0"
lang = "C++20"
include = [ "src", "headers" ]

[dependencies]
engine  = { path="../engine" }
NavMesh = { path="../../NavMesh" }
SFML    = { git="https://github.com/SFML/SFML.git",     recipe="recipes/SFML.bat",  features=[ "graphics" ] }
LuaJIT  = { git="https://github.com/LuaJIT/LuaJIT.git", recipe="recipes/LuaJIT.bat" }

[profile.debug]
include = [ "dbg_headers" ]
"#;

        let mut dependencies: HashMap<String, Dependency> = HashMap::new();
        dependencies.insert("engine".into(),  Dependency::Local{ path: "../engine".into(),     features: vec![] });
        dependencies.insert("NavMesh".into(), Dependency::Local{ path: "../../NavMesh".into(), features: vec![] });
        dependencies.insert("SFML".into(),    Dependency::Git{
            git: "https://github.com/SFML/SFML.git".into(), tag: None, recipe: Some("recipes/SFML.bat".into()), features: vec![ "graphics".into() ]
        });
        dependencies.insert("LuaJIT".into(), Dependency::Git{
            git: "https://github.com/LuaJIT/LuaJIT.git".into(), tag: None, recipe: Some("recipes/LuaJIT.bat".into()), features: vec![]
        });

        let mut profile: HashMap<String, BuildProfile> = HashMap::new();
        profile.insert("debug".into(), BuildProfile{
            include: vec![ "src".into(), "src".into(), "headers".into(), "dbg_headers".into() ], // TODO: duplicates
            defines: vec![ "VANGO_DEBUG".into() ],
            ..Default::default()
        });
        profile.insert("release".into(), BuildProfile{
            include: vec![ "src".into(), "src".into(), "headers".into() ], // TODO: duplicates
            defines: vec![ "VANGO_RELEASE".into() ],
            ..Default::default()
        });

        assert_eq!(VangoFile::from_str(file).unwrap(), VangoFile::Build(BuildFile{
            build: Build{
                package: "Shimmy".to_string(),
                version: "0.1.0".to_string(),
                kind: ProjKind::App,
                lang: Lang::Cpp(120),
                interface: Lang::Cpp(120),
                runtime: None,
            },
            dependencies,
            profile,
        }));
    }

    #[test]
    fn parse_libfile() {
        let file =
r#"
[library]
package = "SFML"
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

        let mut profile: HashMap<String, LibProfile> = HashMap::new();
        profile.insert("debug".into(),
            LibProfile{
                include:  "include".into(),
                libdir:   "bin/debug".into(),
                binaries: vec![ "sfml-network-s".into(), "sfml-audio-s".into(), "sfml-graphics-s".into(), "sfml-window-s".into(), "sfml-system-s".into() ],
                defines:  vec![ "VANGO_DEBUG".into(), "SFML_STATIC".into() ],
            }
        );
        profile.insert("release".into(),
            LibProfile{
                include:  "include".into(),
                libdir:   "bin/release".into(),
                binaries: vec![ "sfml-network-s".into(), "sfml-audio-s".into(), "sfml-graphics-s".into(), "sfml-window-s".into(), "sfml-system-s".into() ],
                defines:  vec![ "VANGO_RELEASE".into(), "SFML_STATIC".into() ],
            }
        );

        assert_eq!(VangoFile::from_str(file).unwrap(), VangoFile::Lib(LibFile{
            library: Library{
                package: "SFML".to_string(),
                version: "3.0.1".to_string(),
                lang: Lang::from_str("C++17").unwrap(),
            },
            profile,
        }));
    }
}

