use std::{collections::HashMap, path::PathBuf, str::FromStr};
use serde::Deserialize;
use crate::error::Error;
use super::{build::BuildFile, Profile, Lang};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibFile {
    pub name:     String,
    pub version:  String,
    pub lang:     Lang,
    pub profiles: HashMap<String, LibProfile>,
}

impl LibFile {
    pub fn from_table(value: toml::Table) -> Result<LibFile, Error> {
        let mut file: SerdeLibFile = value.try_into()?;
        let mut profiles: HashMap<String, LibProfile> = HashMap::new();

        if let Some(d) = file.profile.remove("debug") {
            profiles.insert("debug".to_string(), LibProfile::debug(&file.staticlib.defaults).merge(d));
        } else {
            profiles.insert("debug".to_string(), LibProfile::debug(&file.staticlib.defaults));
        }
        if let Some(r) = file.profile.remove("release") {
            profiles.insert("release".to_string(), LibProfile::release(&file.staticlib.defaults).merge(r));
        } else {
            profiles.insert("release".to_string(), LibProfile::release(&file.staticlib.defaults));
        }
        for (k, p) in file.profile {
            let inherits = p.inherits.clone().ok_or(Error::InvalidCustomProfile(k.clone()))?;
            if inherits == "debug" {
                profiles.insert(k, LibProfile::debug(&file.staticlib.defaults).merge(p));
            } else if inherits == "release" {
                profiles.insert(k, LibProfile::release(&file.staticlib.defaults).merge(p));
            }
        }

        Ok(LibFile{
            name:    file.staticlib.name,
            version: file.staticlib.version,
            lang:    Lang::from_str(&file.staticlib.lang)?,
            profiles,
        })
    }

    pub fn take(&mut self, profile: &Profile) -> Result<LibProfile, Error> {
        match profile {
            Profile::Debug => self.profiles.remove("debug"),
            Profile::Release => self.profiles.remove("release"),
            Profile::Custom(s) => self.profiles.remove(s),
        }.ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
    }

    pub fn validate(self, max_lang: Lang) -> Result<Self, Error> {
        if self.lang > max_lang {
            Err(Error::IncompatibleCppStd(self.name))
        } else {
            Ok(self)
        }
    }
}

impl TryFrom<BuildFile> for LibFile {
    type Error = Error;

    fn try_from(value: BuildFile) -> Result<Self, Self::Error> {
        let name = value.name;
        if !value.kind.is_lib() {
            return Err(Error::InvalidDependency(name));
        }
        let haslib = value.kind.has_lib();
        let profiles: HashMap<_, _> = value.profiles.into_iter().map(|(k, p)| {
            let prof = LibProfile{
                include: p.include_pub,
                libdir: format!("bin/{k}").into(),
                binaries: if haslib { vec![ name.clone().into() ] } else { Vec::new() },
                defines: p.defines,
            };
            (k, prof)
        }).collect();

        Ok(Self{
            name,
            version: value.version,
            lang:    value.interface,
            profiles,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibProfile {
    pub include:  PathBuf,
    pub libdir:   PathBuf,
    pub binaries: Vec<PathBuf>,
    pub defines:  Vec<String>,
}

impl LibProfile {
    fn debug(defaults: &SerdeLibProfile) -> Self {
        let mut defines = vec![ "VANGO_DEBUG".to_string() ];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(String::to_owned));
        }
        Self{
            include:  defaults.include.clone().unwrap_or("include".into()),
            libdir:   defaults.libdir.clone().unwrap_or("bin/debug".into()),
            binaries: defaults.binaries.iter().flatten().map(PathBuf::to_owned).collect(),
            defines,
        }
    }

    fn release(defaults: &SerdeLibProfile) -> Self {
        let mut defines = vec![ "VANGO_RELEASE".to_string() ];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(String::to_owned));
        }
        Self{
            include:  defaults.include.clone().unwrap_or("include".into()),
            libdir:   defaults.libdir.clone().unwrap_or("bin/release".into()),
            binaries: defaults.binaries.iter().flatten().map(PathBuf::to_owned).collect(),
            defines,
        }
    }

    fn merge(mut self, other: SerdeLibProfile) -> Self {
        if let Some(inc) = other.include { self.include = inc; }
        if let Some(dir) = other.libdir { self.libdir = dir; }
        self.binaries.extend(other.binaries.unwrap_or_default());
        self.defines.extend(other.defines.unwrap_or_default());
        self
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeLibFile {
    staticlib: SerdeLibrary,
    #[serde(default)]
    profile: HashMap<String, SerdeLibProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeLibrary {
    name:    String,
    version: String,
    lang:    String,

    #[serde(flatten)]
    defaults: SerdeLibProfile,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
struct SerdeLibProfile {
    inherits: Option<String>,
    include:  Option<PathBuf>,
    libdir:   Option<PathBuf>,
    binaries: Option<Vec<PathBuf>>,
    defines:  Option<Vec<String>>,
}


#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[allow(unused)]
pub struct SerdeFeature {
    requires: Vec<String>,
    binaries: Option<Vec<PathBuf>>,
}

