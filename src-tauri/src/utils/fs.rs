use std::path::{Path, PathBuf};

pub fn first_existing(paths: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    paths.into_iter().find(|path| path.exists())
}

pub fn path_contains(path: &Path, needle: &str) -> bool {
    std::fs::read_to_string(path)
        .map(|content| content.to_lowercase().contains(&needle.to_lowercase()))
        .unwrap_or(false)
}
