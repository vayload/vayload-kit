use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::http_client::HttpClient;
use crate::utils::parse_package;

pub fn update_dependencies(package: Option<&str>, http_client: &HttpClient) -> Result<()> {
    let manifest_path = Path::new("plugin.json5");

    if !manifest_path.exists() {
        anyhow::bail!("No plugin.json5 found. Are you in a Vayload project?");
    }

    let content = fs::read_to_string(manifest_path).context("Failed to read plugin.json5")?;
    let mut manifest: serde_json::Value = json5::from_str(&content).context("Failed to parse plugin.json5")?;

    if let Some(pkg) = package {
        update_single_package(&mut manifest, pkg, http_client)?;
    } else {
        update_all_packages(&mut manifest, http_client)?;
    }

    fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?).context("Failed to write plugin.json5")?;

    println!("{} Dependencies updated successfully!", "✅".green());

    Ok(())
}

fn update_single_package(manifest: &mut serde_json::Value, package: &str, http_client: &HttpClient) -> Result<()> {
    let (id, _) = parse_package(package);

    println!("{} Updating {}", "🔄".bold(), id.cyan());

    let latest = fetch_latest_version(&id, http_client)?;

    let mut updated = false;

    if let Some(deps) = manifest.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        if let Some(dep) = deps.get_mut(&id) {
            let old_version = dep.as_str().unwrap_or("*").to_string();
            *dep = serde_json::json!(latest.clone());
            println!(
                "{} {}: {} -> {}",
                "✓".green(),
                id.cyan(),
                old_version.yellow(),
                latest.green()
            );
            updated = true;
        }
    }

    if let Some(dev_deps) = manifest.get_mut("dev-dependencies").and_then(|d| d.as_object_mut()) {
        if let Some(dep) = dev_deps.get_mut(&id) {
            let old_version = dep.as_str().unwrap_or("*").to_string();
            *dep = serde_json::json!(latest.clone());
            println!(
                "{} {} (dev): {} -> {}",
                "✓".green(),
                id.cyan(),
                old_version.yellow(),
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

fn update_all_packages(manifest: &mut serde_json::Value, http_client: &HttpClient) -> Result<()> {
    println!("{} Updating all dependencies...", "🔄".bold());

    let deps_keys = ["dependencies", "dev-dependencies"];

    for key in deps_keys {
        if let Some(deps) = manifest.get_mut(key).and_then(|d| d.as_object_mut()) {
            let packages: Vec<String> = deps.keys().cloned().collect();

            for pkg in packages {
                if let Some(dep) = deps.get_mut(&pkg) {
                    let current_version = dep.as_str().unwrap_or("*").to_string();

                    if current_version != "*" {
                        match fetch_latest_version(&pkg, http_client) {
                            Ok(latest) => {
                                if current_version != latest {
                                    *dep = serde_json::json!(latest.clone());
                                    println!(
                                        "{} {}: {} -> {}",
                                        "✓".green(),
                                        pkg.cyan(),
                                        current_version.yellow(),
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
                    }
                }
            }
        }
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
