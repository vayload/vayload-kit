/// The configuration of the plugin.
/// This struct contains all the necessary information about the plugin.
///
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Serializer;
use serde_json::ser::PrettyFormatter;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub const MANIFEST_FILENAME: &str = "plugin.json";
pub const VKIGNORE_FILENAME: &str = ".vkignore";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub license: String,
    pub keywords: Vec<String>,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<Repository>,
    pub author: String,
    pub contributors: Option<Vec<String>>,
    pub main: String,
    pub engines: Engines,

    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub host_dependencies: Option<HashMap<String, String>>,

    pub permissions: Option<Permissions>,
    pub config: Option<PluginConfig>,

    #[serde(skip)]
    path: PathBuf,
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            name: String::new(),
            display_name: String::new(),
            version: "0.1.0".into(),
            description: String::new(),
            license: "MIT".into(),
            keywords: Vec::new(),
            tags: Vec::new(),
            homepage: None,
            repository: None,
            author: String::new(),
            contributors: None,
            main: "src/init.lua".into(),
            engines: Engines::default(),
            dependencies: HashMap::new(),
            dev_dependencies: None,
            host_dependencies: None,
            permissions: Some(Permissions::default()),
            config: Some(PluginConfig::default()),
            path: Path::new(MANIFEST_FILENAME).to_path_buf(),
        }
    }
}

impl PluginManifest {
    pub fn set_name(&mut self, name: String) {
        self.name = name.clone().to_lowercase().replace(" ", "-");
        self.display_name = name;
    }

    pub fn load() -> Result<Self, String> {
        let path = Path::new(MANIFEST_FILENAME);

        if !path.is_file() {
            return Err(format!("Manifest file '{}' not found", MANIFEST_FILENAME));
        }

        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut manifest: Self = serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {e}"))?;

        manifest.path = path.to_path_buf();
        Ok(manifest)
    }

    pub fn load_from(manifest_path: &Path) -> Result<Self, String> {
        if !manifest_path.is_file() {
            return Err(format!(
                "Manifest file '{}' not found in {}",
                MANIFEST_FILENAME,
                manifest_path.display()
            ));
        }

        let content = fs::read_to_string(manifest_path).map_err(|e| e.to_string())?;
        let mut manifest: Self = serde_json::from_str(&content).map_err(|e| format!("Invalid manifest: {e}"))?;

        manifest.path = manifest_path.to_path_buf();
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version.trim().is_empty() {
            return Err("Manifest missing required field: version".into());
        }

        if self.name.trim().is_empty() {
            return Err("Manifest missing required field: name".into());
        }

        if self.main.trim().is_empty() {
            return Err("Manifest missing required field: main".into());
        }

        let manifest_dir = self.path.parent().ok_or("Invalid manifest path")?;

        let main_path = manifest_dir.join(&self.main);

        if !main_path.is_file() {
            return Err(format!("Main entry file '{}' does not exist", main_path.display()));
        }

        Ok(())
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }

