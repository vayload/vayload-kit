use std::path::{Path, PathBuf};

pub fn parse_package(spec: &str) -> (String, Option<String>) {
    match spec.split_once('@') {
        Some((id, version)) => (id.to_string(), Some(version.to_string())),
        None => (spec.to_string(), None),
    }
}

pub fn get_directory(directory: &Option<String>) -> Result<PathBuf, String> {
    if let Some(dir) = directory {
        Ok(Path::new(dir).to_path_buf())
    } else {
        std::env::current_dir().map_err(|e| format!("Failed to get directory: {}", e))
    }
}
