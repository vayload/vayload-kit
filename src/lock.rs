use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const LOCK_FILENAME: &str = "plugin.lock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: u32,
    pub packages: HashMap<String, LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub integrity: String,
    #[serde(rename = "type")]
    pub dep_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    #[serde(rename = "production")]
    Production,
    #[serde(rename = "development")]
    Development,
}

impl Default for LockFile {
    fn default() -> Self {
        Self { version: 1, packages: HashMap::new() }
    }
}

#[allow(unused)]
impl LockFile {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let lock: LockFile = serde_json::from_str(&content)?;
        Ok(lock)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_package(&mut self, name: &str, version: &str, integrity: &str, is_dev: bool) {
        let key = format!("{}@{}", name, version);
        let pkg = LockedPackage {
            integrity: integrity.to_string(),
            dep_type: if is_dev {
                DependencyType::Development
            } else {
                DependencyType::Production
            },
        };
        self.packages.insert(key, pkg);
    }

    pub fn remove_package(&mut self, name: &str, version: &str) {
        let key = format!("{}@{}", name, version);
        self.packages.remove(&key);
    }

    pub fn get_package(&self, name: &str, version: &str) -> Option<&LockedPackage> {
        let key = format!("{}@{}", name, version);
        self.packages.get(&key)
    }

    pub fn has_package(&self, name: &str, version: &str) -> bool {
        let key = format!("{}@{}", name, version);
        self.packages.contains_key(&key)
    }

    pub fn get_integrity(&self, name: &str, version: &str) -> Option<&str> {
        self.get_package(name, version).map(|p| p.integrity.as_str())
    }

    pub fn get_any_version(&self, name: &str) -> Option<String> {
        for key in self.packages.keys() {
            if let Some(version) = key.strip_prefix(&format!("{}@", name)) {
                return Some(version.to_string());
            }
        }
        None
    }

    pub fn list_packages(&self) -> Vec<(String, String, &LockedPackage)> {
        self.packages
            .iter()
            .filter_map(|(key, pkg)| {
                let parts: Vec<&str> = key.split('@').collect();
                if parts.len() >= 2 {
                    let name = parts[..parts.len() - 1].join("@");
                    let version = parts[parts.len() - 1].to_string();
                    Some((name, version, pkg))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_file_default() {
        let lock = LockFile::default();
        assert_eq!(lock.version, 1);
        assert!(lock.packages.is_empty());
    }

    #[test]
    fn test_add_package() {
        let mut lock = LockFile::default();
        lock.add_package("test-pkg", "1.0.0", "sha256-abc123", false);

        assert!(lock.has_package("test-pkg", "1.0.0"));
        assert_eq!(lock.get_integrity("test-pkg", "1.0.0"), Some("sha256-abc123"));
    }

    #[test]
    fn test_add_dev_package() {
        let mut lock = LockFile::default();
        lock.add_package("test-pkg", "1.0.0", "sha256-abc123", true);

        let pkg = lock.get_package("test-pkg", "1.0.0").unwrap();
        assert_eq!(pkg.dep_type, DependencyType::Development);
    }

    #[test]
    fn test_remove_package() {
        let mut lock = LockFile::default();
        lock.add_package("test-pkg", "1.0.0", "sha256-abc123", false);
        lock.remove_package("test-pkg", "1.0.0");

        assert!(!lock.has_package("test-pkg", "1.0.0"));
    }

    #[test]
    fn test_get_any_version() {
        let mut lock = LockFile::default();
        lock.add_package("test-pkg", "1.0.0", "sha256-abc", false);
        lock.add_package("test-pkg", "2.0.0", "sha256-def", false);

        let version = lock.get_any_version("test-pkg");
        assert!(version.is_some());
    }

    #[test]
    fn test_list_packages() {
        let mut lock = LockFile::default();
        lock.add_package("pkg-a", "1.0.0", "sha256-aaa", false);
        lock.add_package("pkg-b", "2.0.0", "sha256-bbb", true);

        let packages = lock.list_packages();
        assert_eq!(packages.len(), 2);
    }
}
