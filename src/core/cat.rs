use std::path::Path;
use sha2::{Sha256, Digest};
use crate::handler::error::SnapError;
use crate::types::FileEntry;
use tracing::{trace, instrument};

#[instrument]
pub fn read_file_content(
    root: &Path,
    entry: &FileEntry,
    max_size: Option<u64>,
) -> Result<FileEntry, SnapError> {
    let full_path = root.join(&entry.path);

    // Cek ukuran file jika ada batasan
    if let Some(max) = max_size {
        let metadata = std::fs::metadata(&full_path)
            .map_err(|e| SnapError::FileReadError {
                path: full_path.clone(),
                source: e,
            })?;
        if metadata.len() > max {
            return Err(SnapError::FileTooLarge {
                path: full_path.clone(),
                size: metadata.len(),
                max,
            });
        }
    }

    let content = std::fs::read_to_string(&full_path)
        .map_err(|e| SnapError::FileReadError {
            path: full_path.clone(),
            source: e,
        })?;

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let hash_bytes = hasher.finalize();
    // Konversi byte array ke string heksadesimal secara manual
    let hash = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    trace!("Baca {} => hash {}", entry.path, hash);
    Ok(FileEntry {
        path: entry.path.clone(),
        content,
        sha256: hash,
    })
}
