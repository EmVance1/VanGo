use std::{collections::HashMap, path::PathBuf, str::FromStr};
use serde::Deserialize;
use crate::{error::Error, repr::{self, Lang, BuildFile}};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibFile {
    pub library: String,
    pub lang:    Lang,
    pub profile: HashMap<String, LibProfile>,
}

impl LibFile {
    pub fn from_str(value: &str) -> Result<LibFile, Error> {
        let mut file = serde_json::from_str::<SerdeLibFile>(value)?;
        let mut profile: HashMap<String, LibProfile> = HashMap::new();

        if let Some(d) = file.profile.remove("debug") {
            profile.insert("debug".to_string(), LibProfile::debug(&file.defaults).merge(d));
        } else {
            profile.insert("debug".to_string(), LibProfile::debug(&file.defaults));
        }
        if let Some(r) = file.profile.remove("release") {
            profile.insert("release".to_string(), LibProfile::release(&file.defaults).merge(r));
        } else {
            profile.insert("release".to_string(), LibProfile::release(&file.defaults));
        }
        for (k, p) in file.profile {
            let inherits = p.inherits.clone().ok_or(Error::InvalidCustomProfile(k.clone()))?;
            if inherits == "debug" {
                profile.insert(k, LibProfile::debug(&file.defaults).merge(p));
            } else if inherits == "release" {
                profile.insert(k, LibProfile::release(&file.defaults).merge(p));
            }
        }

        Ok(LibFile{
            library: file.library,
            lang: Lang::from_str(&file.lang)?,
            profile,
        })
    }

    pub fn take(&mut self, profile: &repr::Profile) -> Result<LibProfile, Error> {
        match profile {
            repr::Profile::Debug => self.profile.remove("debug"),
            repr::Profile::Release => self.profile.remove("release"),
            repr::Profile::Custom(s) => self.profile.remove(s),
        }.ok_or(Error::ProfileUnavailable(self.library.clone(), profile.to_string()))
    }

    pub fn validate(self, max_lang: Lang) -> Result<Self, Error> {
        if self.lang > max_lang {
            Err(Error::IncompatibleCppStd(self.library))
        } else {
            Ok(self)
        }
    }
}

impl From<BuildFile> for LibFile {
    fn from(value: BuildFile) -> Self {
        let project = value.project;
        let profile: HashMap<_, _> = value.profile.into_iter().map(|(k, p)| {
            let prof = LibProfile{
                include: p.include_pub,
                libdir: format!("bin/{k}").into(),
                binaries: vec![ project.clone().into() ],
                defines: p.defines,
            };
            (k, prof)
        }).collect();

        Self {
            library: project,
            lang: value.lang,
            profile,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SerdeLibFile {
    pub library: String,
    pub lang:    String,

    #[serde(flatten)]
    pub defaults: SerdeProfile,
    #[serde(default)]
    pub profile: HashMap<String, SerdeProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibProfile {
    pub include:  PathBuf,
    pub libdir:  PathBuf,
    pub binaries: Vec<PathBuf>,
    pub defines:  Vec<String>,
}

impl LibProfile {
    fn debug(defaults: &SerdeProfile) -> Self {
        Self{
            include: defaults.include.clone().unwrap_or("include".into()),
            libdir: defaults.libdir.clone().unwrap_or("bin/debug".into()),
            binaries: defaults.binaries.iter().flatten().map(|b| b.to_owned()).collect(),
            defines:  defaults.defines.iter().flatten().map(|d| d.to_owned()).collect(),
        }
    }

    fn release(defaults: &SerdeProfile) -> Self {
        Self{
            include: defaults.include.clone().unwrap_or("include".into()),
            libdir: defaults.libdir.clone().unwrap_or("bin/release".into()),
            binaries: defaults.binaries.iter().flatten().map(|b| b.to_owned()).collect(),
            defines:  defaults.defines.iter().flatten().map(|d| d.to_owned()).collect(),
        }
    }

    fn merge(mut self, other: SerdeProfile) -> Self {
        if let Some(inc) = other.include { self.include = inc; }
        if let Some(dir) = other.libdir { self.libdir = dir; }
        self.binaries.extend(other.binaries.unwrap_or_default());
        self.defines.extend(other.defines.unwrap_or_default());
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
struct SerdeProfile {
    pub inherits: Option<String>,
    pub include:  Option<PathBuf>,
    pub libdir:  Option<PathBuf>,
    pub binaries: Option<Vec<PathBuf>>,
    pub defines:  Option<Vec<String>>,
}

