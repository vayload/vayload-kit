use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::dependency::DependencyInstaller;
use crate::http_client::HttpClient;
use crate::manifest::PluginManifest;

const DEFAULT_DEPS_DIR: &str = ".deps";

pub fn install_all(deps_dir: Option<&str>, include_dev: bool, http_client: &HttpClient) -> Result<()> {
    let manifest = PluginManifest::load().map_err(|e| anyhow::anyhow!(e))?;

    // Support for installing dependencies from a custom directory
    let deps_path = Path::new(deps_dir.unwrap_or(DEFAULT_DEPS_DIR));

    println!("{} Installing dependencies...", "📦".bold());

    let mut installer = DependencyInstaller::new(http_client.clone(), Some(deps_path))?;
    installer.install_all(&manifest, include_dev)?;

    Ok(())
}
