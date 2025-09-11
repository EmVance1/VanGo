use std::{collections::HashMap, path::PathBuf, str::FromStr};
use serde::Deserialize;
use crate::{config::ProjKind, error::Error};
use super::{Lang, Profile};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFile {
    pub name: String,
    pub version: String,
    pub lang: Lang,
    pub kind: ProjKind,
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
            profiles.insert("debug".to_string(), BuildProfile::debug(&file.package.defaults).merge(d));
        } else {
            profiles.insert("debug".to_string(), BuildProfile::debug(&file.package.defaults));
        }
        if let Some(r) = file.profile.remove("release") {
            profiles.insert("release".to_string(), BuildProfile::release(&file.package.defaults).merge(r));
        } else {
            profiles.insert("release".to_string(), BuildProfile::release(&file.package.defaults));
        }
        for (k, p) in file.profile {
            let inherits = p.inherits.clone().ok_or(Error::InvalidCustomProfile(k.clone()))?;
            if inherits == "debug" {
                profiles.insert(k, BuildProfile::debug(&file.package.defaults).merge(p));
            } else if inherits == "release" {
                profiles.insert(k, BuildProfile::release(&file.package.defaults).merge(p));
            }
        }
        let lang = Lang::from_str(&file.package.lang)?;
        let mut kind = file.package.kind.map(|p| ProjKind::from_str(&p).unwrap()).unwrap_or_default();
        if let ProjKind::SharedLib{ implib } = &mut kind {
            *implib = file.package.implib.unwrap_or(true);
        }

        for (_, v) in file.dependencies {
            dependencies.push(v.try_into()?);
        }

        Ok(BuildFile{
            name:      file.package.name,
            version:   file.package.version,
            lang,
            kind,
            interface: file.package.interface.map_or(lang, |l| Lang::from_str(&l).unwrap()),
            runtime:   file.package.runtime,
            dependencies,
            profiles,
        })
    }

    pub fn get(&self, profile: &Profile) -> Result<&BuildProfile, Error> {
        match profile {
            Profile::Debug => self.profiles.get("debug"),
            Profile::Release => self.profiles.get("release"),
            Profile::Custom(s) => self.profiles.get(s),
        }.ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
    }

    pub fn take(&mut self, profile: &Profile) -> Result<BuildProfile, Error> {
        match profile {
            Profile::Debug => self.profiles.remove("debug"),
            Profile::Release => self.profiles.remove("release"),
            Profile::Custom(s) => self.profiles.remove(s),
        }.ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Local{
        path: PathBuf,
        #[serde(default)]
        features: Vec<String>,
    },
    Headers{
        headers: PathBuf,
        #[serde(default)]
        features: Vec<String>,
    },
    Git{
        git: String,
        tag: Option<String>,
        recipe: Option<PathBuf>,
        #[serde(default)]
        features: Vec<String>,
    },
    System{ system: PathBuf, target: Option<String> },
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WarnLevel { None, Basic, High }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Runtime { DynamicDebug, DynamicRelease, StaticDebug, StaticRelease }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProfile {
    pub src: PathBuf,
    pub include: Vec<PathBuf>,
    pub include_pub: PathBuf,
    pub pch: Option<PathBuf>,
    pub defines: Vec<String>,
    pub settings: BuildSettings,

    pub compiler_options: Vec<String>,
    pub linker_options: Vec<String>,
}

impl BuildProfile {
    pub(super) fn debug(defaults: &SerdeBuildProfile) -> Self {
        let mut include = vec![ PathBuf::from("src") ];
        if let Some(inc) = &defaults.include {
            include.extend(inc.iter().map(PathBuf::to_owned));
        }
        let mut defines = vec![ "VANGO_DEBUG".to_string() ];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(String::to_owned));
        }
        Self{
            src: defaults.src.clone().unwrap_or("src".into()),
            include,
            include_pub: defaults.include_pub.clone().unwrap_or("src".into()),
            pch: defaults.pch.clone(),
            defines,
            settings: BuildSettings {
                opt_level:     defaults.build_settings.opt_level.unwrap_or(0),
                opt_size:      defaults.build_settings.opt_size.unwrap_or(false),
                opt_speed:     defaults.build_settings.opt_speed.unwrap_or(false),
                opt_linktime:  defaults.build_settings.opt_linktime.unwrap_or(false),
                iso_compliant: defaults.build_settings.iso_compliant.unwrap_or(false),
                warn_level:    defaults.build_settings.warn_level.unwrap_or(WarnLevel::Basic),
                warn_as_error: defaults.build_settings.warn_as_error.unwrap_or(false),
                debug_info:    defaults.build_settings.debug_info.unwrap_or(true),
                runtime:       defaults.build_settings.runtime.unwrap_or(Runtime::DynamicDebug),
                pthread:       defaults.build_settings.pthread.unwrap_or(false),
                aslr:          defaults.build_settings.aslr.unwrap_or(true),
                rtti:          defaults.build_settings.rtti.unwrap_or(true),
            },

            compiler_options: defaults.compiler_options.iter().flatten().map(String::to_owned).collect(),
            linker_options: defaults.linker_options.iter().flatten().map(String::to_owned).collect(),
        }
    }

    pub(super) fn release(defaults: &SerdeBuildProfile) -> Self {
        let mut include = vec![ PathBuf::from("src") ];
        if let Some(inc) = &defaults.include {
            include.extend(inc.iter().map(PathBuf::to_owned));
        }
        let mut defines = vec![ "VANGO_RELEASE".to_string() ];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(String::to_owned));
        }
        Self{
            src: defaults.src.clone().unwrap_or("src".into()),
            include,
            include_pub: defaults.include_pub.clone().unwrap_or("src".into()),
            pch: defaults.pch.clone(),
            defines,
            settings: BuildSettings {
                opt_level:     defaults.build_settings.opt_level.unwrap_or(3),
                opt_size:      defaults.build_settings.opt_size.unwrap_or(false),
                opt_speed:     defaults.build_settings.opt_speed.unwrap_or(false),
                opt_linktime:  defaults.build_settings.opt_linktime.unwrap_or(true),
                iso_compliant: defaults.build_settings.iso_compliant.unwrap_or(false),
                warn_level:    defaults.build_settings.warn_level.unwrap_or(WarnLevel::Basic),
                warn_as_error: defaults.build_settings.warn_as_error.unwrap_or(false),
                debug_info:    defaults.build_settings.debug_info.unwrap_or(false),
                runtime:       defaults.build_settings.runtime.unwrap_or(Runtime::DynamicRelease),
                pthread:       defaults.build_settings.pthread.unwrap_or(false),
                aslr:          defaults.build_settings.aslr.unwrap_or(true),
                rtti:          defaults.build_settings.rtti.unwrap_or(true),
            },

            compiler_options: defaults.compiler_options.iter().flatten().map(String::to_owned).collect(),
            linker_options:   defaults.linker_options.iter().flatten().map(String::to_owned).collect(),
        }
    }

    fn merge(mut self, other: SerdeBuildProfile) -> Self {
        if let Some(src) = other.src { self.src = src; }
        self.include.extend(other.include.unwrap_or_default());
        if let Some(inc) = other.include_pub { self.include_pub = inc; }
        self.defines.extend(other.defines.unwrap_or_default());
        other.build_settings.opt_level.inspect(    |s| self.settings.opt_level = *s);
        other.build_settings.opt_size.inspect(     |s| self.settings.opt_size = *s);
        other.build_settings.opt_speed.inspect(    |s| self.settings.opt_speed = *s);
        other.build_settings.opt_linktime.inspect( |s| self.settings.opt_linktime = *s);
        other.build_settings.iso_compliant.inspect(|s| self.settings.iso_compliant = *s);
        other.build_settings.warn_level.inspect(   |s| self.settings.warn_level = *s);
        other.build_settings.warn_as_error.inspect(|s| self.settings.warn_as_error = *s);
        other.build_settings.debug_info.inspect(   |s| self.settings.debug_info = *s);
        other.build_settings.runtime.inspect(      |s| self.settings.runtime = *s);
        other.build_settings.pthread.inspect(      |s| self.settings.pthread = *s);
        other.build_settings.aslr.inspect(         |s| self.settings.aslr = *s);
        other.build_settings.rtti.inspect(         |s| self.settings.rtti = *s);

        self.compiler_options.extend(other.compiler_options.unwrap_or_default());
        self.linker_options.extend(other.linker_options.unwrap_or_default());
        self
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildSettings {
    pub opt_level:     u32,
    pub opt_size:      bool,
    pub opt_speed:     bool,
    pub opt_linktime:  bool,
    pub iso_compliant: bool,
    pub warn_level:    WarnLevel,
    pub warn_as_error: bool,
    pub debug_info:    bool,
    pub runtime:       Runtime,
    pub pthread:       bool,
    pub aslr:          bool,
    pub rtti:          bool,
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
    src: Option<PathBuf>,
    include: Option<Vec<PathBuf>>,
    include_pub: Option<PathBuf>,
    pch: Option<PathBuf>,
    defines: Option<Vec<String>>,

    #[serde(flatten)]
    build_settings: SerdeBuildSettings,

    compiler_options: Option<Vec<String>>,
    linker_options: Option<Vec<String>>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SerdeBuildSettings {
    opt_level:     Option<u32>,
    opt_size:      Option<bool>,
    opt_speed:     Option<bool>,
    opt_linktime:  Option<bool>,
    iso_compliant: Option<bool>,
    warn_level:    Option<WarnLevel>,
    warn_as_error: Option<bool>,
    debug_info:    Option<bool>,
    runtime:       Option<Runtime>,
    pthread:       Option<bool>,
    aslr:          Option<bool>,
    rtti:          Option<bool>,
}

