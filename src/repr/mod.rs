mod buildfile;
mod libfile;

pub use buildfile::*;
pub use libfile::*;
use std::fmt::Display;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjKind {
    App,
    Lib,
}

impl ProjKind {
    #[cfg(target_os = "windows")]
    pub fn ext(&self, mingw: bool) -> String {
        match self {
            Self::App => ".exe".to_string(),
            Self::Lib => if mingw { ".a".to_string() } else { ".lib".to_string() },
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn ext(&self, _: bool) -> String {
        match self {
            Self::App =>   "".to_string(),
            Self::Lib => ".a".to_string(),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    Debug,
    Release,
}

impl Config {
    #[allow(unused)]
    pub fn is_debug  (&self) -> bool { *self == Config::Debug }
    pub fn is_release(&self) -> bool { *self == Config::Release }
    pub fn as_arg(&self) -> String {
        match self {
            Self::Debug   => "DEBUG".to_string(),
            Self::Release => "RELEASE".to_string(),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug   => write!(f, "debug"),
            Self::Release => write!(f, "release"),
        }
    }
}

