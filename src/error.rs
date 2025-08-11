use thiserror::Error;


#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid action '{0}' provided")]
    BadAction(String),
    #[error("toolchain 'MSVC' unavailable on non-windows platforms")]
    MsvcUnavailable,
    #[error("unknown toolchain argument '{0}'")]
    UnknownToolChain(String),
    #[error("unexpected arguments provided to '{0}' action: '{1:?}'")]
    ExtraArgs(String, Vec<String>),
    #[error("file '{0}' not found")]
    FileNotFound(String),
    #[error("directory '{0}' not found")]
    #[allow(unused)]
    DirNotFound(String),
    #[error("parse json error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("'{0}' is not a valid C++ standard")]
    InvalidCppStd(String),
    #[error("library '{0}' C++ standard incompatible with current project")]
    IncompatibleCppStd(String),
    #[error("library '{0}' does not have config '{1}'")]
    ConfigUnavailable(String, String),
    #[error("compiler unavailable for current toolchain: '{0}'")]
    MissingCompiler(String),
    #[error("failed to compile project '{0}'")]
    CompilerFail(String),
    #[error("failed to link application '{0}'")]
    LinkerFail(String),
    #[error("failed to link library '{0}'")]
    ArchiverFail(String),
    #[error("missing  'test' directory in this project")]
    MissingTests,
    #[error("filesystem error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("build failed")]
    #[allow(unused)]
    Unknown,
}

