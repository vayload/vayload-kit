use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[allow(unused)]
const DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[cfg(debug_assertions)]
pub fn default_config_path() -> PathBuf {
    PathBuf::from("./config.toml")
}

#[cfg(not(debug_assertions))]
pub fn default_config_path() -> PathBuf {
    dirs::home_dir().expect("No home directory").join(".vayload-kit").join("config.toml")
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: AppServer,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppServer {
    pub registry_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        if let Ok(registry_url) = std::env::var("VK_REGISTRY_URL") {
            return Ok(AppConfig { server: AppServer { registry_url } });
        }

        #[cfg(feature = "full")]
        {
            let path = default_config_path();

            #[cfg(not(debug_assertions))]
            if !path.exists() {
                use std::fs;
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&path, DEFAULT_CONFIG)?;
                println!("Created default config at {:?}", path);
            }

            let settings = config::Config::builder().add_source(config::File::from(path)).build()?;

            Ok(settings.try_deserialize()?)
        }

        #[cfg(not(feature = "full"))]
        {
            let settings = config::Config::builder()
                .add_source(config::File::from_str(DEFAULT_CONFIG, config::FileFormat::Toml))
                .build()?;

            Ok(settings.try_deserialize()?)
        }
    }
}
