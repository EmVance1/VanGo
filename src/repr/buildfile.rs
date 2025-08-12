use super::Config;
use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BuildFile {
    pub project: String,
    pub lang: String,
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
    pub include_public: Option<String>,

    #[serde(default)]
    pub compiler_options: Vec<String>,
    #[serde(default)]
    pub linker_options: Vec<String>,

}

impl BuildFile {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    pub fn finalise(mut self, config: Config) -> Self {
        self.defines.push(config.as_define().to_string());
        self.incdirs.push(self.srcdir.clone());
        self
    }
}

fn src_default() -> String {
    "src/".to_string()
}
