use crate::handler::config::SnapConfig;
use ignore::WalkBuilder;
use std::path::Path;

pub fn build_walker(root: &Path, config: &SnapConfig) -> WalkBuilder {
    let mut builder = WalkBuilder::new(root);
    builder
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .hidden(!config.include_hidden)
        .follow_links(config.follow_symlinks);

    if let Some(depth) = config.max_depth {
        builder.max_depth(Some(depth)); // ✅ bungkus Some
    }

    for ign in &config.ignore_files {
        builder.add_custom_ignore_filename(ign);
    }
    builder.add_custom_ignore_filename(".snapcatignore");
    builder
}
