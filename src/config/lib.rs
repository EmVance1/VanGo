use std::{collections::HashMap, path::PathBuf, str::FromStr};
use serde::Deserialize;
use crate::{error::Error, repr::{self, Lang}};
use super::build::BuildFile;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibFile {
    pub library: Library,
    pub profile: HashMap<String, LibProfile>,
}

impl LibFile {
    pub fn from_table(value: toml::Table) -> Result<LibFile, Error> {
        let mut file: SerdeLibFile = value.try_into()?;
        let mut profile: HashMap<String, LibProfile> = HashMap::new();

        if let Some(d) = file.profile.remove("debug") {
            profile.insert("debug".to_string(), LibProfile::debug(&file.library.defaults).merge(d));
        } else {
            profile.insert("debug".to_string(), LibProfile::debug(&file.library.defaults));
        }
        if let Some(r) = file.profile.remove("release") {
            profile.insert("release".to_string(), LibProfile::release(&file.library.defaults).merge(r));
        } else {
            profile.insert("release".to_string(), LibProfile::release(&file.library.defaults));
        }
        for (k, p) in file.profile {
            let inherits = p.inherits.clone().ok_or(Error::InvalidCustomProfile(k.clone()))?;
            if inherits == "debug" {
                profile.insert(k, LibProfile::debug(&file.library.defaults).merge(p));
            } else if inherits == "release" {
                profile.insert(k, LibProfile::release(&file.library.defaults).merge(p));
            }
        }

        Ok(LibFile{
            library: Library{
                package: file.library.package,
                version: file.library.version,
                lang: Lang::from_str(&file.library.lang)?,
            },
            profile,
        })
    }

    pub fn take(&mut self, profile: &repr::Profile) -> Result<LibProfile, Error> {
        match profile {
            repr::Profile::Debug => self.profile.remove("debug"),
            repr::Profile::Release => self.profile.remove("release"),
            repr::Profile::Custom(s) => self.profile.remove(s),
        }.ok_or(Error::ProfileUnavailable(self.library.package.clone(), profile.to_string()))
    }

    pub fn validate(self, max_lang: Lang) -> Result<Self, Error> {
        if self.library.lang > max_lang {
            Err(Error::IncompatibleCppStd(self.library.package))
        } else {
            Ok(self)
        }
    }
}

impl From<BuildFile> for LibFile {
    fn from(value: BuildFile) -> Self {
        let package = value.build.package;
        let profile: HashMap<_, _> = value.profile.into_iter().map(|(k, p)| {
            let prof = LibProfile{
                include: p.include_pub,
                libdir: format!("bin/{k}").into(),
                binaries: vec![ package.clone().into() ],
                defines: p.defines,
            };
            (k, prof)
        }).collect();

        Self {
            library: Library {
                package,
                version: value.build.version,
                lang:    value.build.lang,
            },
            profile,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub package: String,
    pub version: String,
    pub lang: Lang,
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
            defines.extend(def.iter().map(|d| d.to_owned()));
        }
        Self{
            include:  defaults.include.clone().unwrap_or("include".into()),
            libdir:   defaults.libdir.clone().unwrap_or("bin/debug".into()),
            binaries: defaults.binaries.iter().flatten().map(|b| b.to_owned()).collect(),
            defines,
        }
    }

    fn release(defaults: &SerdeLibProfile) -> Self {
        let mut defines = vec![ "VANGO_RELEASE".to_string() ];
        if let Some(def) = &defaults.defines {
            defines.extend(def.iter().map(|d| d.to_owned()));
        }
        Self{
            include:  defaults.include.clone().unwrap_or("include".into()),
            libdir:   defaults.libdir.clone().unwrap_or("bin/release".into()),
            binaries: defaults.binaries.iter().flatten().map(|b| b.to_owned()).collect(),
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
    library: SerdeLibrary,
    #[serde(default)]
    profile: HashMap<String, SerdeLibProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeLibrary {
    package: String,
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
pub struct SerdeFeature {
    requires: Vec<String>,
    binaries: Option<Vec<PathBuf>>,
}

