mod buildfile;
mod libfile;
mod args;

pub use buildfile::*;
pub use libfile::*;
use std::fmt::Display;
use crate::Error;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjKind {
    App,
    Lib,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolChain {
    Msvc,
    Gnu,
    Clang,
}

impl Default for ToolChain {
    fn default() -> Self {
        if cfg!(target_os = "windows") {
            ToolChain::Msvc
        } else if cfg!(target_os = "linux") {
            ToolChain::Gnu
        } else {
            ToolChain::Clang
        }
    }
}

#[allow(unused)]
impl ToolChain {
    pub fn is_msvc(&self) -> bool {
        matches!(self, Self::Msvc)
    }
    pub fn is_gnu(&self) -> bool {
        matches!(self, Self::Gnu)
    }
    pub fn is_clang(&self) -> bool {
        matches!(self, Self::Clang)
    }

    pub fn is_posix(&self) -> bool {
        matches!(self, Self::Gnu|Self::Clang)
    }
    pub fn is_llvm(&self) -> bool {
        matches!(self, Self::Clang)
    }

    pub fn lib_prefix(&self) -> &'static str {
        match self {
            Self::Msvc => "",
            _ => "lib",
        }
    }
    pub fn ext(&self, kind: ProjKind) -> &'static str {
        match kind {
            ProjKind::App => self.app_ext(),
            ProjKind::Lib => self.lib_ext(),
        }
    }
    pub fn app_ext(&self) -> &'static str {
        match self {
            Self::Msvc => ".exe",
            _ => "",
        }
    }
    pub fn lib_ext(&self) -> &'static str {
        match self {
            Self::Msvc => ".lib",
            _ => ".a",
        }
    }

    pub fn compiler(&self, cpp: bool) -> &'static str {
        match self {
            Self::Msvc  => "cl",
            Self::Gnu   => if cpp { "g++" } else { "gcc" }
            Self::Clang => if cpp { "clang++" } else { "clang" }
        }
    }
    pub fn linker(&self, cpp: bool) -> &'static str {
        match self {
            Self::Msvc => "LINK",
            Self::Gnu|Self::Clang => self.compiler(cpp),
        }
    }
    pub fn archiver(&self) -> &'static str {
        match self {
            Self::Msvc  => "LIB",
            Self::Gnu   => "ar",
            Self::Clang => "llvm-ar",
        }
    }

    pub fn as_arg(&self) -> &'static str {
        match self {
            Self::Msvc  => "-t=msvc",
            Self::Gnu   => "-t=gnu",
            Self::Clang => "-t=clang",
        }
    }

    pub fn args(&self) -> args::Args {
        args::Args(*self)
    }
}

impl Display for ToolChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Msvc  => write!(f, "MSVC"),
            Self::Gnu   => write!(f, "GCC"),
            Self::Clang => write!(f, "Clang/LLVM"),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Config {
    Debug,
    Release,
}

#[allow(unused)]
impl Config {
    pub fn is_debug(&self) -> bool {
        *self == Config::Debug
    }
    pub fn is_release(&self) -> bool {
        *self == Config::Release
    }
    pub fn as_define(&self) -> &'static str {
        match self {
            Self::Debug   => "DEBUG",
            Self::Release => "RELEASE",
        }
    }
    pub fn as_arg(&self) -> &'static str {
        match self {
            Self::Debug   => "--debug",
            Self::Release => "--release",
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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Cpp(u32),
    C(u32),
}

impl Lang {
    pub fn is_cpp(&self) -> bool {
        matches!(self, Self::Cpp(_))
    }
    pub fn is_latest(&self) -> bool {
        match *self {
            Self::Cpp(n) => n == 123,
            Self::C(n)   => n == 123,
        }
    }

    pub fn src_ext(&self) -> &'static str {
        match self {
            Self::Cpp(..) => ".cpp",
            Self::C(..)   => ".c",
        }
    }

    pub fn numeric(&self) -> u32 {
        match *self {
            Self::Cpp(n)|Self::C(n) => if n >= 100 { n - 100 } else { n },
        }
    }
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Cpp(_) => write!(f, "c++{}", self.numeric()),
            Self::C(_)   => write!(f, "c{}",   self.numeric()),
        }
    }
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

impl TryFrom<&str> for Lang {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let cpp = value.to_ascii_lowercase();
        if cpp.starts_with("c++") {
            let num: u32 = cpp.strip_prefix("c++")
                .unwrap()
                .parse()
                .map_err(|_| Error::InvalidCppStd(cpp.to_string()))?;
            if !matches!(num, 98 | 3 | 11 | 14 | 17 | 20 | 23) {
                Err(Error::InvalidCppStd(cpp.to_string()))
            } else if num < 80 {
                Ok(Lang::Cpp(100 + num))
            } else {
                Ok(Lang::Cpp(num))
            }
        } else {
            let num: u32 = cpp.strip_prefix("c")
                .ok_or(Error::InvalidCppStd(cpp.to_string()))?
                .parse()
                .map_err(|_| Error::InvalidCppStd(cpp.to_string()))?;
            if !matches!(num, 89 | 99 | 11 | 17 | 20 | 23) {
                Err(Error::InvalidCppStd(cpp.to_string()))
            } else if num < 80 {
                Ok(Lang::C(100 + num))
            } else {
                Ok(Lang::C(num))
            }
        }
    }
}

impl TryFrom<&String> for Lang {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_lang_cpp() {
        assert_eq!(Lang::try_from("c++98").unwrap(), Lang::Cpp(98));
        assert_eq!(Lang::try_from("c++03").unwrap(), Lang::Cpp(103));
        assert_eq!(Lang::try_from("c++11").unwrap(), Lang::Cpp(111));
        assert_eq!(Lang::try_from("c++14").unwrap(), Lang::Cpp(114));
        assert_eq!(Lang::try_from("C++17").unwrap(), Lang::Cpp(117));
        assert_eq!(Lang::try_from("C++20").unwrap(), Lang::Cpp(120));
        assert_eq!(Lang::try_from("C++23").unwrap(), Lang::Cpp(123));
    }

    #[test]
    pub fn parse_lang_c() {
        assert_eq!(Lang::try_from("c89").unwrap(), Lang::C(89));
        assert_eq!(Lang::try_from("c99").unwrap(), Lang::C(99));
        assert_eq!(Lang::try_from("C11").unwrap(), Lang::C(111));
        assert_eq!(Lang::try_from("C17").unwrap(), Lang::C(117));
        assert_eq!(Lang::try_from("C20").unwrap(), Lang::C(120));
        assert_eq!(Lang::try_from("C23").unwrap(), Lang::C(123));
    }

    #[test]
    pub fn parse_lang_err() {
        assert!(Lang::try_from("c++24").is_err());
        assert!(Lang::try_from("c++").is_err());
        assert!(Lang::try_from("c14").is_err());
        assert!(Lang::try_from("c4").is_err());
        assert!(Lang::try_from("c").is_err());
        assert!(Lang::try_from("3").is_err());
    }
}
