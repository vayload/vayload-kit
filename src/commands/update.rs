use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::encoding::json5;
use crate::http_client::HttpClient;
use crate::manifest::{MANIFEST_FILENAME, PluginManifest};
use crate::utils::parse_package;

pub fn update_dependencies(package: Option<&str>, http_client: &HttpClient) -> Result<()> {
    let manifest_path = Path::new(MANIFEST_FILENAME);

    let content = fs::read_to_string(manifest_path).context("Failed to read manifest file")?;
    let mut manifest: PluginManifest = json5::from_str(&content).context("Failed to parse manifest file")?;

    if let Some(pkg) = package {
        update_single_package(&mut manifest, pkg, http_client)?;
    } else {
        update_all_packages(&mut manifest, http_client)?;
    }

    fs::write(manifest_path, json5::to_string_pretty(&manifest)?).context("Failed to write manifest file")?;

    println!("{} Dependencies updated successfully!", "✅".green());

    Ok(())
}

fn update_single_package(manifest: &mut PluginManifest, package: &str, http_client: &HttpClient) -> Result<()> {
    let (id, _) = parse_package(package);

    println!("{} Updating {}", "🔄".bold(), id.cyan());

    let latest = fetch_latest_version(&id, http_client)?;

    let mut updated = false;

    // ---- dependencies ----
    if let Some(old_version) = manifest.dependencies.get_mut(&id) {
        let previous = old_version.clone();
        *old_version = latest.clone();

        println!(
            "{} {}: {} -> {}",
            "✓".green(),
            id.cyan(),
            previous.yellow(),
            latest.green()
        );

        updated = true;
    }

    // ---- dev_dependencies ----
    #[allow(clippy::collapsible_if)]
    if let Some(dev_deps) = manifest.dev_dependencies.as_mut() {
        if let Some(old_version) = dev_deps.get_mut(&id) {
            let previous = old_version.clone();
            *old_version = latest.clone();

            println!(
                "{} {} (dev): {} -> {}",
                "✓".green(),
                id.cyan(),
                previous.yellow(),
                latest.green()
            );

            updated = true;
        }
    }

    if !updated {
        anyhow::bail!("Package {} not found in dependencies", id);
    }

    Ok(())
}

fn update_all_packages(manifest: &mut PluginManifest, http_client: &HttpClient) -> Result<()> {
    println!("{} Updating all dependencies...", "🔄".bold());

    for (pkg, version) in manifest.dependencies.iter_mut() {
        update_version(pkg, version, http_client)?;
    }

    if let Some(dev_deps) = manifest.dev_dependencies.as_mut() {
        for (pkg, version) in dev_deps.iter_mut() {
            update_version(pkg, version, http_client)?;
        }
    }

    Ok(())
}

fn update_version(pkg: &str, version: &mut String, http_client: &HttpClient) -> Result<()> {
    let current = version.clone();

    if current == "*" {
        return Ok(());
    }

    match fetch_latest_version(pkg, http_client) {
        Ok(latest) => {
            if current != latest {
                *version = latest.clone();

                println!(
                    "{} {}: {} -> {}",
                    "✓".green(),
                    pkg.cyan(),
                    current.yellow(),
                    latest.green()
                );
            } else {
                println!("{} {}: already at latest", "-".yellow(), pkg.cyan());
            }
        },
        Err(_) => {
            println!("{} {}: could not fetch latest version", "⚠".yellow(), pkg.cyan());
        },
    }

    Ok(())
}

fn fetch_latest_version(id: &str, http_client: &HttpClient) -> Result<String> {
    #[derive(serde::Deserialize)]
    struct PackageInfo {
        #[serde(rename = "latestVersion")]
        latest_version: String,
    }

    let info = http_client.get::<PackageInfo>(&format!("/packages/{}", id))?;
    Ok(info.latest_version)
}
