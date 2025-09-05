use std::{collections::HashMap, path::PathBuf, str::FromStr};
use serde::Deserialize;
use crate::{config::ProjKind, error::Error};
use super::{Lang, Profile};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFile {
    pub build: Build,
    pub dependencies: Vec<Dependency>,
    pub profile: HashMap<String, BuildProfile>,
}

impl BuildFile {
    pub fn from_table(value: toml::Table) -> Result<Self, Error> {
        let mut file: SerdeBuildFile = value.try_into()?;
        let mut profile: HashMap<String, BuildProfile> = HashMap::new();

        if let Some(d) = file.profile.remove("debug") {
            profile.insert("debug".to_string(), BuildProfile::debug(&file.build.defaults).merge(d));
        } else {
            profile.insert("debug".to_string(), BuildProfile::debug(&file.build.defaults));
        }
        if let Some(r) = file.profile.remove("release") {
            profile.insert("release".to_string(), BuildProfile::release(&file.build.defaults).merge(r));
        } else {
            profile.insert("release".to_string(), BuildProfile::release(&file.build.defaults));
        }
        for (k, p) in file.profile {
            let inherits = p.inherits.clone().ok_or(Error::InvalidCustomProfile(k.clone()))?;
            if inherits == "debug" {
                profile.insert(k, BuildProfile::debug(&file.build.defaults).merge(p));
            } else if inherits == "release" {
                profile.insert(k, BuildProfile::release(&file.build.defaults).merge(p));
            }
        }
        let lang = Lang::from_str(&file.build.lang)?;
        let mut kind = file.build.kind.map(|p| ProjKind::from_str(&p).unwrap()).unwrap_or_default();
        if let ProjKind::SharedLib{ implib } = &mut kind {
            *implib = file.build.implib.unwrap_or(true);
        }

        Ok(BuildFile{
            build: Build{
                package:    file.build.package,
                version:    file.build.version,
                lang,
                kind,
                interface:  file.build.interface.map(|l| Lang::from_str(&l).unwrap()).unwrap_or(lang),
                runtime:    file.build.runtime,
            },
            dependencies:   file.build.dependencies,
            profile,
        })
    }

