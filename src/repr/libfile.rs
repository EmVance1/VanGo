use std::{
    collections::HashMap,
    path::PathBuf,
};
use crate::{Config, error::Error};
use serde::Deserialize;
use super::{Lang, BuildFile};


#[derive(Debug, Clone, Deserialize)]
pub struct LibFile {
    pub library: String,
    pub lang:    String,
    pub include: PathBuf,
    #[serde(default)]
    pub all: Option<LibConfig>,
    #[serde(default)]
    pub configs: HashMap<String, LibConfig>,
}

impl LibFile {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibConfig {
    #[serde(rename = "binary.debug")]
    pub binary_debug: PathBuf,
    #[serde(rename = "binary.release")]
    pub binary_release: PathBuf,
    #[serde(alias = "libs")]
    #[serde(alias = "links")]
    pub archives: Vec<PathBuf>,
    #[serde(default)]
    pub defines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LibData {
    pub incdir:   PathBuf,
    pub libdir:   Option<PathBuf>,
    pub archives: Vec<PathBuf>,
    pub defines:  Vec<String>,
}

impl LibFile {
    pub fn validate(self, max_lang: Lang) -> Result<Self, Error> {
        if self.lang.parse::<Lang>()? > max_lang {
            Err(Error::IncompatibleCppStd(self.library))
        } else {
            Ok(self)
        }
    }

    pub fn linearise(mut self, config: Config, version: Option<&str>) -> Result<LibData, Error> {
        let cfg = if let Some(ver) = version {
            self.configs.remove(ver).ok_or(Error::ConfigUnavailable(self.library, ver.to_string()))?
        } else if let Some(all) = self.all {
            all
        } else {
            return Ok(LibData{ incdir: self.include, libdir: None, archives: vec![], defines: vec![] })
        };

        Ok(LibData{
            incdir:   self.include,
            libdir:   Some(if config.is_release() { cfg.binary_release } else { cfg.binary_debug }),
            archives: cfg.archives,
            defines:  cfg.defines,
        })
    }
}

impl From<BuildFile> for LibFile {
    fn from(value: BuildFile) -> Self {
        let include = value.include_public.unwrap_or(value.srcdir);

        Self {
            library: value.project.clone(),
            lang: value.lang,
            include,
            all: Some(LibConfig {
                binary_debug: "bin/debug".into(),
                binary_release: "bin/release/".into(),
                archives: vec![ PathBuf::from(value.project) ],
                defines: value.defines,
            }),
            configs: HashMap::default(),
        }
    }
}

