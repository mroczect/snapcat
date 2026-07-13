use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum SnapError {
    #[error("Gagal membaca file ignore '{path}'")]
    #[diagnostic(code(snapcat::ignore::read), help("pastikan file ada dan dapat dibaca"))]
    IgnoreFileRead {
        path: String,
        #[source]
        source: std::io::Error,
        #[label("terjadi saat membuka file ini")]
        loc: SourceSpan,
    },

    #[error("Gagal membaca file '{path}'")]
    #[diagnostic(code(snapcat::io::read))]
    FileReadError {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("File terlalu besar '{path}': {size} bytes (max: {max})")]
    #[diagnostic(code(snapcat::io::file_too_large))]
    FileTooLarge {
        path: std::path::PathBuf,
        size: u64,
        max: u64,
    },

    #[error("Direktori '{path}' tidak ditemukan")]
    #[diagnostic(code(snapcat::tree::not_found))]
    DirNotFound {
        path: std::path::PathBuf,
    },

    #[error("Direktori '{path}' bukan sebuah direktori")]
    #[diagnostic(code(snapcat::tree::not_a_directory))]
    NotADirectory {
        path: std::path::PathBuf,
    },

    #[error("Gagal membuat output file '{path}'")]
    #[diagnostic(code(snapcat::output::create))]
    OutputCreateError {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Format output tidak didukung: {0}")]
    #[diagnostic(code(snapcat::format::unsupported))]
    UnsupportedFormat(String),

    #[error("Konfigurasi error: {msg}")]
    #[diagnostic(code(snapcat::config))]
    ConfigError { msg: String },

    #[error("Gagal memformat output: {msg}")]
    #[diagnostic(code(snapcat::format::internal))]
    FormatError { msg: String },

    #[error("Internal error: {0}")]  // ✅ perbaiki: pakai {0}
    #[diagnostic(code(snapcat::internal))]
    InternalError(String),
}
