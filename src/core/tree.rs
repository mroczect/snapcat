use crate::handler::config::SnapConfig;
use crate::handler::error::SnapError;
use crate::types::{FileEntry, NodeKind, TreeNode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, instrument, trace};

#[instrument(skip(root, config))]
pub fn walk_and_build(
    root: &Path,
    config: &SnapConfig,
) -> Result<(TreeNode, Vec<FileEntry>, String), SnapError> {
    let root = root.canonicalize().map_err(|_| SnapError::DirNotFound {
        path: root.to_path_buf(),
    })?;
    info!("Starting walk from {:?}", root);

    let walker = crate::utils::ignore::build_walker(&root, config);

    let mut entries: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for result in walker.build() {
        let entry = result.map_err(|e| SnapError::InternalError(format!("Walk error: {}", e)))?;
        let path = entry.path().to_path_buf();
        if path == root {
            continue;
        }
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| SnapError::InternalError("Failed to strip prefix".into()))?
            .to_path_buf();

        let relative_for_log = relative.clone();

        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            entries.entry(relative.clone()).or_default();
            let parent = relative
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::new());
            entries.entry(parent).or_default().push(relative);
        } else if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            files.push(relative);
        }
        trace!("Processed: {:?}", relative_for_log);
    }

    for children in entries.values_mut() {
        children.sort();
    }
    files.sort();

    fn build_node(
        name: String,
        children_map: &HashMap<PathBuf, Vec<PathBuf>>,
        base: &Path,
    ) -> Result<TreeNode, SnapError> {
        let mut children = Vec::new();
        if let Some(child_paths) = children_map.get(base) {
            for child in child_paths {
                let child_name = child
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "??".to_string());
                let child_full = base.join(child);
                if children_map.contains_key(&child_full) {
                    children.push(build_node(child_name, children_map, &child_full)?);
                } else {
                    children.push(TreeNode {
                        name: child_name,
                        kind: NodeKind::File,
                        children: Vec::new(),
                    });
                }
            }
        }
        Ok(TreeNode {
            name,
            kind: NodeKind::Directory,
            children,
        })
    }

    let root_name = root
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".to_string());

    let tree_root = build_node(root_name, &entries, Path::new(""))?;

    fn render_tree(node: &TreeNode, prefix: String, is_last: bool) -> String {
        let mut out = String::new();
        let connector = if is_last { "└── " } else { "├── " };
        out.push_str(&format!("{}{}{}\n", prefix, connector, node.name));
        let child_count = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            out.push_str(&render_tree(child, child_prefix, i == child_count - 1));
        }
        out
    }

    let mut tree_str = if tree_root.children.is_empty() {
        format!("{}\n", tree_root.name)
    } else {
        render_tree(&tree_root, String::new(), true)
    };

    let dir_count = count_dirs(&tree_root);
    let file_count = files.len();
    tree_str.push_str(&format!(
        "{} directories, {} files\n",
        dir_count, file_count
    ));

    let file_entries = files
        .into_iter()
        .map(|p| FileEntry {
            path: p.to_string_lossy().into_owned(),
            content: String::new(),
            sha256: String::new(),
        })
        .collect();

    Ok((tree_root, file_entries, tree_str))
}

fn count_dirs(node: &TreeNode) -> usize {
    let mut count = 0;
    for child in &node.children {
        if child.kind == NodeKind::Directory {
            count += 1;
            count += count_dirs(child);
        }
    }
    count
}
