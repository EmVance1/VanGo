mod args;

use std::{
    str::FromStr,
    fmt::Display,
    io::Write,
};
use crate::{log_warn_ln, Error};


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ProjKind {
    #[default]
    App,
    SharedLib{ implib: bool },
    StaticLib,
}

impl FromStr for ProjKind {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "app"       => Ok(ProjKind::App),
            "sharedlib" => Ok(ProjKind::SharedLib{ implib: true }),
            "staticlib" => Ok(ProjKind::StaticLib),
            _ => Err(Error::Unknown)
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolChain {
    Msvc,
    Gnu,
    Clang,
    Zig,
}

impl Default for ToolChain {
    fn default() -> Self {
        let sysdef = Self::system_default();
        match std::env::var("VANGO_DEFAULT_TOOLCHAIN") {
            Ok(var) => match var.as_str() {
                "msvc"  => return ToolChain::Msvc,
                "gnu"   => return ToolChain::Gnu,
                "clang" => return ToolChain::Clang,
                "zig"   => return ToolChain::Zig,
                _ => log_warn_ln!("'$VANGO_DEFAULT_TOOLCHAIN' was not a valid toolchain, defaulting to: {sysdef}"),
            }
            Err(std::env::VarError::NotUnicode(..)) => log_warn_ln!("'$VANGO_DEFAULT_TOOLCHAIN' was not a valid toolchain, defaulting to: {sysdef}"),
            _ => ()
        }
        sysdef
    }
}

#[allow(unused)]
impl ToolChain {
    pub fn system_default() -> Self {
        if cfg!(target_os = "windows") {
            ToolChain::Msvc
        } else if cfg!(target_os = "linux") {
            ToolChain::Gnu
        } else {
            ToolChain::Clang
        }
    }

    pub fn is_msvc(&self) -> bool {
        matches!(self, Self::Msvc)
    }
    pub fn is_gnu(&self) -> bool {
        matches!(self, Self::Gnu)
    }
    pub fn is_clang(&self) -> bool {
        matches!(self, Self::Clang|Self::Zig)
    }

    pub fn is_posix(&self) -> bool {
        matches!(self, Self::Gnu|Self::Clang|Self::Zig)
    }
    pub fn is_llvm(&self) -> bool {
        matches!(self, Self::Clang|Self::Zig)
    }

    pub fn shared_lib_prefix(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            ""
        } else {
            "lib"
        }
    }
    pub fn static_lib_prefix(&self) -> &'static str {
        match self {
            Self::Msvc => "",
            _ => "lib",
        }
    }
    pub fn ext(&self, kind: ProjKind) -> &'static str {
        match kind {
            ProjKind::App => self.app_ext(),
            ProjKind::SharedLib{..} => self.shared_lib_ext(),
            ProjKind::StaticLib => self.static_lib_ext(),
        }
    }
    pub fn app_ext(&self) -> &'static str {
        match self {
            Self::Msvc => "exe",
            _ => "",
        }
    }
    pub fn shared_lib_ext(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        }
    }
    pub fn static_lib_ext(&self) -> &'static str {
        match self {
            Self::Msvc => "lib",
            _ => "a",
        }
    }

    pub fn compiler(&self, cpp: bool) -> &'static str {
        match self {
            Self::Msvc  => "cl",
            Self::Gnu   => if cpp { "g++" }     else { "gcc" }
            Self::Clang => if cpp { "clang++" } else { "clang" }
            Self::Zig   => "zig"
        }
    }
    pub fn compiler_as_arg(&self, cpp: bool) -> Option<&'static str> {
        match self {
            Self::Msvc|Self::Gnu|Self::Clang => None,
            Self::Zig   => Some(if cpp { "c++" } else { "cc" })
        }
    }
    pub fn linker(&self, cpp: bool) -> &'static str {
        match self {
            Self::Msvc => "LINK",
            Self::Gnu|Self::Clang|Self::Zig => self.compiler(cpp),
        }
    }
    pub fn linker_as_arg(&self, cpp: bool) -> Option<&'static str> {
        match self {
            Self::Msvc|Self::Gnu|Self::Clang => None,
            Self::Zig => self.compiler_as_arg(cpp),
        }
    }
    pub fn archiver(&self) -> &'static str {
        match self {
            Self::Msvc  => "LIB",
            Self::Gnu   => "ar",
            Self::Clang => "llvm-ar",
            Self::Zig   => "zig",
        }
    }
    pub fn archiver_as_arg(&self) -> Option<&'static str> {
        match self {
            Self::Msvc|Self::Gnu|Self::Clang => None,
            Self::Zig => Some("ar"),
        }
    }

    pub fn as_arg(&self) -> &'static str {
        match self {
            Self::Msvc  => "-t=msvc",
            Self::Gnu   => "-t=gnu",
            Self::Clang => "-t=clang",
            Self::Zig   => "-t=zig",
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
            Self::Clang => write!(f, "Clang"),
            Self::Zig   => write!(f, "Zig/Clang"),
        }
    }
}