    pub fn take(&mut self, profile: &Profile) -> Result<BuildProfile, Error> {
        match profile {
            Profile::Debug => self.profile.remove("debug"),
            Profile::Release => self.profile.remove("release"),
            Profile::Custom(s) => self.profile.remove(s),
        }.ok_or(Error::ProfileUnavailable(self.build.package.clone(), profile.to_string()))
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build {
    pub package: String,
    pub version: String,
    pub lang: Lang,
    pub kind: ProjKind,
    pub interface: Lang,
    pub runtime: Option<String>,
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
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WarnLevel { #[default] None, Basic, High }

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Runtime { #[default] DynamicDebug, DynamicRelease, StaticDebug, StaticRelease }

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BuildProfile {
    pub src: PathBuf,
    pub include: Vec<PathBuf>,
    pub include_pub: PathBuf,
    pub pch: Option<PathBuf>,
    pub defines: Vec<String>,

    pub opt_level:     u32,
    pub opt_size:      bool,
    pub opt_speed:     bool,
    pub opt_linktime:  bool,
    pub iso_compliant: bool,
    pub warn_level:    WarnLevel,
    pub warn_as_error: bool,
    pub debug_info:    bool,
    pub runtime:       Runtime,
    pub aslr:          bool,

    pub compiler_options: Vec<String>,
    pub linker_options: Vec<String>,
}

impl BuildProfile {
    fn debug(defaults: &SerdeBuildProfile) -> Self {
        let mut include = vec![ PathBuf::from("src") ];
        if let Some(inc) = &defaults.include {
            include.extend(inc.iter().map(|s| s.to_owned()));
        }
        let mut defines = vec![ "VANGO_DEBUG".to_string() ];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(|d| d.to_owned()));
        }
        Self{
            src: defaults.src.clone().unwrap_or("src".into()),
            include,
            include_pub: defaults.include_pub.clone().unwrap_or("src".into()),
            pch: defaults.pch.clone(),
            defines,

            opt_level:     defaults.build_settings.opt_level.unwrap_or(0),
            opt_size:      defaults.build_settings.opt_size.unwrap_or(false),
            opt_speed:     defaults.build_settings.opt_speed.unwrap_or(false),
            opt_linktime:  defaults.build_settings.opt_linktime.unwrap_or(false),
            iso_compliant: defaults.build_settings.iso_compliant.unwrap_or(false),
            warn_level:    defaults.build_settings.warn_level.unwrap_or(WarnLevel::Basic),
            warn_as_error: defaults.build_settings.warn_as_error.unwrap_or(false),
            debug_info:    defaults.build_settings.debug_info.unwrap_or(true),
            runtime:       defaults.build_settings.runtime.unwrap_or(Runtime::DynamicDebug),
            aslr:          defaults.build_settings.aslr.unwrap_or(true),

            compiler_options: defaults.compiler_options.iter().flatten().map(|o| o.to_string()).collect(),
            linker_options: defaults.linker_options.iter().flatten().map(|o| o.to_string()).collect(),
        }
    }

    fn release(defaults: &SerdeBuildProfile) -> Self {
        let mut include = vec![ PathBuf::from("src") ];
        if let Some(inc) = &defaults.include {
            include.extend(inc.iter().map(|s| s.to_owned()));
        }
        let mut defines = vec![ "VANGO_RELEASE".to_string() ];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(|d| d.to_owned()));
        }
        Self{
            src: defaults.src.clone().unwrap_or("src".into()),
            include,
            include_pub: defaults.include_pub.clone().unwrap_or("src".into()),
            pch: defaults.pch.clone(),
            defines,

            opt_level:     defaults.build_settings.opt_level.unwrap_or(3),
            opt_size:      defaults.build_settings.opt_size.unwrap_or(false),
            opt_speed:     defaults.build_settings.opt_speed.unwrap_or(false),
            opt_linktime:  defaults.build_settings.opt_linktime.unwrap_or(true),
            iso_compliant: defaults.build_settings.iso_compliant.unwrap_or(false),
            warn_level:    defaults.build_settings.warn_level.unwrap_or(WarnLevel::Basic),
            warn_as_error: defaults.build_settings.warn_as_error.unwrap_or(false),
            debug_info:    defaults.build_settings.debug_info.unwrap_or(false),
            runtime:       defaults.build_settings.runtime.unwrap_or(Runtime::DynamicRelease),
            aslr:          defaults.build_settings.aslr.unwrap_or(true),

            compiler_options: defaults.compiler_options.iter().flatten().map(|o| o.to_owned()).collect(),
            linker_options:   defaults.linker_options.iter().flatten().map(|o| o.to_owned()).collect(),
        }
    }

    fn merge(mut self, other: SerdeBuildProfile) -> Self {
        if let Some(src) = other.src { self.src = src; }
        self.include.extend(other.include.unwrap_or_default());
        if let Some(inc) = other.include_pub { self.include_pub = inc; }
        self.defines.extend(other.defines.unwrap_or_default());

        other.build_settings.opt_level.inspect(    |s| self.opt_level = *s);
        other.build_settings.opt_size.inspect(     |s| self.opt_size = *s);
        other.build_settings.opt_speed.inspect(    |s| self.opt_speed = *s);
        other.build_settings.opt_linktime.inspect( |s| self.opt_linktime = *s);
        other.build_settings.iso_compliant.inspect(|s| self.iso_compliant = *s);
        other.build_settings.warn_level.inspect(   |s| self.warn_level = *s);
        other.build_settings.warn_as_error.inspect(|s| self.warn_as_error = *s);
        other.build_settings.debug_info.inspect(   |s| self.debug_info = *s);
        other.build_settings.runtime.inspect(      |s| self.runtime = *s);
        other.build_settings.aslr.inspect(         |s| self.aslr = *s);

        self.compiler_options.extend(other.compiler_options.unwrap_or_default());
        self.linker_options.extend(other.linker_options.unwrap_or_default());
        self
    }
}

/*
impl Default for BuildProfile {
    fn default() -> Self {
        Self{
            src: "src".into(),
            include: vec![ "src".into() ],
            include_pub: "src".into(),
            pch: None,
            defines: Vec::new(),
            compiler_options: Vec::new(),
            linker_options: Vec::new(),
        }
    }
}
*/

#[derive(Debug, Clone, PartialEq, Eq)]
struct BuildSettings {
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeBuildFile {
    build: SerdeBuild,
    #[serde(default)]
    profile: HashMap<String, SerdeBuildProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeBuild {
    package: String,
    version: String,
    lang: String,
    kind: Option<String>,
    #[serde(alias = "import-lib")]
    implib: Option<bool>,
    interface: Option<String>,
    runtime: Option<String>,
    dependencies: Vec<Dependency>,

    #[serde(flatten)]
    defaults: SerdeBuildProfile,
}


#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
struct SerdeBuildProfile {
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
    aslr:          Option<bool>,
}

