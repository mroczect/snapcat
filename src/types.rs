use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SnapOutput {
    pub tree: String,
    pub files: Vec<FileEntry>,
}


#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub content: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    pub name: String,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum NodeKind {
    Directory,
    File,
}
