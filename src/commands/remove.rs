use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::manifest::PluginManifest;

pub fn remove_dependency(package: &str) -> Result<()> {
    println!("{} Removing package {}", "🗑️".bold(), package.cyan());
    let mut manifest = PluginManifest::load().map_err(|e| anyhow::anyhow!(e))?;

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

    manifest.persist().map_err(|e| anyhow::anyhow!(e))?;

    // TODO: Remove package from cache directory, API is unstable
    let cache_dir = Path::new(".vk").join("modules").join(package);
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).ok();
        println!("{} Removed cached files", "✓".green());
    }

    println!("{} Package {} removed successfully!", "✅".green(), package.cyan());

    Ok(())
}
