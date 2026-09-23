mod raw;

use super::{Artefact, Language, Profile, Toolchain, Version};
use crate::{config::Platform, error::Error};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFile {
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
    pub profiles: HashMap<String, BuildProfile>,
}

impl BuildFile {
    pub fn from_table(value: toml::Table) -> Result<Self, Error> {
        let mut file: raw::BuildFile = value.try_into()?;
        let mut profiles: HashMap<String, BuildProfile> = HashMap::new();
        let mut dependencies: Vec<(String, Dependency)> = Vec::new();

        if let Some(d) = file.profile.remove("debug") {
            profiles.insert("debug".to_string(), BuildProfile::debug(&file.package.defaults).merge(d).finish());
        } else {
            profiles.insert("debug".to_string(), BuildProfile::debug(&file.package.defaults).finish());
        }
        if let Some(r) = file.profile.remove("release") {
            profiles.insert(
                "release".to_string(),
                BuildProfile::release(&file.package.defaults).merge(r).finish(),
            );
        } else {
            profiles.insert("release".to_string(), BuildProfile::release(&file.package.defaults).finish());
        }
        for (k, p) in file.profile {
            let inherits = p.inherits.clone().ok_or(Error::InvalidCustomProfile(k.clone()))?;
            if inherits == "debug" {
                profiles.insert(k, BuildProfile::debug(&file.package.defaults).merge(p).finish());
            } else if inherits == "release" {
                profiles.insert(k, BuildProfile::release(&file.package.defaults).merge(p).finish());
            }
        }

        let lang = Language::from_str(&file.package.lang)?;
        let interface = if let Some(interface) = file.package.interface {
            Language::from_str(&interface)?
        } else {
            lang
        };

        let artefact = file.package.artefact.unwrap_or_default();
        let implib = (artefact == Artefact::SharedLib) && (Platform::current()? == Platform::Windows);

        for (k, v) in file.dependencies {
            dependencies.push((k, v.try_into()?));
        }

        Ok(BuildFile {
            name: file.package.name,
            version: Version::from_str(&file.package.version)?,
            lang,
            artefact,
            implib,
            toolchain: file.package.toolchain,
            interface,
            runtime: file.package.runtime,
            vcpkg: file.vcpkg.unwrap_or(VcpkgConfig {
                triplet: "x64-linux".to_string(),
            }),
            dependencies,
            profiles,
        })
    }

    pub fn get(&self, profile: &Profile) -> Result<&BuildProfile, Error> {
        match profile {
            Profile::Debug => self.profiles.get("debug"),
            Profile::Release => self.profiles.get("release"),
            Profile::Custom(s) => self.profiles.get(s),
        }
        .ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
    }

