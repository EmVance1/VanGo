use super::{Runtime, VcpkgConfig, WarnLevel};
use crate::config::{Artefact, Sanitizer, Toolchain};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BuildFile {
    pub package: Build,
    pub vcpkg: Option<VcpkgConfig>,
    pub dependencies: toml::Table,
    #[serde(default)]
    pub profile: HashMap<String, BuildProfile>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Build {
    pub name: String,
    pub version: String,
    pub lang: String,
    #[serde(alias = "type")]
    pub artefact: Option<Artefact>,
    pub toolchain: Option<Toolchain>,
    pub implib: Option<bool>,
    pub interface: Option<String>,
    pub runtime: Option<String>,

    #[serde(flatten)]
    pub defaults: BuildProfile,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct BuildProfile {
    pub inherits: Option<String>,

    pub defines: Vec<String>,
    pub include: Vec<PathBuf>,
    pub pch: Option<PathBuf>,

    #[serde(flatten)]
    pub build_settings: BuildSettings,

    pub compiler_options: Vec<String>,
    pub linker_options: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct BuildSettings {
    pub opt_level: Option<u32>,
    pub opt_size: Option<bool>,
    pub opt_speed: Option<bool>,
    pub opt_linktime: Option<bool>,
    pub iso_compliant: Option<bool>,
    pub warn_level: Option<WarnLevel>,
    pub warn_as_error: Option<bool>,
    pub debug_info: Option<bool>,
    pub runtime: Option<Runtime>,

    pub aslr: Option<bool>,
    pub no_rtti: Option<bool>,
    pub no_except: Option<bool>,

    pub pthreads: Option<bool>,
    pub sanitize: Sanitizer,
}
