#![allow(dead_code)]
mod build;
mod lib;

use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum VangoFile {
    App(build::BuildFile),
    Lib(lib::LibFile),
}

