pub mod core;
pub mod handler;
pub mod types;
pub mod utils;

use rayon::prelude::*;
use std::path::Path;
use tracing::{info, instrument};

use handler::config::{OutputFormat, SnapConfig};
use handler::error::SnapError;
use types::SnapOutput;
use utils::fmt;

#[instrument(skip(config))]
pub fn snap(root: &Path, config: &SnapConfig) -> Result<SnapOutput, SnapError> {
    let root = root.canonicalize().map_err(|_| SnapError::DirNotFound {
        path: root.to_path_buf(),
    })?;
    info!("Snap started at {:?}", root);

    let (tree_str, file_entries) = core::tree::walk_and_build(&root, config)?;

    let num_threads = config.jobs.unwrap_or_else(num_cpus::get);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| SnapError::InternalError(format!("Failed to create thread pool: {}", e)))?;

    let filled_entries = pool.install(|| {
        file_entries
            .par_iter()
            .filter_map(|entry| {
                match core::cat::read_file_content(&root, entry, config.max_file_size) {
                    Ok(e) => Some(e),
                    Err(err) => {
                        tracing::warn!("Skipping file {}: {}", entry.path, err);
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
    });

    Ok(SnapOutput {
        tree: tree_str,
        files: filled_entries,
    })
}

pub fn output(result: SnapOutput, config: &SnapConfig) -> Result<(), SnapError> {
    let formatted = match config.format {
        OutputFormat::Json => fmt::format_json(&result)?,
        OutputFormat::Markdown => fmt::format_markdown(&result)?,
    };

    if let Some(out_path) = &config.output {
        std::fs::write(out_path, &formatted).map_err(|e| SnapError::OutputCreateError {
            path: out_path.clone(),
            source: e,
        })?;
        info!("Output written to {:?}", out_path);
    } else {
        println!("{}", formatted);
    }
    Ok(())
}
