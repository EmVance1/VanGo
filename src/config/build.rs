use std::{collections::HashMap, path::PathBuf, str::FromStr};
use serde::Deserialize;
use crate::{config::ProjKind, error::Error};
use super::{Lang, Profile};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFile {
    pub build: Build,
    pub dependencies: HashMap<String, Dependency>,
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
            dependencies:   file.dependencies,
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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProfile {
    pub src: PathBuf,
    pub include: Vec<PathBuf>,
    pub include_pub: PathBuf,
    pub pch: Option<PathBuf>,
    pub defines: Vec<String>,
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
            compiler_options: defaults.compiler_options.iter().flatten().map(|o| o.to_owned()).collect(),
            linker_options:   defaults.linker_options.iter().flatten().map(|o| o.to_owned()).collect(),
        }
    }

    fn merge(mut self, other: SerdeBuildProfile) -> Self {
        if let Some(src) = other.src { self.src = src; }
        self.include.extend(other.include.unwrap_or_default());
        if let Some(inc) = other.include_pub { self.include_pub = inc; }
        self.defines.extend(other.defines.unwrap_or_default());
        self.compiler_options.extend(other.compiler_options.unwrap_or_default());
        self.linker_options.extend(other.linker_options.unwrap_or_default());
        self
    }
}


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


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeBuildFile {
    build: SerdeBuild,
    dependencies: HashMap<String, Dependency>,
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
    compiler_options: Option<Vec<String>>,
    linker_options: Option<Vec<String>>,
}

