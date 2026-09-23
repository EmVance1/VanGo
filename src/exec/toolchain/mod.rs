mod cmd;
mod output;
#[cfg(test)]
mod test;

use serde::Deserialize;
use std::{fmt::Display, path::{Path, PathBuf}, str::FromStr };
use crate::{Error, config::{Platform, Language}, exec::{PreCompHead, BuildInfo}};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Toolchain {
    Msvc,
    Gcc,
    Mingw,
    ClangMsvc,
    ClangGcc,
    ClangMingw,
    Zig,
    Emcc,
}

impl Toolchain {
    pub fn system_default() -> Result<Self, Error> {
        match Platform::current()? {
            Platform::Windows => Ok(Toolchain::Msvc),
            Platform::Linux   => Ok(Toolchain::Gcc),
            Platform::Macos   => Ok(Toolchain::ClangGcc),
        }
    }

    pub fn user_default() -> Result<Self, Error> {
        let sysdef = Self::system_default()?;
        match std::env::var("VANGO_DEFAULT_TOOLCHAIN") {
            Ok(var) => match Self::from_str(&var) {
                Ok(tc) => Ok(tc),
                Err(_) => {
                    // log_error_ln!("{}", e);
                    // log_warn_ln!("'$VANGO_DEFAULT_TOOLCHAIN' was not a valid toolchain, defaulting to: {sysdef}");
                    Ok(sysdef)
                }
            },
            Err(std::env::VarError::NotUnicode(..)) => {
                // log_warn_ln!("'$VANGO_DEFAULT_TOOLCHAIN' was not a valid toolchain, defaulting to: {sysdef}");
                Ok(sysdef)
            }
            Err(std::env::VarError::NotPresent) => Ok(sysdef)
        }
    }

