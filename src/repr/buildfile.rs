use std::{collections::HashMap, path::PathBuf, str::FromStr};
use serde::Deserialize;
use crate::{error::Error, repr::{self, Lang}};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFile {
    pub project: String,
    pub lang:    Lang,
    pub dependencies: Vec<String>,
    pub profile: HashMap<String, BuildProfile>,
}

impl BuildFile {
    pub fn from_str(value: &str) -> Result<Self, Error> {
        let mut file = serde_json::from_str::<SerdeBuildFile>(value)?;
        let mut profile: HashMap<String, BuildProfile> = HashMap::new();

        if let Some(d) = file.profile.remove("debug") {
            profile.insert("debug".to_string(), BuildProfile::debug(&file.defaults).merge(d));
        } else {
            profile.insert("debug".to_string(), BuildProfile::debug(&file.defaults));
        }
        if let Some(r) = file.profile.remove("release") {
            profile.insert("release".to_string(), BuildProfile::release(&file.defaults).merge(r));
        } else {
            profile.insert("release".to_string(), BuildProfile::release(&file.defaults));
        }
        for (k, p) in file.profile {
            let inherits = p.inherits.clone().ok_or(Error::InvalidCustomProfile(k.clone()))?;
            if inherits == "debug" {
                profile.insert(k, BuildProfile::debug(&file.defaults).merge(p));
            } else if inherits == "release" {
                profile.insert(k, BuildProfile::release(&file.defaults).merge(p));
            }
        }

        Ok(BuildFile{
            project: file.project,
            lang: Lang::from_str(&file.lang)?,
            dependencies: file.dependencies,
            profile,
        })
    }

    pub fn take(&mut self, profile: &repr::Profile) -> Result<BuildProfile, Error> {
        match profile {
            repr::Profile::Debug => self.profile.remove("debug"),
            repr::Profile::Release => self.profile.remove("release"),
            repr::Profile::Custom(s) => self.profile.remove(s),
        }.ok_or(Error::ProfileUnavailable(self.project.clone(), profile.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeBuildFile {
    pub project: String,
    pub lang: String,
    pub dependencies: Vec<String>,

    #[serde(flatten)]
    pub defaults: SerdeProfile,
    #[serde(default)]
    pub profile: HashMap<String, SerdeProfile>,
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
    fn debug(defaults: &SerdeProfile) -> Self {
        let mut include = vec![ PathBuf::from("src") ];
        if let Some(inc) = &defaults.include {
            include.extend(inc.iter().map(|s| s.to_owned()));
        }
        let mut defines = vec![ "VANGO_DEBUG".to_string() ];
        if let Some(inc) = &defaults.defines {
            defines.extend(inc.iter().map(|s| s.to_owned()));
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

    fn release(defaults: &SerdeProfile) -> Self {
        let mut include = vec![ PathBuf::from("src") ];
        if let Some(inc) = &defaults.include {
            include.extend(inc.iter().map(|s| s.to_owned()));
        }
        let mut defines = vec![ "VANGO_RELEASE".to_string() ];
        if let Some(inc) = &defaults.defines {
            defines.extend(inc.iter().map(|s| s.to_owned()));
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

    fn merge(mut self, other: SerdeProfile) -> Self {
        if let Some(src) = other.src { self.src = src; }
        self.include.extend(other.include.unwrap_or_default());
        if let Some(inc) = other.include_pub { self.include_pub = inc; }
        self.defines.extend(other.defines.unwrap_or_default());
        self.compiler_options.extend(other.compiler_options.unwrap_or_default());
        self.linker_options.extend(other.linker_options.unwrap_or_default());
        self
    }
}


#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
struct SerdeProfile {
    pub inherits: Option<String>,
    pub src: Option<PathBuf>,
    pub include: Option<Vec<PathBuf>>,
    pub include_pub: Option<PathBuf>,
    pub pch: Option<PathBuf>,
    pub defines: Option<Vec<String>>,
    pub compiler_options: Option<Vec<String>>,
    pub linker_options: Option<Vec<String>>,
}

