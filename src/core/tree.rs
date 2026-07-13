use crate::handler::config::SnapConfig;
use crate::handler::error::SnapError;
use crate::types::FileEntry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, instrument, trace};

#[instrument(skip(root, config))]
pub fn walk_and_build(
    root: &Path,
    config: &SnapConfig,
) -> Result<(String, Vec<FileEntry>), SnapError> {
    info!("Starting walk from {:?}", root);

    let walker = crate::utils::ignore::build_walker(root, config);

    let mut entries: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for result in walker.build() {
        let entry = result.map_err(|e| SnapError::InternalError(format!("Walk error: {}", e)))?;
        let path = entry.path().to_path_buf();
        if path == root {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| SnapError::InternalError("Failed to strip prefix".into()))?
            .to_path_buf();

        let parent = relative
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::new());
        entries.entry(parent).or_default().push(relative.clone());

        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            entries.entry(relative.clone()).or_default();
        } else {
            files.push(relative.clone());
        }
        trace!("Processed: {:?}", relative);
    }

    for children in entries.values_mut() {
        children.sort();
    }
    files.sort();

    let root_name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".to_string());

    let (tree_str, _dirs, _files) = render_tree_from_entries(Path::new(""), &entries, &root_name);

    let dir_count = count_dirs_from_entries(Path::new(""), &entries);
    let file_count = files.len();
    let summary = format!("\n{} directories, {} files", dir_count, file_count);
    let full_tree = format!("{}{}", tree_str, summary);

    let file_entries = files
        .into_iter()
        .map(|p| FileEntry {
            path: p.to_string_lossy().into_owned(),
            content: String::new(),
            sha256: String::new(),
        })
        .collect();

    Ok((full_tree, file_entries))
}

fn render_tree_from_entries(
    base: &Path,
    entries: &HashMap<PathBuf, Vec<PathBuf>>,
    name: &str,
) -> (String, usize, usize) {
    let children = entries.get(base);
    if children.is_none() || children.unwrap().is_empty() {
        return (format!("{}\n", name), 0, 0);
    }

    let children = children.unwrap();
    let child_count = children.len();

    let mut out = String::new();
    let mut total_dirs = 0;
    let mut total_files = 0;

    for (i, child_path) in children.iter().enumerate() {
        let is_last = i == child_count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_name = child_path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "??".to_string());

        let child_full = base.join(child_path);
        let (child_tree, child_dirs, child_files) =
            render_tree_from_entries(&child_full, entries, &child_name);

        out.push_str(&format!("{}{}{}", "    ", connector, child_name));
        if !child_tree.is_empty() {
            out.push_str(&child_tree);
        } else {
            out.push('\n');
        }

        if entries.contains_key(&child_full) {
            total_dirs += 1 + child_dirs;
            total_files += child_files;
        } else {
            total_files += 1;
        }
    }

    let root_line = format!("{}\n", name);
    (format!("{}{}", root_line, out), total_dirs, total_files)
}

fn count_dirs_from_entries(base: &Path, entries: &HashMap<PathBuf, Vec<PathBuf>>) -> usize {
    let mut count = 0;
    if let Some(children) = entries.get(base) {
        for child in children {
            let child_full = base.join(child);
            if entries.contains_key(&child_full) {
                count += 1 + count_dirs_from_entries(&child_full, entries);
            }
        }
    }
    count
}
