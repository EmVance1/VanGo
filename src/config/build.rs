use super::{Lang, Profile, ProjKind, ToolChain, Version};
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFile {
    pub name: String,
    pub version: Version,
    pub lang: Lang,
    pub kind: ProjKind,
    pub toolchain: Option<ToolChain>,
    pub interface: Lang,
    pub runtime: Option<String>,
    pub dependencies: Vec<Dependency>,
    pub profiles: HashMap<String, BuildProfile>,
}

impl BuildFile {
    pub fn from_table(value: toml::Table) -> Result<Self, Error> {
        let mut file: SerdeBuildFile = value.try_into()?;
        let mut profiles: HashMap<String, BuildProfile> = HashMap::new();
        let mut dependencies: Vec<Dependency> = Vec::new();

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
        let lang = Lang::from_str(&file.package.lang)?;
        let mut kind = ProjKind::from_str(&file.package.kind.unwrap_or("app".to_string()))?;
        if let ProjKind::SharedLib { implib } = &mut kind {
            *implib = file.package.implib.unwrap_or(true);
        }
        let interface = if let Some(interface) = file.package.interface {
            Lang::from_str(&interface)?
        } else {
            lang
        };

        let toolchain = if let Some(tc) = file.package.toolchain {
            Some(ToolChain::from_str(&tc)?)
        } else {
            None
        };

        for (_, v) in file.dependencies {
            dependencies.push(v.try_into()?);
        }

        Ok(BuildFile {
            name: file.package.name,
            version: Version::from_str(&file.package.version)?,
            lang,
            kind,
            toolchain,
            interface,
            runtime: file.package.runtime,
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
    Local {
        path: PathBuf,
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
        recipe: Option<PathBuf>,
        #[serde(default)]
        features: Vec<String>,
    },
    System {
        system: PathBuf,
        target: Option<String>,
    },
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
    pub(super) fn debug(defaults: &SerdeBuildProfile) -> Self {
        let mut defines = vec!["VANGO_DEBUG".to_string()];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(String::to_owned));
        }
        Self {
            baseprof: Profile::Debug,
            defines,
            include: defaults.include.iter().flatten().map(PathBuf::to_owned).collect(),
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
                pthreads: defaults.build_settings.pthreads.unwrap_or(false),
                aslr: defaults.build_settings.aslr.unwrap_or(true),
                no_rtti: defaults.build_settings.no_rtti.unwrap_or(false),
                no_except: defaults.build_settings.no_except.unwrap_or(false),
            },

            compiler_options: defaults.compiler_options.iter().flatten().map(String::to_owned).collect(),
            linker_options: defaults.linker_options.iter().flatten().map(String::to_owned).collect(),
        }
    }

    pub(super) fn release(defaults: &SerdeBuildProfile) -> Self {
        let mut defines = vec!["VANGO_RELEASE".to_string()];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(String::to_owned));
        }
        Self {
            baseprof: Profile::Release,
            defines,
            include: defaults.include.iter().flatten().map(PathBuf::to_owned).collect(),
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
                pthreads: defaults.build_settings.pthreads.unwrap_or(false),
                aslr: defaults.build_settings.aslr.unwrap_or(true),
                no_rtti: defaults.build_settings.no_rtti.unwrap_or(false),
                no_except: defaults.build_settings.no_except.unwrap_or(false),
            },

            compiler_options: defaults.compiler_options.iter().flatten().map(String::to_owned).collect(),
            linker_options: defaults.linker_options.iter().flatten().map(String::to_owned).collect(),
        }
    }

    fn merge(mut self, other: SerdeBuildProfile) -> Self {
        self.defines.extend(other.defines.unwrap_or_default());
        self.include.extend(other.include.unwrap_or_default());
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
        other.build_settings.pthreads.inspect(|s| self.settings.pthreads = *s);
        other.build_settings.aslr.inspect(|s| self.settings.aslr = *s);
        other.build_settings.no_rtti.inspect(|s| self.settings.no_rtti = *s);
        other.build_settings.no_except.inspect(|s| self.settings.no_except = *s);

        self.compiler_options.extend(other.compiler_options.unwrap_or_default());
        self.linker_options.extend(other.linker_options.unwrap_or_default());
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
    pub pthreads: bool,
    pub aslr: bool,
    pub no_rtti: bool,
    pub no_except: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SerdeBuildFile {
    package: SerdeBuild,
    dependencies: toml::Table,
    #[serde(default)]
    profile: HashMap<String, SerdeBuildProfile>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SerdeBuild {
    name: String,
    version: String,
    lang: String,
    kind: Option<String>,
    toolchain: Option<String>,
    implib: Option<bool>,
    interface: Option<String>,
    runtime: Option<String>,

    #[serde(flatten)]
    defaults: SerdeBuildProfile,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub(super) struct SerdeBuildProfile {
    inherits: Option<String>,

    defines: Option<Vec<String>>,
    include: Option<Vec<PathBuf>>,
    pch: Option<PathBuf>,

    #[serde(flatten)]
    build_settings: SerdeBuildSettings,

    compiler_options: Option<Vec<String>>,
    linker_options: Option<Vec<String>>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SerdeBuildSettings {
    opt_level: Option<u32>,
    opt_size: Option<bool>,
    opt_speed: Option<bool>,
    opt_linktime: Option<bool>,
    iso_compliant: Option<bool>,
    warn_level: Option<WarnLevel>,
    warn_as_error: Option<bool>,
    debug_info: Option<bool>,
    runtime: Option<Runtime>,
    pthreads: Option<bool>,
    aslr: Option<bool>,
    no_rtti: Option<bool>,
    no_except: Option<bool>,
}
