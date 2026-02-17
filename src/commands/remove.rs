use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn remove_dependency(package: &str) -> Result<()> {
    println!("{} Removing package {}", "🗑️".bold(), package.cyan());

    let manifest_path = Path::new("plugin.json5");

    if !manifest_path.exists() {
        anyhow::bail!("No plugin.json5 found. Are you in a Vayload project?");
    }

    let content = fs::read_to_string(manifest_path).context("Failed to read plugin.json5")?;
    let mut manifest: serde_json::Value = json5::from_str(&content).context("Failed to parse plugin.json5")?;

    let mut removed = false;

    if let Some(deps) = manifest.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        if deps.remove(package).is_some() {
            removed = true;
            println!("{} Removed from dependencies", "✓".green());
        }
    }

    if let Some(dev_deps) = manifest.get_mut("dev-dependencies").and_then(|d| d.as_object_mut()) {
        if dev_deps.remove(package).is_some() {
            removed = true;
            println!("{} Removed from dev-dependencies", "✓".green());
        }
    }

    if !removed {
        anyhow::bail!("Package {} not found in dependencies", package);
    }

    fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?).context("Failed to write plugin.json5")?;

    let cache_dir = Path::new(".vk").join("node_modules").join(package);
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir).ok();
        println!("{} Removed cached files", "✓".green());
    }

    println!("{} Package {} removed successfully!", "✅".green(), package.cyan());

    Ok(())
}
