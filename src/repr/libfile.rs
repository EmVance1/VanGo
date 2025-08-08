use crate::{Config, error::Error};
use serde::Deserialize;
use std::collections::HashMap;

use super::BuildFile;

#[derive(Debug, Clone, Deserialize)]
pub struct LibFile {
    pub library: String,
    pub minstd: String,
    pub include: String,
    #[serde(default)]
    pub all: Option<LibConfig>,
    #[serde(default)]
    pub configs: HashMap<String, LibConfig>,
}

impl LibFile {
    pub fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibConfig {
    #[serde(rename = "binary.debug")]
    pub binary_debug: String,
    #[serde(rename = "binary.release")]
    pub binary_release: String,
    pub links: Vec<String>,
    #[serde(default)]
    pub defines: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibData {
    pub incdir: String,
    pub libdir: String,
    pub links: Vec<String>,
    pub defines: Vec<String>,
}

impl LibFile {
    pub fn validate(self, maxcpp: &str) -> Result<Self, Error> {
        if u32_from_cppstd(&self.minstd)? > u32_from_cppstd(maxcpp)? {
            Err(Error::IncompatibleCppStd(self.library))
        } else {
            Ok(self)
        }
    }

    pub fn linearise(self, config: Config, version: Option<&str>) -> Result<LibData, Error> {
        let (libdir, links, defines) = if let Some(ver) = version {
            if let Some(cfg) = self.configs.get(ver) {
                if config.is_release() {
                    (
                        cfg.binary_release.clone(),
                        cfg.links.clone(),
                        cfg.defines.clone(),
                    )
                } else {
                    (
                        cfg.binary_debug.clone(),
                        cfg.links.clone(),
                        cfg.defines.clone(),
                    )
                }
            } else {
                return Err(Error::ConfigUnavailable(self.library, ver.to_string()));
            }
        } else if let Some(all) = self.all {
            if config.is_release() {
                (all.binary_release, all.links, all.defines)
            } else {
                (all.binary_debug, all.links, all.defines)
            }
        } else {
            return Err(Error::ConfigUnavailable(
                self.library,
                "default".to_string(),
            ));
        };

        Ok(LibData {
            incdir: self.include,
            libdir,
            links,
            defines,
        })
    }
}

impl From<BuildFile> for LibFile {
    fn from(value: BuildFile) -> Self {
        let include = if let Some(inc) = value.inc_public {
            inc
        } else {
            value.srcdir
        };
        Self {
            library: value.project.clone(),
            minstd: value.cpp,
            include,
            all: Some(LibConfig {
                binary_debug: "bin/debug/".to_string(),
                binary_release: "bin/release/".to_string(),
                links: vec![value.project],
                defines: value.defines,
            }),
            configs: HashMap::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Lang {
    Cpp(u32),
    C(u32),
}

impl Ord for Lang {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Lang::Cpp(a), Lang::Cpp(b)) => a.cmp(b),
            (Lang::Cpp(_), Lang::C(_)) => 1.cmp(&0),
            (Lang::C(_), Lang::Cpp(_)) => 0.cmp(&1),
            (Lang::C(a), Lang::C(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Lang {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn u32_from_cppstd(cpp: &str) -> Result<Lang, Error> {
    let cpp = cpp.to_ascii_lowercase();
    if cpp.starts_with("c++") {
        let num: u32 = cpp
            .strip_prefix("c++")
            .unwrap()
            .parse()
            .map_err(|_| Error::InvalidCppStd(cpp.to_string()))?;
        if !matches!(num, 98 | 3 | 11 | 14 | 17 | 20 | 23) {
            Err(Error::InvalidCppStd(cpp.to_string()))
        } else if num < 50 {
            Ok(Lang::Cpp(100 + num))
        } else {
            Ok(Lang::Cpp(num))
        }
    } else if cpp == "c" {
        Ok(Lang::C(0))
    } else {
        let num: u32 = cpp
            .strip_prefix("c")
            .ok_or(Error::InvalidCppStd(cpp.to_string()))?
            .parse()
            .map_err(|_| Error::InvalidCppStd(cpp.to_string()))?;
        if !matches!(num, 89 | 99 | 11 | 17 | 20) {
            Err(Error::InvalidCppStd(cpp.to_string()))
        } else if num < 50 {
            Ok(Lang::C(100 + num))
        } else {
            Ok(Lang::C(num))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_u32_from_cppstd() {
        assert_eq!(u32_from_cppstd("c++98").unwrap(), Lang::Cpp(98));
        assert_eq!(u32_from_cppstd("c++03").unwrap(), Lang::Cpp(103));
        assert_eq!(u32_from_cppstd("c++11").unwrap(), Lang::Cpp(111));
        assert_eq!(u32_from_cppstd("c++14").unwrap(), Lang::Cpp(114));
        assert_eq!(u32_from_cppstd("C++17").unwrap(), Lang::Cpp(117));
        assert_eq!(u32_from_cppstd("C++20").unwrap(), Lang::Cpp(120));
        assert_eq!(u32_from_cppstd("C++23").unwrap(), Lang::Cpp(123));
    }

    #[test]
    pub fn test_u32_from_cstd() {
        assert_eq!(u32_from_cppstd("c89").unwrap(), Lang::C(89));
        assert_eq!(u32_from_cppstd("c99").unwrap(), Lang::C(99));
        assert_eq!(u32_from_cppstd("C11").unwrap(), Lang::C(111));
        assert_eq!(u32_from_cppstd("C17").unwrap(), Lang::C(117));
        assert_eq!(u32_from_cppstd("C20").unwrap(), Lang::C(120));
    }

    #[test]
    pub fn test_u32_from_cstd_err() {
        assert!(u32_from_cppstd("c++24").is_err());
        assert!(u32_from_cppstd("c++").is_err());
        assert!(u32_from_cppstd("c23").is_err());
        assert!(u32_from_cppstd("c4").is_err());
        assert!(u32_from_cppstd("3").is_err());
    }
}
