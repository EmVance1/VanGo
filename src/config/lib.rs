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
    #[serde(default)]
    pub features: HashMap<String, Feature>,
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
    pub include:     Option<Vec<PathBuf>>,
    pub bin_debug:   Option<Vec<PathBuf>>,
    pub bin_release: Option<Vec<PathBuf>>,
    pub binaries:    Option<Vec<PathBuf>>,
    pub macros:      Option<Vec<String>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Feature {
    pub requires: Vec<String>,
    pub binaries: Option<Vec<PathBuf>>,
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
macros = [ "SFML_STATIC" ]

[profile.debug]
libdirs = [ "bin/debug" ]
binaries = [ "sfml-network-s", "sfml-audio-s", "sfml-graphics-s", "sfml-window-s", "sfml-system-s",
    "freetype", "FLAC", "vorbisenc", "vorbisfile", "vorbis", "ogg.lib",
    "opengl32", "gdi32", "ws2_32", "winmm" ],

[profile.release]
libdirs = [ "bin/release" ]
binaries = [ "sfml-network-s", "sfml-audio-s", "sfml-graphics-s", "sfml-window-s", "sfml-system-s",
    "freetype", "FLAC", "vorbisenc", "vorbisfile", "vorbis", "ogg.lib",
    "opengl32", "gdi32", "ws2_32", "winmm" ],
"#;

        /*
        let mut profile: HashMap<String, Profile> = HashMap::new();
        profile.insert("static".into(),
            Profile{ bin_debug: Some(vec![ "bin/debug".into() ]), bin_release: Some(vec![ "bin/release".into() ]),
                binaries: Some(vec![ "sfml-network-d".into(), "sfml-audio-d".into(), "sfml-graphics-d".into(), "sfml-window-d".into(), "sfml-system-d".into() ]),
                ..Default::default()
            }
        );
        profile.insert("dynamic".into(),
            Profile{ bin_debug: Some(vec![ "bin/debug".into() ]), bin_release: Some(vec![ "bin/release".into() ]),
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
            features: HashMap::new(),
        });
        */
    }
}
