mod profile;
mod raw;

use super::{Artefact, Language, Profile, Version, build::PackageManifest};
use crate::{exec::Toolchain, error::Error};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibManifest {
    pub name: String,
    pub version: Version,
    pub lang: Language,
    pub profiles: HashMap<Profile, LibProfile>,
}

impl LibManifest {
    pub fn from_table(value: toml::Table) -> Result<LibManifest, Error> {
        let file: raw::LibManifest = value.try_into()?;
        file.try_into()
    }

    pub fn from_build(value: PackageManifest, toolchain: Toolchain) -> Result<Self, Error> {
        let name = value.name;
        if !value.artefact.is_lib() {
            return Err(Error::InvalidDependency(name));
        }
        let libbase = if toolchain == Toolchain::system_default()? {
            PathBuf::from("bin")
        } else {
            PathBuf::from("bin").join(toolchain.as_directory())
        };
        let haslib = (value.artefact == Artefact::StaticLib) || value.implib;
        let profiles: HashMap<_, _> = value
            .profiles
            .into_iter()
            .map(|(k, p)| {
                let prof = LibProfile {
                    include: "include".into(),
                    libdir: libbase.join(k.to_string()),
                    binaries: if haslib { vec![name.clone().into()] } else { Vec::new() },
                    defines: p.defines,
                };
                (k, prof)
            })
            .collect();

        Ok(Self {
            name,
            version: value.version,
            lang: value.interface,
            profiles,
        })
    }

    pub fn remove(&mut self, profile: &Profile) -> Result<LibProfile, Error> {
        self.profiles
            .remove(profile)
            .ok_or(Error::ProfileUnavailable(self.name.clone(), profile.to_string()))
    }

    pub fn validate(self, other_name: &str, other_lang: Language) -> Result<Self, Error> {
        if self.lang > other_lang {
            Err(Error::IncompatibleCppStd(self.name, self.lang, other_name.to_string(), other_lang))
        } else {
            Ok(self)
        }
    }
}

impl TryFrom<raw::LibManifest> for LibManifest {
    type Error = Error;

    fn try_from(value: raw::LibManifest) -> Result<Self, Self::Error> {
        let mut profiles = HashMap::new();

        profiles.insert(Profile::Debug, LibProfile::default_debug().layer(value.staticlib.defaults.clone()));
        profiles.insert(
            Profile::Release,
            LibProfile::default_release().layer(value.staticlib.defaults.clone()),
        );
        for (k, v) in value.profile {
            let base = v.inherits.as_ref().unwrap_or(&k);
            if base == "debug" {
                profiles.insert(
                    Profile::from_str(&k)?,
                    LibProfile::default_debug().layer(value.staticlib.defaults.clone()).layer(v),
                );
            } else if base == "release" {
                profiles.insert(
                    Profile::from_str(&k)?,
                    LibProfile::default_release().layer(value.staticlib.defaults.clone()).layer(v),
                );
            } else {
                return Err(Error::InvalidCustomProfile(k));
            };
        }

        Ok(LibManifest {
            name: value.staticlib.name,
            version: Version::from_str(&value.staticlib.version)?,
            lang: Language::from_str(&value.staticlib.lang)?,
            profiles,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibProfile {
    pub include: PathBuf,
    pub libdir: PathBuf,
    pub binaries: Vec<PathBuf>,
    pub defines: Vec<String>,
}
