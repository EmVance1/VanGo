mod profile;
mod raw;

use super::{Artefact, Language, Profile, Toolchain, Version};
use crate::{config::{Platform}, error::Error};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub name: String,
    pub version: Version,
    pub lang: Language,
    pub artefact: Artefact,
    pub implib: bool,
    pub toolchain: Option<Toolchain>,
    pub interface: Language,
    pub runtime: Option<String>,
    pub vcpkg: VcpkgConfig,
    pub dependencies: Vec<(String, Dependency)>,
    pub profiles: HashMap<Profile, BuildProfile>,
}

impl PackageManifest {
    pub fn from_table(value: toml::Table) -> Result<Self, Error> {
        let file: raw::PackageManifest = value.try_into()?;
        file.try_into()
    }

    pub fn get(&self, profile: &Profile) -> Result<&BuildProfile, Error> {
        self.profiles.get(profile).ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
    }

    pub fn remove(&mut self, profile: &Profile) -> Result<BuildProfile, Error> {
        self.profiles.remove(profile).ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
    }
}

impl TryFrom<raw::PackageManifest> for PackageManifest {
    type Error = Error;

    fn try_from(value: raw::PackageManifest) -> Result<Self, Self::Error> {
        let lang = Language::from_str(&value.package.lang)?;
        let interface = if let Some(interface) = value.package.interface {
            Language::from_str(&interface)?
        } else {
            lang
        };

        let artefact = value.package.artefact.unwrap_or_default();
        let implib = (artefact == Artefact::SharedLib) && (Platform::current()? == Platform::Windows);
        let mut profiles = HashMap::new();
        profiles.insert(Profile::Debug,   BuildProfile::default_release().layer(value.package.defaults.clone()));
        profiles.insert(Profile::Release, BuildProfile::default_release().layer(value.package.defaults.clone()));
        for (k, v) in value.profile {
            let base = v.inherits.as_ref().unwrap_or(&k);
            if base == "debug" {
                profiles.insert(Profile::from_str(&k)?, BuildProfile::default_debug().layer(value.package.defaults.clone()).layer(v));
            } else if base == "release" {
                profiles.insert(Profile::from_str(&k)?, BuildProfile::default_release().layer(value.package.defaults.clone()).layer(v));
            } else {
                return Err(Error::InvalidCustomProfile(k));
            };
        }

        let mut dependencies: Vec<(String, Dependency)> = Vec::new();
        for (k, v) in value.dependencies {
            dependencies.push((k, v.try_into()?));
        }

        Ok(PackageManifest {
            name: value.package.name,
            version: Version::from_str(&value.package.version)?,
            lang,
            artefact,
            implib,
            toolchain: value.package.toolchain,
            interface,
            runtime: value.package.runtime,
            vcpkg: value.vcpkg.unwrap_or(VcpkgConfig {
                triplet: "x64-linux".to_string(),
            }),
            dependencies,
            profiles,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Package {
        src: PathBuf,
        #[serde(default)]
        targets: Vec<PathBuf>,
        #[serde(default)]
        features: Vec<String>,
    },
    Headers {
        headers: PathBuf,
        #[serde(default)]
        features: Vec<String>,
    },
    Git {
        git: String,
        tag: Option<String>,
        #[serde(default)]
        features: Vec<String>,
    },
    System {
        system: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VcpkgConfig {
    pub triplet: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WarnLevel {
    None = 0,
    Basic = 1,
    High = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Runtime {
    DynamicDebug,
    DynamicRelease,
    StaticDebug,
    StaticRelease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProfile {
    pub baseprof: Profile,

    pub defines: Vec<String>,
    pub include: Vec<PathBuf>,
    pub pch: Option<PathBuf>,
    pub settings: BuildSettings,

    pub compiler_options: Vec<String>,
    pub linker_options: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct BuildSettings {
    pub opt_level: u32,
    pub opt_size: bool,
    pub opt_speed: bool,
    pub opt_linktime: bool,
    pub iso_compliant: bool,
    pub warn_level: WarnLevel,
    pub warn_as_error: bool,
    pub debug_info: bool,
    pub runtime: Runtime,
    pub aslr: bool,
    pub no_rtti: bool,
    pub no_except: bool,

    pub pthreads: bool,
    pub asan: bool,
    pub tsan: bool,
    pub lsan: bool,
    pub ubsan: bool,
}
