use std::path::{Path, PathBuf};

pub fn parse_package(spec: &str) -> (String, Option<String>) {
    match spec.rsplit_once('@') {
        Some((name, version)) if !name.is_empty() => (name.to_string(), Some(version.to_string())),
        _ => (spec.to_string(), None),
    }
}

pub fn get_directory(directory: &Option<String>) -> Result<PathBuf, String> {
    if let Some(dir) = directory {
        Ok(Path::new(dir).to_path_buf())
    } else {
        std::env::current_dir().map_err(|e| format!("Failed to get directory: {}", e))
    }
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn build_user_agent() -> String {
    format!("{}/{}", APP_NAME, APP_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_with_version() {
        let (name, version) = parse_package("my-package@1.0.0");
        assert_eq!(name, "my-package");
        assert_eq!(version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_package_without_version() {
        let (name, version) = parse_package("my-package");
        assert_eq!(name, "my-package");
        assert_eq!(version, None);
    }

    #[test]
    fn test_build_user_agent_format() {
        let ua = build_user_agent();
        assert!(ua.contains('/'));
        assert!(ua.starts_with("vayload-kit"));
    }

    #[test]
    fn parses_plain_package() {
        let (name, version) = parse_package("pkg");

        assert_eq!(name, "pkg");
        assert_eq!(version, None);
    }

    #[test]
    fn parses_package_with_version() {
        let (name, version) = parse_package("pkg@1.0");

        assert_eq!(name, "pkg");
        assert_eq!(version, Some("1.0".to_string()));
    }

    #[test]
    fn parses_scoped_package() {
        let (name, version) = parse_package("@scope/pkg@2.0");

        assert_eq!(name, "@scope/pkg");
        assert_eq!(version, Some("2.0".to_string()));
    }
}
