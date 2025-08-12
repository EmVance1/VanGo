use crate::{Config, error::Error};
use serde::Deserialize;
use std::collections::HashMap;
use super::{Lang, BuildFile};


#[derive(Debug, Clone, Deserialize)]
pub struct LibFile {
    pub library: String,
    pub lang: String,
    pub include: String,
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
    pub binary_debug: String,
    #[serde(rename = "binary.release")]
    pub binary_release: String,
    pub links: Vec<String>,
    #[serde(default)]
    pub defines: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibData {
    pub incdir: String,
    pub libdir: Option<String>,
    pub links: Vec<String>,
    pub defines: Vec<String>,
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
            return Ok(LibData{ incdir: self.include, libdir: None, links: vec![], defines: vec![] })
        };

        Ok(LibData {
            incdir: self.include,
            libdir: Some(if config.is_release() { cfg.binary_release } else { cfg.binary_debug }),
            links: cfg.links,
            defines: cfg.defines,
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
                binary_debug: "bin/debug/".to_string(),
                binary_release: "bin/release/".to_string(),
                links: vec![value.project],
                defines: value.defines,
            }),
            configs: HashMap::default(),
        }
    }
}

