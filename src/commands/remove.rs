use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::{
    encoding::json5,
    manifest::{MANIFEST_FILENAME, PluginManifest},
};

pub fn remove_dependency(package: &str) -> Result<()> {
    let manifest_path = Path::new(MANIFEST_FILENAME);

    println!("{} Removing package {}", "🗑️".bold(), package.cyan());
    let content = fs::read_to_string(manifest_path).context("Failed to read manifest file")?;
    let mut manifest: PluginManifest = json5::from_str(&content).context("Failed to parse manifest file")?;

    let mut removed = false;

    if manifest.dependencies.remove(package).is_some() {
        removed = true;
        println!("{} Removed from dependencies", "✓".green());
    }

    #[allow(clippy::collapsible_if)]
    if let Some(deps) = manifest.dev_dependencies.as_mut() {
        if deps.remove(package).is_some() {
            removed = true;
            println!("{} Removed from dev-dependencies", "✓".green());
        }
    }

    if !removed {
        anyhow::bail!("Package {} not found in dependencies", package);
    }

    fs::write(manifest_path, json5::to_string_pretty(&manifest)?).context("Failed to write manifest file")?;

    // TODO: Remove package from cache directory, API is unstable
    let cache_dir = Path::new(".vk").join("modules").join(package);
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).ok();
        println!("{} Removed cached files", "✓".green());
    }

    println!("{} Package {} removed successfully!", "✅".green(), package.cyan());

    Ok(())
}
