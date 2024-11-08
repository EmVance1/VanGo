use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use crate::fetch::FileInfo;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjKind {
    App,
    Lib,
}

impl ProjKind {
    pub fn ext(&self) -> String {
        match self {
            Self::App => "exe".to_string(),
            Self::Lib => "lib".to_string(),
        }
    }
}


fn src_def() -> String      {       "src/".to_string()   }
fn inc_def() -> Vec<String> { vec![ "src/".to_string() ] }


#[derive(Debug, Clone, Deserialize)]
pub struct BuildDef {
    pub project: String,
    #[serde(alias = "cpp")]
    pub cppstd: String,
    #[serde(default = "src_def")]
    pub src_dir: String,
    #[serde(default = "inc_def")]
    pub inc_dirs: Vec<String>,
    #[serde(default)]
    pub defines: Vec<String>,
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub pch: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    Debug,
    Release,
}

impl Config {
    // pub fn is_debug  (&self) -> bool { *self == Config::Debug }
    pub fn is_release(&self) -> bool { *self == Config::Release }

    pub fn as_arg(&self) -> String {
        match self {
            Self::Debug   => "DEBUG".to_string(),
            Self::Release => "RELEASE".to_string(),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug   => write!(f, "debug"),
            Self::Release => write!(f, "release"),
        }
    }
}



#[derive(Debug, Clone, Deserialize)]
pub struct LibDef {
    pub library: String,
    pub minstd: String,
    pub include: String,
    #[serde(default)]
    pub all: Option<LibConfig>,
    pub configs: HashMap<String, LibConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibConfig {
    #[serde(default)]
    pub defines: Vec<String>,
    #[serde(rename = "binary.debug")]
    pub binary_debug: String,
    #[serde(rename = "binary.release")]
    pub binary_release: String,
    pub links: Vec<String>
}


#[derive(Debug, Clone)]
pub struct Dependencies {
    pub incdirs: Vec<String>,
    pub headers: Vec<FileInfo>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
    pub defines: Vec<String>,
}

