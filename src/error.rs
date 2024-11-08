use thiserror::Error;


#[derive(Debug, Error)]
pub enum Error {
    #[error("file '{0}' not found")]
    FileNotFound(String),
    #[error("parse json error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("library '{0}' C++ version incompatible with current project")]
    IncompatibleCpp(String),
    #[error("library '{0}' does not have config '{1}'")]
    ConfigUnavailable(String, String),
    #[error("no program entry point 'main.cpp' or 'lib.cpp' found")]
    MissingEntryPoint,
    #[error("failed to compile file '{0}'")]
    CompilerFail(String),
    #[error("failed to link project '{0}'")]
    LinkerFail(String),
    #[error("build failed")]
    #[allow(unused)]
    Unknown
}

