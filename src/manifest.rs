/// The configuration of the plugin.
/// This struct contains all the necessary information about the plugin.
///
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        }
    }
}

impl PluginManifest {
    pub fn set_name(&mut self, name: String) {
        self.name = name.clone().to_lowercase().replace(" ", "-");
        self.display_name = name;
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
