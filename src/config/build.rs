use std::{
    collections::HashMap,
    path::PathBuf,
};
use serde::Deserialize;


#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct BuildFile {
    pub build: Build,
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub profile: HashMap<String, Profile>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Build {
    pub package: String,
    pub version: String,
    pub lang: String,

    pub pch: Option<PathBuf>,

    #[serde(flatten)]
    pub profile: MainProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Local{
        path: PathBuf,
        #[serde(default)]
        features: Vec<String>,
    },
    Headers{
        headers: PathBuf,
        #[serde(default)]
        features: Vec<String>,
    },
    Git{
        git: String,
        tag: Option<String>,
        recipe: Option<PathBuf>,
        #[serde(default)]
        features: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct MainProfile {
    pub src: PathBuf,
    pub include: Vec<PathBuf>,
    pub include_pub: PathBuf,
    pub macros: Vec<String>,
    pub compiler_options: Vec<String>,
    pub linker_options: Vec<String>,
}

impl Default for MainProfile {
    fn default() -> Self {
        MainProfile {
            src: "src".into(),
            include: vec![ "src".into() ],
            include_pub: "src".into(),
            macros: Vec::new(),
            compiler_options: Vec::new(),
            linker_options: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub src: Option<PathBuf>,
    pub include: Option<Vec<PathBuf>>,
    pub include_pub: Option<PathBuf>,
    pub macros: Option<Vec<String>>,
    pub compiler_options: Option<Vec<String>>,
    pub linker_options: Option<Vec<String>>,
}


#[cfg(test)]
mod tests {
    use super::*;

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
SFUtil  = { path="../../SFUtil" }
NavMesh = { path="../../NavMesh" }
SFML    = { git="https://github.com/SFML/SFML.git",     recipe="recipes/SFML.bat",  features=[ "static" ] }
LuaJIT  = { git="https://github.com/LuaJIT/LuaJIT.git", recipe="recipes/LuaJIT.bat" }

[profile.debug]
include = [ "src", "headers", "dbg_headers" ]
"#;

        let mut dependencies: HashMap<String, Dependency> = HashMap::new();
        dependencies.insert("engine".into(),  Dependency::Local{ path: "../engine".into(), features: vec![] });
        dependencies.insert("SFUtil".into(),  Dependency::Local{ path: "../../SFUtil".into(), features: vec![] });
        dependencies.insert("NavMesh".into(), Dependency::Local{ path: "../../NavMesh".into(), features: vec![] });
        dependencies.insert("SFML".into(),
            Dependency::Git{ git: "https://github.com/SFML/SFML.git".into(),     tag: None, recipe: Some("recipes/SFML.bat".into()),
                features: vec![ "static".into() ] }
        );
        dependencies.insert("LuaJIT".into(),
            Dependency::Git{ git: "https://github.com/LuaJIT/LuaJIT.git".into(), tag: None, recipe: Some("recipes/LuaJIT.bat".into()), features: vec![] }
        );

        let mut profile: HashMap<String, Profile> = HashMap::new();
        profile.insert("debug".into(),  Profile{ include: Some(vec![ "src".into(), "headers".into(), "dbg_headers".into() ]), ..Default::default() });

        assert_eq!(toml::from_str::<BuildFile>(file).unwrap(), BuildFile{
            build: Build{
                package: "Shimmy".to_string(),
                version: "0.1.0".to_string(),
                lang: "C++20".to_string(),
                profile: MainProfile { include: vec![ "src".into(), "headers".into() ], ..Default::default() },
                ..Default::default()
            },
            dependencies,
            profile,
        });
    }
}
