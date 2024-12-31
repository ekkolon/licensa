use std::path::PathBuf;

pub fn sort_paths(paths: &mut Vec<PathBuf>) {
    paths.sort_by(|a, b| a.to_str().unwrap_or("").cmp(b.to_str().unwrap_or("")))
}
