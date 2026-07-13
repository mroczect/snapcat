use crate::handler::error::SnapError;
use crate::types::{FileEntry, SnapOutput};
use serde::Serialize;

#[derive(Serialize)]
struct JsonOutput {
    tree: String,
    files: Vec<FileEntry>,
}

pub fn format_json(output: &SnapOutput) -> Result<String, SnapError> {
    let json_out = JsonOutput {
        tree: output.tree.clone(),
        files: output.files.clone(),
    };
    serde_json::to_string_pretty(&json_out)
        .map_err(|e| SnapError::FormatError { msg: e.to_string() })
}

pub fn format_markdown(output: &SnapOutput) -> Result<String, SnapError> {
    let mut md = String::new();
    md.push_str("# Snapcat Output\n\n```\n");
    md.push_str(&output.tree);
    md.push_str("```\n\n## Files\n\n");
    for f in &output.files {
        md.push_str(&format!("### {}\n", f.path));
        md.push_str("```\n");
        md.push_str(&f.content);
        if !f.content.ends_with('\n') {
            md.push('\n');
        }
        md.push_str("```\n\n");
        md.push_str(&format!("SHA256: `{}`\n\n", f.sha256));
    }
    Ok(md)
}

// Tambahkan di error.rs:
// #[error("Gagal memformat output: {msg}")]
// FormatError { msg: String },
