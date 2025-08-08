use super::Config;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BuildFile {
    pub project: String,
    pub cpp: String,
    pub dependencies: Vec<String>,

    #[serde(default = "src_default")]
    pub srcdir: String,
    #[serde(default)]
    pub incdirs: Vec<String>,
    #[serde(default)]
    pub defines: Vec<String>,
    #[serde(default)]
    pub pch: Option<String>,

    #[serde(default)]
    #[serde(rename = "compiler-options")]
    pub compiler_options: Vec<String>,
    #[serde(default)]
    #[serde(rename = "linker-options")]
    pub linker_options: Vec<String>,

    #[serde(default)]
    #[serde(rename = "include-public")]
    pub inc_public: Option<String>,
}

impl BuildFile {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    pub fn finalise(mut self, config: Config) -> Self {
        self.defines.push(config.as_arg());
        self.incdirs.push(self.srcdir.clone());
        self
    }
}

fn src_default() -> String {
    "src/".to_string()
}
