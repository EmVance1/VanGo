use thiserror::Error;
use std::path::PathBuf;


#[derive(Debug, Error)]
pub enum Error {
    #[error("action '{0}' is not valid - see 'help' for list of actions")]
    BadAction(String),
    // #[error("not enough arguments provided to '{0}' action")]
    // MissingArgs(String),
    #[error("unexpected arguments provided to '{0}' action: '{1:?}'")]
    ExtraArgs(String, Vec<String>),
    #[error("toolchain 'MSVC' unavailable on non-windows platforms")]
    MsvcUnavailable,
    #[error("toolchain '{0}' is unavailable")]
    UnknownToolChain(String),
    #[error("directory '{0}' was not found")]
    DirectoryNotFound(PathBuf),
    #[error("directory '{0}' does not contain a build script")]
    MissingBuildScript(PathBuf),
    #[error("toml error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("'{0}' is not a valid C/C++ standard")]
    InvalidCppStd(String),
    #[error("library '{0}' C/C++ standard incompatible with current project")]
    IncompatibleCppStd(String),
    #[error("project '{0}' does not contain profile '{1}'")]
    ProfileUnavailable(String, String),
    #[error("custom profile '{0}' must inherit from a builtin profile")]
    InvalidCustomProfile(String),
    #[error("project dependency '{0}' is not a library")]
    InvalidDependency(String),
    #[error("toolchain '{0}' compiler is unavailable")]
    MissingCompiler(String),
    #[error("toolchain '{0}' archiver is unavailable")]
    MissingArchiver(String),
    #[error("toolchain '{0}' linker is unavailable")]
    MissingLinker(String),
    #[error("failed to compile project '{0}'")]
    CompilerFail(PathBuf),
    #[error("failed to archive project '{0}'")]
    ArchiverFail(PathBuf),
    #[error("failed to link project '{0}'")]
    LinkerFail(PathBuf),
    #[error("project does not contain 'test' directory")]
    MissingTests,
    #[error("binary '{0}' is not runnable on current platform")]
    InvalidExe(PathBuf),
    #[error("static library '{0}' is not executable")]
    LibNotExe(PathBuf),
    #[error("executable '{0}' was killed by the host OS (potential segfault)")]
    ExeKilled(PathBuf),
    #[error("OS or IO error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("build failed")]
    #[allow(unused)]
    Unknown,
}

