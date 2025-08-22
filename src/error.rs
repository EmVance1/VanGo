use thiserror::Error;
use std::path::PathBuf;


#[derive(Debug, Error)]
pub enum Error {
    #[error("action '{0}' is not valid - see 'help' for list of actions")]
    BadAction(String),
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
    #[error("json error: {0}")]
    JsonParse(#[from] serde_json::Error),
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
    #[error("toolchain '{0}' compiler is unavailable")]
    MissingCompiler(String),
    #[error("toolchain '{0}' archiver is unavailable")]
    MissingArchiver(String),
    #[error("toolchain '{0}' linker is unavailable")]
    MissingLinker(String),
    #[error("failed to compile project '{0}'")]
    CompilerFail(PathBuf),
    #[error("failed to link library '{0}'")]
    ArchiverFail(PathBuf),
    #[error("failed to link application '{0}'")]
    LinkerFail(PathBuf),
    #[error("project does not contain 'test' directory")]
    MissingTests,
    #[error("binary '{0}' is not runnable on current platform")]
    InvalidExe(PathBuf),
    #[error("static library '{0}' is not executable")]
    LibNotExe(PathBuf),
    #[error("OS or IO error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("build failed")]
    #[allow(unused)]
    Unknown,
}

