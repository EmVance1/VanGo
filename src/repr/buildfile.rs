use std::path::PathBuf;
use super::Config;
use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildFile {
    pub project: String,
    pub lang: String,
    pub dependencies: Vec<String>,

    #[serde(default = "src_default")]
    pub srcdir: PathBuf,
    #[serde(default)]
    pub incdirs: Vec<PathBuf>,
    #[serde(default)]
    pub defines: Vec<String>,
    #[serde(default)]
    pub pch: Option<PathBuf>,

    #[serde(default)]
    pub include_public: Option<PathBuf>,

    #[serde(default)]
    pub compiler_options: Vec<String>,
    #[serde(default)]
    pub linker_options: Vec<String>,

}

impl BuildFile {
    pub fn finalise(mut self, config: Config) -> Self {
        self.defines.push(config.as_define().to_string());
        self.incdirs.push(self.srcdir.clone());
        self
    }
}

fn src_default() -> PathBuf {
    "src".into()
}

