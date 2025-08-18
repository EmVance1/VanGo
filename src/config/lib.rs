use std::{
    collections::HashMap,
    path::PathBuf,
};
use serde::Deserialize;


#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct LibFile {
    pub library: Library,
    #[serde(default)]
    pub profile: HashMap<String, Profile>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct Library {
    pub package: String,
    pub version: String,
    pub lang: String,
    #[serde(rename="include-pub")]
    pub include_pub: PathBuf,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub include:  Option<Vec<PathBuf>>,
    pub libdirs:  Option<Vec<PathBuf>>,
    pub binaries: Option<Vec<PathBuf>>,
    pub macros:   Option<Vec<String>>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_libfile() {
        let file =
r#"
[library]
package = "SFML"
version = "3.0.1"
lang = "C++17"
include-pub = "include"

[profile.debug]
libdirs = [ "lib" ]
binaries = [ "sfml-network-d", "sfml-audio-d", "sfml-graphics-d", "sfml-window-d", "sfml-system-d" ]

[profile.release]
libdirs = [ "lib" ]
binaries = [ "sfml-network", "sfml-audio", "sfml-graphics", "sfml-window", "sfml-system" ]
"#;

        let mut profile: HashMap<String, Profile> = HashMap::new();
        profile.insert("debug".into(),
            Profile{ libdirs: Some(vec![ "lib".into() ]),
                binaries: Some(vec![ "sfml-network-d".into(), "sfml-audio-d".into(), "sfml-graphics-d".into(), "sfml-window-d".into(), "sfml-system-d".into() ]),
                ..Default::default()
            }
        );
        profile.insert("release".into(),
            Profile{ libdirs: Some(vec![ "lib".into() ]),
                binaries: Some(vec![ "sfml-network".into(), "sfml-audio".into(), "sfml-graphics".into(), "sfml-window".into(), "sfml-system".into() ]),
                ..Default::default()
            }
        );

        assert_eq!(toml::from_str::<LibFile>(file).unwrap(), LibFile{
            library: Library{
                package: "SFML".to_string(),
                version: "3.0.1".to_string(),
                lang: "C++17".to_string(),
                include_pub: "include".into(),
                ..Default::default()
            },
            profile,
        });
    }
}