#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
            Self::Debug     => Some("VANGO_DEBUG"),
            Self::Release   => Some("VANGO_RELEASE"),
            Self::Custom(s) => None,
        }
    }
    pub fn as_arg(&self) -> String {
        match self {
            Self::Debug     => "--debug".to_string(),
            Self::Release   => "--release".to_string(),
            Self::Custom(s) => format!("--profile={s}"),
        }
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug     => write!(f, "debug"),
            Self::Release   => write!(f, "release"),
            Self::Custom(s) => write!(f, "{s}"),
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
            Self::Cpp(..) => "cpp",
            Self::C(..)   => "c",
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

impl FromStr for Lang {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_lang_cpp() {
        assert_eq!(Lang::from_str("c++98").unwrap(), Lang::Cpp(98));
        assert_eq!(Lang::from_str("c++03").unwrap(), Lang::Cpp(103));
        assert_eq!(Lang::from_str("c++11").unwrap(), Lang::Cpp(111));
        assert_eq!(Lang::from_str("c++14").unwrap(), Lang::Cpp(114));
        assert_eq!(Lang::from_str("C++17").unwrap(), Lang::Cpp(117));
        assert_eq!(Lang::from_str("C++20").unwrap(), Lang::Cpp(120));
        assert_eq!(Lang::from_str("C++23").unwrap(), Lang::Cpp(123));
    }

    #[test]
    pub fn parse_lang_c() {
        assert_eq!(Lang::from_str("c89").unwrap(), Lang::C(89));
        assert_eq!(Lang::from_str("c99").unwrap(), Lang::C(99));
        assert_eq!(Lang::from_str("C11").unwrap(), Lang::C(111));
        assert_eq!(Lang::from_str("C17").unwrap(), Lang::C(117));
        assert_eq!(Lang::from_str("C20").unwrap(), Lang::C(120));
        assert_eq!(Lang::from_str("C23").unwrap(), Lang::C(123));
    }

    #[test]
    pub fn parse_lang_err() {
        assert!(Lang::from_str("3").is_err());
        assert!(Lang::from_str("c").is_err());
        assert!(Lang::from_str("c4").is_err());
        assert!(Lang::from_str("c14").is_err());
        assert!(Lang::from_str("c++").is_err());
        assert!(Lang::from_str("c++24").is_err());
    }

    #[test]
    pub fn lang_cmp() {
        assert!(Lang::from_str("C99").unwrap() >  Lang::from_str("C89").unwrap());
        assert!(Lang::from_str("C11").unwrap() >  Lang::from_str("C89").unwrap());
        assert!(Lang::from_str("C89").unwrap() >= Lang::from_str("C89").unwrap());
        assert!(Lang::from_str("C99").unwrap() >= Lang::from_str("C89").unwrap());
        assert!(Lang::from_str("C11").unwrap() >= Lang::from_str("C99").unwrap());
        assert!(Lang::from_str("C++03").unwrap() >  Lang::from_str("C++98").unwrap());
        assert!(Lang::from_str("C++11").unwrap() >  Lang::from_str("C++98").unwrap());
        assert!(Lang::from_str("C++98").unwrap() >= Lang::from_str("C++98").unwrap());
        assert!(Lang::from_str("C++03").unwrap() >= Lang::from_str("C++98").unwrap());
        assert!(Lang::from_str("C++11").unwrap() >= Lang::from_str("C++98").unwrap());
    }
}
