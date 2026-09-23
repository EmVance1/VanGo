use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LibManifest {
    pub staticlib: Library,
    #[serde(default)]
    pub profile: HashMap<String, LibProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Library {
    pub name: String,
    pub version: String,
    pub lang: String,

    #[serde(flatten)]
    pub defaults: LibProfile,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct LibProfile {
    pub inherits: Option<String>,
    pub include: Option<PathBuf>,
    pub libdir: Option<PathBuf>,
    pub binaries: Vec<PathBuf>,
    pub defines: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[allow(dead_code)]
pub struct Feature {
    pub requires: Vec<String>,
    pub binaries: Vec<PathBuf>,
}

