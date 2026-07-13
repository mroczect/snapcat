use serde::{Deserialize, Serialize};
use config::{Config, File, Environment};
use std::path::PathBuf;
use crate::handler::error::SnapError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapConfig {
    pub format: OutputFormat,
    pub output: Option<PathBuf>,
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub include_hidden: bool,
    pub ignore_files: Vec<String>,
    pub max_file_size: Option<u64>,
    pub verbose: bool,
    pub jobs: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]  // 👈 tambahkan ValueEnum
pub enum OutputFormat {
    Json,
    Markdown,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Json,
            output: None,
            max_depth: None,
            follow_symlinks: false,
            include_hidden: false,
            ignore_files: vec![".snapcatignore".into()],
            max_file_size: Some(10_000_000),
            verbose: false,
            jobs: Some(num_cpus::get()),
        }
    }
}

impl SnapConfig {
    pub fn load(cli_format: Option<OutputFormat>, cli_output: Option<PathBuf>, cli_verbose: bool) -> Result<Self, SnapError> {
        let s = Config::builder()
            .add_source(File::with_name(".snapcatconfig").required(false))
            .add_source(Environment::with_prefix("SNAPCAT").separator("__"))
            .build()
            .map_err(|e| SnapError::ConfigError { msg: e.to_string() })?;

        let mut cfg: SnapConfig = s.try_deserialize()
            .map_err(|e| SnapError::ConfigError { msg: e.to_string() })?;

        if let Some(f) = cli_format { cfg.format = f; }
        if let Some(o) = cli_output { cfg.output = Some(o); }
        if cli_verbose { cfg.verbose = true; }

        Ok(cfg)
    }
}
