mod build;
mod lib;

use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VangoFile {
    App(build::BuildFile),
    Lib(lib::LibFile),
}

