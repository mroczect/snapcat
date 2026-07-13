use crate::handler::error::SnapError;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
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
    pub fn load(
        cli_format: Option<OutputFormat>,
        cli_output: Option<PathBuf>,
        cli_verbose: bool,
        cli_max_depth: Option<usize>,
        cli_include_hidden: bool,
        cli_follow_symlinks: bool,
        cli_jobs: Option<usize>,
        cli_max_file_size: Option<u64>,
    ) -> Result<Self, SnapError> {
        let config = Config::builder()
            .add_source(File::with_name(".snapcatconfig").required(false))
            .add_source(Environment::with_prefix("SNAPCAT").separator("__"))
            .build()?;

        let mut cfg: SnapConfig = config.try_deserialize()?;

        if let Some(f) = cli_format {
            cfg.format = f;
        }
        if let Some(o) = cli_output {
            cfg.output = Some(o);
        }
        if cli_verbose {
            cfg.verbose = true;
        }
        if let Some(d) = cli_max_depth {
            cfg.max_depth = Some(d);
        }
        if cli_include_hidden {
            cfg.include_hidden = true;
        }
        if cli_follow_symlinks {
            cfg.follow_symlinks = true;
        }
        if let Some(j) = cli_jobs {
            cfg.jobs = Some(j);
        }
        if let Some(s) = cli_max_file_size {
            cfg.max_file_size = Some(s);
        }

        if cfg.ignore_files.is_empty() {
            cfg.ignore_files.push(".snapcatignore".into());
        }

        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<(), SnapError> {
        if let Some(0) = self.max_depth {
            return Err(SnapError::InvalidConfig {
                msg: "max_depth must not be 0. Use None for unlimited.".into(),
            });
        }
        if let Some(0) = self.max_file_size {
            return Err(SnapError::InvalidConfig {
                msg: "max_file_size must not be 0. Use None for unlimited.".into(),
            });
        }
        if let Some(0) = self.jobs {
            return Err(SnapError::InvalidConfig {
                msg: "jobs must not be 0. Use None or a value >= 1.".into(),
            });
        }
        Ok(())
    }
}
