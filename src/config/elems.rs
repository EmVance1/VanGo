use crate::Error;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, str::FromStr, collections::HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('.');
        let result = Version {
            major: iter
                .next()
                .ok_or(Error::MimicTomlSemver(s.to_string()))?
                .parse()
                .map_err(|_| Error::MimicTomlSemver(s.to_string()))?,
            minor: iter
                .next()
                .ok_or(Error::MimicTomlSemver(s.to_string()))?
                .parse()
                .map_err(|_| Error::MimicTomlSemver(s.to_string()))?,
            patch: iter
                .next()
                .ok_or(Error::MimicTomlSemver(s.to_string()))?
                .parse()
                .map_err(|_| Error::MimicTomlSemver(s.to_string()))?,
        };
        if iter.next().is_some() {
            Err(Error::MimicTomlSemver(s.to_string()))
        } else {
            Ok(result)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    Macos,
}

impl Platform {
    pub fn current() -> Result<Self, Error> {
        if cfg!(windows) {
            Ok(Platform::Windows)
        } else if cfg!(target_os = "linux") {
            Ok(Platform::Linux)
        } else if cfg!(target_os = "macos") {
            Ok(Platform::Macos)
        } else {
            Err(Error::PlatformUnavailable)
        }
    }

    pub fn fmt_shared_lib(self, name: &str) -> PathBuf {
        match self {
            Self::Windows => PathBuf::from(name).with_added_extension("dll"),
            Self::Linux => PathBuf::from(&format!("lib{}.so", name)),
            Self::Macos => PathBuf::from(&format!("lib{}.dylib", name)),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Artefact {
    #[default]
    Executable,
    SharedLib,
    StaticLib,
}

impl Artefact {
    pub fn is_lib(self) -> bool {
        matches!(self, Artefact::StaticLib | Artefact::SharedLib)
    }
}

impl FromStr for Artefact {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "executable" => Ok(Artefact::Executable),
            "sharedlib" => Ok(Artefact::SharedLib),
            "staticlib" => Ok(Artefact::StaticLib),
            _ => Err(Error::MimicTomlArtefact(s.to_string())),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Profile {
    #[default]
    Debug,
    Release,
    Custom(String),
}

#[allow(unused)]
impl Profile {
    pub fn is_debug(&self) -> bool {
        *self == Self::Debug
    }
    pub fn is_release(&self) -> bool {
        *self == Self::Release
    }
    pub fn as_define(&self) -> Option<&'static str> {
        match self {
            Self::Debug => Some("VANGO_DEBUG"),
            Self::Release => Some("VANGO_RELEASE"),
            Self::Custom(s) => None,
        }
    }
    pub fn as_arg(&self) -> String {
        match self {
            Self::Debug => "--debug".to_string(),
            Self::Release => "--release".to_string(),
            Self::Custom(s) => format!("--profile={s}"),
        }
    }
}

impl FromStr for Profile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(Profile::Debug),
            "release" => Ok(Profile::Release),
            _ => Ok(Profile::Custom(s.to_string())),
        }
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "debug"),
            Self::Release => write!(f, "release"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Cpp(u32),
    C(u32),
}

impl Language {
    pub fn is_cpp(self) -> bool {
        matches!(self, Self::Cpp(_))
    }

    pub fn src_ext(self) -> &'static str {
        match self {
            Self::Cpp(..) => "cpp",
            Self::C(..) => "c",
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Cpp(n) => write!(f, "c++{}", if n >= 100 { n - 100 } else { n }),
            Self::C(n) => write!(f, "c{}", if n >= 100 { n - 100 } else { n }),
        }
    }
}

impl Ord for Language {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Language::Cpp(a), Language::Cpp(b)) | (Language::C(a), Language::C(b)) => a.cmp(b),
            (Language::Cpp(_), Language::C(_)) => 1.cmp(&0),
            (Language::C(_), Language::Cpp(_)) => 0.cmp(&1),
        }
    }
}

