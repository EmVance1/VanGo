#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjKind {
    App,
    Lib,
}


fn src_def() -> String      {       "src/".to_string()   }
fn inc_def() -> Vec<String> { vec![ "src/".to_string() ] }
fn obj_def() -> String      {       "obj/".to_string()   }


#[derive(Debug, Deserialize)]
pub struct BuildDef {
    pub project: String,
    #[serde(alias = "cpp")]
    pub cppstd: String,

    #[serde(default = "src_def")]
    #[serde(alias = "srcdir")]
    pub src: String,
    #[serde(default = "inc_def")]
    #[serde(alias = "incdir")]
    pub inc: Vec<String>,
    #[serde(default = "obj_def")]
    #[serde(alias = "objdir")]
    pub obj: String,

    #[serde(default)]
    pub defines: Vec<String>,

    #[serde(default)]
    pub require: Vec<String>,

    pub deps: HashMap<String, LibDef>,

    #[serde(default)]
    pub pch: Option<String>,

    #[serde(default)]
    #[serde(rename = "cfg.debug")]
    pub debug_settings: HashMap<String, ConfigSettings>,
    #[serde(default)]
    #[serde(rename = "cfg.release")]
    pub release_settings: HashMap<String, ConfigSettings>,
}


#[derive(Debug, Deserialize)]
pub enum BinaryLibDir {
    Mono(String),
    Config(HashMap<String, String>),
}

#[derive(Debug, Deserialize)]
pub struct LibDef {
    pub include: String,
    #[serde(default)]
    pub binary: Option<Vec<String>>,
    #[serde(default)]
    pub link: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Dependencies {
    pub incdirs: Vec<String>,
    pub headers: Vec<crate::fetch::FileInfo>,
    pub libdirs: Vec<String>,
    pub links: Vec<String>,
}


#[derive(Debug, Deserialize)]
pub struct ConfigSettings {
    #[serde(default)]
    switches: Vec<String>,
    #[serde(default)]
    defines: Vec<String>,
    #[serde(default)]
    link: Vec<String>,
}