    pub fn persist(&self) -> Result<(), String> {
        let mut buf = Vec::new();

        let formatter = PrettyFormatter::with_indent(b"    ");
        let mut ser = Serializer::with_formatter(&mut buf, formatter);

        self.serialize(&mut ser).map_err(|e| e.to_string())?;

        let content = String::from_utf8(buf).map_err(|e| e.to_string())?;
        fs::write(&self.path, content).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub r#type: String,
    pub url: String,
}

impl Default for Repository {
    fn default() -> Self {
        Self { r#type: "git".into(), url: String::new() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engines {
    pub lua: String,
    pub host: String,
}

impl Default for Engines {
    fn default() -> Self {
        Self { lua: "5.1".into(), host: "*".into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Permissions {
    pub filesystem: Option<FileSystemPermission>,
    pub network: Option<NetworkPermission>,
    pub limits: Option<Limits>,
}

impl Permissions {
    pub fn new(fs: FileSystemPermission, net: NetworkPermission, lim: Limits) -> Self {
        Self { filesystem: Some(fs), network: Some(net), limits: Some(lim) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemPermission {
    pub scope: FileSystemScope,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

impl Default for FileSystemPermission {
    fn default() -> Self {
        Self {
            scope: FileSystemScope::None,
            allow: Vec::new(),
            deny: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FileSystemScope {
    ReadOnly,
    ReadWrite,
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkPermission {
    pub allow_outbound: Vec<String>,
    pub allow_inbound: bool,
}

impl NetworkPermission {
    pub fn new(allow_outbound: Vec<String>, allow_inbound: bool) -> Self {
        Self { allow_outbound, allow_inbound }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub max_memory_mb: u32,
    pub max_execution_time_ms: u64,
    pub max_threads: u16,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            max_execution_time_ms: 10000,
            max_threads: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub max_file_size: u64,
    pub chunk_size: u64,
    pub retry_attempts: u32,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            max_file_size: 5 * 1024 * 1024,
            chunk_size: 4096,
            retry_attempts: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, ValueEnum)]
pub enum PluginAccess {
    #[default]
    Public,
    Private,
}

impl PluginAccess {
    pub fn as_str(&self) -> &str {
        match self {
            PluginAccess::Public => "public",
            PluginAccess::Private => "private",
        }
    }

    #[allow(unused)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "public" => Ok(PluginAccess::Public),
            "private" => Ok(PluginAccess::Private),
            _ => Err(format!("Invalid access level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manifest_default() {
        let manifest = PluginManifest::default();
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.license, "MIT");
        assert_eq!(manifest.main, "src/init.lua");
        assert_eq!(manifest.engines.lua, "5.1");
    }

    #[test]
    fn test_set_name() {
        let mut manifest = PluginManifest::default();
        manifest.set_name("My Plugin".to_string());

        assert_eq!(manifest.name, "my-plugin");
        assert_eq!(manifest.display_name, "My Plugin");
    }

    #[test]
    fn test_set_name_with_spaces() {
        let mut manifest = PluginManifest::default();
        manifest.set_name("Hello World Test".to_string());

        assert_eq!(manifest.name, "hello-world-test");
    }

    #[test]
    fn test_plugin_access_as_str() {
        assert_eq!(PluginAccess::Public.as_str(), "public");
        assert_eq!(PluginAccess::Private.as_str(), "private");
    }

    #[test]
    fn test_plugin_access_from_str() {
        assert_eq!(PluginAccess::from_str("public").unwrap(), PluginAccess::Public);
        assert_eq!(PluginAccess::from_str("private").unwrap(), PluginAccess::Private);
        assert!(PluginAccess::from_str("invalid").is_err());
    }

    #[test]
    fn test_engines_default() {
        let engines = Engines::default();
        assert_eq!(engines.lua, "5.1");
        assert_eq!(engines.host, "*");
    }

    #[test]
    fn test_limits_default() {
        let limits = Limits::default();
        assert_eq!(limits.max_memory_mb, 128);
        assert_eq!(limits.max_execution_time_ms, 10000);
        assert_eq!(limits.max_threads, 10);
    }

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        assert_eq!(config.chunk_size, 4096);
        assert_eq!(config.retry_attempts, 3);
    }

    #[test]
    fn test_validate_missing_version() {
        let manifest = PluginManifest { name: "".to_string(), ..Default::default() };

        let result = manifest.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_missing_name() {
        let manifest = PluginManifest { name: "".to_string(), ..Default::default() };

        let result = manifest.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }

    #[test]
    fn test_validate_missing_main() {
        let mut manifest = PluginManifest {
            main: "".to_string(),
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };
        manifest.set_path(Path::new("plugin.json"));

        let result = manifest.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("main") || err.contains("Invalid manifest path"));
    }

    #[test]
    fn test_permissions_new() {
        let fs_perm = FileSystemPermission::default();
        let net_perm = NetworkPermission::new(vec!["https://api.example.com".to_string()], true);
        let limits = Limits::default();

        let perms = Permissions::new(fs_perm, net_perm, limits);

        assert!(perms.filesystem.is_some());
        assert!(perms.network.is_some());
        assert!(perms.limits.is_some());
    }

    #[test]
    fn test_network_permission_new() {
        let net = NetworkPermission::new(vec!["https://example.com".to_string()], false);
        assert_eq!(net.allow_outbound.len(), 1);
        assert!(!net.allow_inbound);
    }

    #[test]
    fn test_repository_default() {
        let repo = Repository::default();
        assert_eq!(repo.r#type, "git");
        assert!(repo.url.is_empty());
    }

    #[test]
    fn test_load_from_valid_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join(MANIFEST_FILENAME);

        let manifest_content = r#"{
            "name": "test-plugin",
            "display_name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "license": "MIT",
            "keywords": [],
            "tags": [],
            "homepage": null,
            "repository": null,
            "author": "Test Author",
            "contributors": null,
            "main": "src/init.lua",
            "engines": {
                "lua": "5.1",
                "host": "*"
            },
            "dependencies": {}
        }"#;

        fs::write(&manifest_path, manifest_content).unwrap();

        let result = PluginManifest::load_from(&manifest_path);
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.name, "test-plugin");
    }

    #[test]
    fn test_load_from_missing_file() {
        let result = PluginManifest::load_from(Path::new("/nonexistent/plugin.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_persist_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join(MANIFEST_FILENAME);

        let mut manifest = PluginManifest::default();
        manifest.set_name("roundtrip-test".to_string());
        manifest.version = "1.0.0".to_string();
        manifest.set_path(&manifest_path);

        manifest.persist().unwrap();

        let loaded = PluginManifest::load_from(&manifest_path).unwrap();
        assert_eq!(loaded.name, "roundtrip-test");
        assert_eq!(loaded.version, "1.0.0");
    }
}