impl PartialOrd for Language {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let cpp = value.to_ascii_lowercase();
        if cpp.starts_with("c++") {
            let num: u32 = cpp
                .strip_prefix("c++")
                .unwrap()
                .parse()
                .map_err(|_| Error::InvalidCppStd(cpp.to_string()))?;
            if !matches!(num, 98 | 3 | 11 | 14 | 17 | 20 | 23) {
                Err(Error::InvalidCppStd(cpp.to_string()))
            } else if num < 80 {
                Ok(Language::Cpp(100 + num))
            } else {
                Ok(Language::Cpp(num))
            }
        } else {
            let num: u32 = cpp
                .strip_prefix("c")
                .ok_or(Error::InvalidCppStd(cpp.to_string()))?
                .parse()
                .map_err(|_| Error::InvalidCppStd(cpp.to_string()))?;
            if !matches!(num, 89 | 99 | 11 | 17 | 23) {
                Err(Error::InvalidCppStd(cpp.to_string()))
            } else if num < 80 {
                Ok(Language::C(100 + num))
            } else {
                Ok(Language::C(num))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VcpkgConfig {
    pub triplet: String,
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PrecompiledHeader {
    pub header: PathBuf,
    #[serde(default)]
    pub used_by: HashSet<PathBuf>,
    #[serde(default)]
    pub ignored_by: HashSet<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct Sanitizer {
    pub address: Option<bool>,
    pub thread: Option<bool>,
    pub leak: Option<bool>,
    pub undefined: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_lang_cpp() {
        assert_eq!(Language::from_str("c++98").unwrap(), Language::Cpp(98));
        assert_eq!(Language::from_str("c++03").unwrap(), Language::Cpp(103));
        assert_eq!(Language::from_str("c++11").unwrap(), Language::Cpp(111));
        assert_eq!(Language::from_str("c++14").unwrap(), Language::Cpp(114));
        assert_eq!(Language::from_str("C++17").unwrap(), Language::Cpp(117));
        assert_eq!(Language::from_str("C++20").unwrap(), Language::Cpp(120));
        assert_eq!(Language::from_str("C++23").unwrap(), Language::Cpp(123));
    }

    #[test]
    pub fn parse_lang_c() {
        assert_eq!(Language::from_str("c89").unwrap(), Language::C(89));
        assert_eq!(Language::from_str("c99").unwrap(), Language::C(99));
        assert_eq!(Language::from_str("C11").unwrap(), Language::C(111));
        assert_eq!(Language::from_str("C17").unwrap(), Language::C(117));
        assert_eq!(Language::from_str("C23").unwrap(), Language::C(123));
    }

    #[test]
    pub fn parse_lang_err() {
        assert!(Language::from_str("3").is_err());
        assert!(Language::from_str("c").is_err());
        assert!(Language::from_str("c4").is_err());
        assert!(Language::from_str("c14").is_err());
        assert!(Language::from_str("c20").is_err());
        assert!(Language::from_str("c++").is_err());
        assert!(Language::from_str("c++24").is_err());
        assert!(Language::from_str("c++12").is_err());
        assert!(Language::from_str("abcde").is_err());
    }

    #[test]
    pub fn lang_cmp() {
        assert!(Language::from_str("C99").unwrap() > Language::from_str("C89").unwrap());
        assert!(Language::from_str("C11").unwrap() > Language::from_str("C89").unwrap());
        assert!(Language::from_str("C89").unwrap() >= Language::from_str("C89").unwrap());
        assert!(Language::from_str("C99").unwrap() >= Language::from_str("C89").unwrap());
        assert!(Language::from_str("C11").unwrap() >= Language::from_str("C99").unwrap());
        assert!(Language::from_str("C++03").unwrap() > Language::from_str("C++98").unwrap());
        assert!(Language::from_str("C++11").unwrap() > Language::from_str("C++98").unwrap());
        assert!(Language::from_str("C++98").unwrap() >= Language::from_str("C++98").unwrap());
        assert!(Language::from_str("C++03").unwrap() >= Language::from_str("C++98").unwrap());
        assert!(Language::from_str("C++11").unwrap() >= Language::from_str("C++98").unwrap());
    }
}
