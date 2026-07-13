use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum SnapError {
    #[error("Failed to read ignore file '{path}'")]
    #[diagnostic(code(snapcat::ignore::read), help("ensure the file exists and is readable"))]
    IgnoreFileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read file '{path}'")]
    #[diagnostic(code(snapcat::io::read))]
    FileReadError {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("File too large '{path}': {size} bytes (max: {max})")]
    #[diagnostic(code(snapcat::io::file_too_large))]
    FileTooLarge {
        path: std::path::PathBuf,
        size: u64,
        max: u64,
    },

    #[error("Directory '{path}' not found")]
    #[diagnostic(code(snapcat::tree::not_found))]
    DirNotFound {
        path: std::path::PathBuf,
    },

    #[error("Path '{path}' is not a directory")]
    #[diagnostic(code(snapcat::tree::not_a_directory))]
    NotADirectory {
        path: std::path::PathBuf,
    },

    #[error("Failed to create output file '{path}'")]
    #[diagnostic(code(snapcat::output::create))]
    OutputCreateError {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Unsupported output format: {0}")]
    #[diagnostic(code(snapcat::format::unsupported))]
    UnsupportedFormat(String),

    #[error("Failed to load configuration")]
    #[diagnostic(code(snapcat::config::load), help("Check .snapcatconfig or SNAPCAT_* environment variables."))]
    ConfigLoadError {
        #[source]
        source: config::ConfigError,
    },

    #[error("Invalid configuration: {msg}")]
    #[diagnostic(code(snapcat::config::invalid), help("Refer to the documentation for allowed values."))]
    InvalidConfig { msg: String },

    #[error("Failed to format output: {msg}")]
    #[diagnostic(code(snapcat::format::internal))]
    FormatError { msg: String },

    #[error("Internal error: {0}")]
    #[diagnostic(code(snapcat::internal))]
    InternalError(String),
}

impl From<config::ConfigError> for SnapError {
    fn from(e: config::ConfigError) -> Self {
        SnapError::ConfigLoadError { source: e }
    }
}
