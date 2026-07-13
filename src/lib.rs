pub mod core {
    pub mod tree;
    pub mod cat;
}
pub mod handler {
    pub mod error;
    pub mod config;
}
pub mod utils {
    pub mod ignore;
    pub mod fmt;
    pub mod io;
}
pub mod types;

use std::path::Path;
use rayon::prelude::*;
use tracing::{info, instrument};

use handler::config::{SnapConfig, OutputFormat};
use handler::error::SnapError;
use types::SnapOutput;
use utils::fmt;

#[instrument(skip(config))]
pub fn snap(root: &Path, config: &SnapConfig) -> Result<SnapOutput, SnapError> {
    let root = root.canonicalize().map_err(|_| SnapError::DirNotFound { path: root.to_path_buf() })?;
    info!("Snap dimulai di {:?}", root);

    // Walk & bangun tree
    let (_, file_entries, tree_str) = core::tree::walk_and_build(&root, config)?;

    // Tentukan thread pool
    let num_threads = config.jobs.unwrap_or_else(num_cpus::get);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| SnapError::InternalError(format!("Gagal membuat thread pool: {}", e)))?;

    // Baca isi file paralel + SHA
    let filled_entries = pool.install(|| {
        file_entries
            .par_iter()
            .filter_map(|entry| {
                match core::cat::read_file_content(&root, entry, config.max_file_size) {
                    Ok(e) => Some(e),
                    Err(err) => {
                        tracing::warn!("Melewatkan file {}: {}", entry.path, err);
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
        std::fs::write(out_path, &formatted)
            .map_err(|e| SnapError::OutputCreateError { path: out_path.clone(), source: e })?;
        info!("Output ditulis ke {:?}", out_path);
    } else {
        println!("{}", formatted);
    }
    Ok(())
}