    pub fn take(&mut self, profile: &Profile) -> Result<BuildProfile, Error> {
        match profile {
            Profile::Debug => self.profiles.remove("debug"),
            Profile::Release => self.profiles.remove("release"),
            Profile::Custom(s) => self.profiles.remove(s),
        }
        .ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
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

impl BuildProfile {
    pub(super) fn debug(defaults: &raw::BuildProfile) -> Self {
        let mut defines = defaults.defines.clone();
        defines.push("VANGO_DEBUG".to_string());
        Self {
            baseprof: Profile::Debug,
            defines,
            include: defaults.include.clone(),
            pch: defaults.pch.clone(),

            settings: BuildSettings {
                opt_level: defaults.build_settings.opt_level.unwrap_or(0),
                opt_size: defaults.build_settings.opt_size.unwrap_or(false),
                opt_speed: defaults.build_settings.opt_speed.unwrap_or(false),
                opt_linktime: defaults.build_settings.opt_linktime.unwrap_or(false),
                iso_compliant: defaults.build_settings.iso_compliant.unwrap_or(false),
                warn_level: defaults.build_settings.warn_level.unwrap_or(WarnLevel::Basic),
                warn_as_error: defaults.build_settings.warn_as_error.unwrap_or(false),
                debug_info: defaults.build_settings.debug_info.unwrap_or(true),
                runtime: defaults.build_settings.runtime.unwrap_or(Runtime::DynamicDebug),
                aslr: defaults.build_settings.aslr.unwrap_or(true),
                no_rtti: defaults.build_settings.no_rtti.unwrap_or(false),
                no_except: defaults.build_settings.no_except.unwrap_or(false),

                pthreads: defaults.build_settings.pthreads.unwrap_or(false),
                asan: defaults.build_settings.sanitize.address.unwrap_or(false),
                tsan: defaults.build_settings.sanitize.thread.unwrap_or(false),
                lsan: defaults.build_settings.sanitize.leak.unwrap_or(false),
                ubsan: defaults.build_settings.sanitize.undefined.unwrap_or(false),
            },

            compiler_options: defaults.compiler_options.clone(),
            linker_options: defaults.linker_options.clone(),
        }
    }

    pub(super) fn release(defaults: &raw::BuildProfile) -> Self {
        let mut defines = defaults.defines.clone();
        defines.push("VANGO_RELEASE".to_string());
        Self {
            baseprof: Profile::Release,
            defines,
            include: defaults.include.clone(),
            pch: defaults.pch.clone(),

            settings: BuildSettings {
                opt_level: defaults.build_settings.opt_level.unwrap_or(3),
                opt_size: defaults.build_settings.opt_size.unwrap_or(false),
                opt_speed: defaults.build_settings.opt_speed.unwrap_or(false),
                opt_linktime: defaults.build_settings.opt_linktime.unwrap_or(true),
                iso_compliant: defaults.build_settings.iso_compliant.unwrap_or(false),
                warn_level: defaults.build_settings.warn_level.unwrap_or(WarnLevel::Basic),
                warn_as_error: defaults.build_settings.warn_as_error.unwrap_or(false),
                debug_info: defaults.build_settings.debug_info.unwrap_or(false),
                runtime: defaults.build_settings.runtime.unwrap_or(Runtime::DynamicRelease),
                aslr: defaults.build_settings.aslr.unwrap_or(true),
                no_rtti: defaults.build_settings.no_rtti.unwrap_or(false),
                no_except: defaults.build_settings.no_except.unwrap_or(false),

                pthreads: defaults.build_settings.pthreads.unwrap_or(false),
                asan: defaults.build_settings.sanitize.address.unwrap_or(false),
                tsan: defaults.build_settings.sanitize.thread.unwrap_or(false),
                lsan: defaults.build_settings.sanitize.leak.unwrap_or(false),
                ubsan: defaults.build_settings.sanitize.undefined.unwrap_or(false),
            },

            compiler_options: defaults.compiler_options.clone(),
            linker_options: defaults.linker_options.clone(),
        }
    }

    fn merge(mut self, other: raw::BuildProfile) -> Self {
        self.defines.extend(other.defines);
        self.include.extend(other.include);
        if let Some(pch) = other.pch {
            self.pch = Some(pch);
        }

        other.build_settings.opt_level.inspect(|s| self.settings.opt_level = *s);
        other.build_settings.opt_size.inspect(|s| self.settings.opt_size = *s);
        other.build_settings.opt_speed.inspect(|s| self.settings.opt_speed = *s);
        other.build_settings.opt_linktime.inspect(|s| self.settings.opt_linktime = *s);
        other.build_settings.iso_compliant.inspect(|s| self.settings.iso_compliant = *s);
        other.build_settings.warn_level.inspect(|s| self.settings.warn_level = *s);
        other.build_settings.warn_as_error.inspect(|s| self.settings.warn_as_error = *s);
        other.build_settings.debug_info.inspect(|s| self.settings.debug_info = *s);
        other.build_settings.runtime.inspect(|s| self.settings.runtime = *s);
        other.build_settings.aslr.inspect(|s| self.settings.aslr = *s);
        other.build_settings.no_rtti.inspect(|s| self.settings.no_rtti = *s);
        other.build_settings.no_except.inspect(|s| self.settings.no_except = *s);

        other.build_settings.pthreads.inspect(|s| self.settings.pthreads = *s);
        other.build_settings.sanitize.address.inspect(|s| self.settings.asan = *s);
        other.build_settings.sanitize.thread.inspect(|s| self.settings.tsan = *s);
        other.build_settings.sanitize.leak.inspect(|s| self.settings.lsan = *s);
        other.build_settings.sanitize.undefined.inspect(|s| self.settings.ubsan = *s);

        self.compiler_options.extend(other.compiler_options);
        self.linker_options.extend(other.linker_options);
        self
    }

    fn finish(mut self) -> Self {
        self.include.push("src".into());
        self
    }
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