    pub fn as_directory(self) -> &'static str {
        match self {
            Self::Msvc       => "msvc",
            Self::Gcc        => "gcc",
            Self::Mingw      => "mingw",
            Self::ClangMsvc  => "clang-msvc",
            Self::ClangGcc   => "clang-gcc",
            Self::ClangMingw => "clang-mingw",
            Self::Zig        => "zig",
            Self::Emcc       => "emcc",
        }
    }

    #[allow(dead_code)]
    pub fn is_msvc_compatible(self) -> bool {
        matches!(self, Self::Msvc | Self::ClangMsvc)
    }
    #[allow(dead_code)]
    pub fn is_gcc_compatible(self) -> bool {
        matches!(self, Self::Gcc | Self::ClangGcc | Self::Zig | Self::Emcc)
    }
    #[allow(dead_code)]
    pub fn is_llvm(self) -> bool {
        matches!(self, Self::ClangMsvc | Self::ClangGcc | Self::ClangMingw | Self::Zig | Self::Emcc)
    }
    #[allow(dead_code)]
    pub fn is_emcc(self) -> bool {
        matches!(self, Self::Emcc)
    }
    #[allow(dead_code)]
    pub fn supports(self, platform: Platform) -> bool {
        match platform {
            Platform::Windows => matches!(self, Self::Msvc | Self::Mingw | Self::ClangMsvc | Self::ClangMingw | Self::Zig | Self::Emcc),
            Platform::Linux   => matches!(self, Self::Gcc | Self::ClangGcc | Self::Zig | Self::Emcc),
            Platform::Macos   => matches!(self, Self::Gcc | Self::ClangGcc | Self::Zig | Self::Emcc),
        }
    }
    #[allow(dead_code)]
    pub fn is_windows(self) -> bool {
        matches!(self, Self::Msvc | Self::Mingw | Self::ClangMsvc | Self::ClangMingw)
    }

    pub fn fmt_executable(self, name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    pub fn fmt_static_lib(self, name: &str) -> PathBuf {
        if self.is_msvc_compatible() {
            PathBuf::from(name).with_added_extension("lib")
        } else {
            PathBuf::from(&format!("lib{}.a", name))
        }
    }

    pub fn object_extension(self) -> &'static str {
        if self.is_msvc_compatible() {
            "obj"
        } else {
            "o"
        }
    }

    pub fn compiler(self) -> Compiler {
        Compiler(self)
    }

    pub fn linker(self) -> Linker {
        Linker(self)
    }

    pub fn archiver(self) -> Archiver {
        Archiver(self)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Compiler(Toolchain);

impl Compiler {
    fn exe(self, lang: Language) -> std::process::Command {
        match self.0 {
            Toolchain::Msvc => std::process::Command::new("cl"),
            Toolchain::Gcc | Toolchain::Mingw => std::process::Command::new(if lang.is_cpp() { "g++" } else { "gcc" }),
            Toolchain::ClangMsvc => std::process::Command::new("clang-cl"),
            Toolchain::ClangGcc => std::process::Command::new(if lang.is_cpp() { "clang++" } else { "clang" }),
            Toolchain::ClangMingw => {
                let mut cmd = std::process::Command::new(if lang.is_cpp() { "clang++" } else { "clang" });
                cmd.arg("--target=x86_64-w64-mingw32");
                cmd
            }
            Toolchain::Zig => {
                let mut cmd = std::process::Command::new("zig");
                cmd.arg(if lang.is_cpp() { "c++" } else { "cc" });
                cmd
            }
            Toolchain::Emcc => std::process::Command::new(if lang.is_cpp() { "em++.bat" } else { "emcc.bat" }),
        }
    }

    pub fn command(self, src: &Path, obj: &Path, info: &BuildInfo, pch: &PreCompHead, verbose: bool, echo: bool) -> std::process::Command {
        let mut cmd = self.exe(info.lang);
        if self.0.is_msvc_compatible() {
            cmd::msvc::compiler_args(&mut cmd, src, &obj, &info, &pch, verbose)
        } else {
            cmd::gnu::compiler_args(&mut cmd, src, &obj, &info, &pch, verbose)
        };
        if echo {
            cmd::echo_command(&cmd);
        }
        cmd
    }

    pub fn output(self, output: &std::process::Output) -> bool {
        if let Toolchain::Msvc = self.0 {
            output::msvc::compiler(output)
        } else {
            output::gnu::compiler(output)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Linker(Toolchain);

impl Linker {
    fn exe(self, lang: Language) -> std::process::Command {
        match self.0 {
            Toolchain::Msvc => std::process::Command::new("LINK"),
            Toolchain::Gcc | Toolchain::Mingw => std::process::Command::new(if lang.is_cpp() { "g++" } else { "gcc" }),
            Toolchain::ClangMsvc => std::process::Command::new("lld-link"),
            Toolchain::ClangGcc => std::process::Command::new(if lang.is_cpp() { "clang++" } else { "clang" }),
            Toolchain::ClangMingw => {
                let mut cmd = std::process::Command::new(if lang.is_cpp() { "clang++" } else { "clang" });
                cmd.arg("--target=x86_64-w64-mingw32");
                cmd
            }
            Toolchain::Zig => {
                let mut cmd = std::process::Command::new("zig");
                cmd.arg(if lang.is_cpp() { "c++" } else { "cc" });
                cmd
            }
            Toolchain::Emcc => std::process::Command::new(if lang.is_cpp() { "em++.bat" } else { "emcc.bat" }),
        }
    }

    pub fn command(self, objs: Vec<PathBuf>, info: BuildInfo, verbose: bool, echo: bool) -> std::process::Command {
        // use g++/clang++ etc. when combining C and C++
        let mut cmd = self.exe(if info.cpprt { Language::Cpp(98) } else { info.lang });
        if self.0.is_msvc_compatible() {
            cmd::msvc::linker_args(&mut cmd, objs, info, verbose);
        } else {
            cmd::gnu::linker_args(&mut cmd, objs, info, verbose);
        };
        if echo {
            cmd::echo_command(&cmd);
        }
        cmd
    }

    pub fn output(self, output: &std::process::Output) -> bool {
        if self.0.is_msvc_compatible() {
            output::msvc::linker(output, self.0.is_llvm())
        } else {
            output::gnu::linker(output)
        }
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Archiver(Toolchain);

impl Archiver {
    fn exe(self) -> std::process::Command {
        match self.0 {
            Toolchain::Msvc => std::process::Command::new("LIB"),
            Toolchain::Gcc | Toolchain::Mingw => std::process::Command::new("ar"),
            Toolchain::ClangMsvc => std::process::Command::new("llvm-lib"),
            Toolchain::ClangGcc | Toolchain::ClangMingw | Toolchain::Emcc => std::process::Command::new("llvm-ar"),
            Toolchain::Zig => {
                let mut cmd = std::process::Command::new("zig");
                cmd.arg("ar");
                cmd
            }
        }
    }

    pub fn command(self, objs: Vec<PathBuf>, info: BuildInfo, verbose: bool, echo: bool) -> std::process::Command {
        let mut cmd = self.exe();
        if self.0.is_msvc_compatible() {
            cmd::msvc::archiver_args(&mut cmd, objs, info, verbose);
        } else {
            cmd::gnu::archiver_args(&mut cmd, objs, info, verbose);
        };
        if echo {
            cmd::echo_command(&cmd);
        }
        cmd
    }

    pub fn output(self, output: &std::process::Output) -> bool {
        if self.0.is_msvc_compatible() {
            output::msvc::archiver(output, self.0.is_llvm())
        } else {
            output::gnu::archiver(output)
        }
    }
}

impl FromStr for Toolchain {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "msvc"        => Ok(Toolchain::Msvc),
            "gcc"         => Ok(Toolchain::Gcc),
            "mingw"       => Ok(Toolchain::Mingw),
            "clang-msvc"  => Ok(Toolchain::ClangMsvc),
            "clang-gcc"   => Ok(Toolchain::ClangGcc),
            "clang-mingw" => Ok(Toolchain::ClangMingw),
            "zig"         => Ok(Toolchain::Zig),
            "emcc"        => Ok(Toolchain::Emcc),
            _ => Err(Error::UnknownToolchain(s.to_string())),
        }
    }
}

impl Display for Toolchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Msvc       => write!(f, "MSVC"),
            Self::Gcc        => write!(f, "GCC"),
            Self::Mingw      => write!(f, "MinGW"),
            Self::ClangMsvc  => write!(f, "Clang (MSVC backend)"),
            Self::ClangGcc   => write!(f, "Clang (GNU backend)"),
            Self::ClangMingw => write!(f, "Clang (MinGW backend)"),
            Self::Zig        => write!(f, "Zig"),
            Self::Emcc       => write!(f, "Emscripten"),
        }
    }
}

