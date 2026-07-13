use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::types::{TreeNode, NodeKind, FileEntry};
use crate::handler::error::SnapError;
use crate::handler::config::SnapConfig;
use tracing::{trace, info, instrument};

#[instrument(skip(root, config))]
pub fn walk_and_build(
    root: &Path,
    config: &SnapConfig,
) -> Result<(TreeNode, Vec<FileEntry>, String), SnapError> {
    let root = root
        .canonicalize()
        .map_err(|_| SnapError::DirNotFound {
            path: root.to_path_buf(),
        })?;
    info!("Memulai walk dari {:?}", root);

    // Bangun walker dari config (gunakan crate::utils::ignore agar konsisten)
    let walker = crate::utils::ignore::build_walker(&root, config);

    let mut entries: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for result in walker.build() {
        let entry = result
            .map_err(|e| SnapError::InternalError(format!("Walk error: {}", e)))?;
        let path = entry.path().to_path_buf();
        if path == root {
            continue;
        }
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| SnapError::InternalError("Gagal strip prefix".into()))?
            .to_path_buf();

        // Clone untuk logging sebelum relative dipindahkan
        let relative_for_log = relative.clone();

        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            // Daftarkan direktori sebagai key (agar direktori kosong tetap muncul)
            entries.entry(relative.clone()).or_default();
            let parent = relative
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::new());
            entries.entry(parent).or_default().push(relative);
        } else if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            files.push(relative);
        }
        trace!("Diproses: {:?}", relative_for_log);
    }

    // Bangun TreeNode rekursif (tanpa unwrap)
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
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| SnapError::InternalError("Invalid file name".into()))?
                    .to_string();
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
        .and_then(|s| s.to_str())
        .ok_or_else(|| SnapError::InternalError("Root tanpa nama".into()))?
        .to_string();

    let tree_root = build_node(root_name, &entries, Path::new(""))?;

    // Render tree string
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

    let tree_str = if tree_root.children.is_empty() {
        format!("{}\n", tree_root.name)
    } else {
        render_tree(&tree_root, String::new(), true)
    };

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
