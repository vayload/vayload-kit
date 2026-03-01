use anyhow::Result;
use colored::Colorize;

use crate::http_client::HttpClient;
use crate::manifest::PluginManifest;
use crate::utils::parse_package;

// !TODO: missing update lock file
pub fn update_dependencies(package: Option<&str>, http_client: &HttpClient) -> Result<()> {
    let mut manifest = PluginManifest::load().map_err(|e| anyhow::anyhow!(e))?;

    if let Some(pkg) = package {
        update_single_package(&mut manifest, pkg, http_client)?;
    } else {
        update_all_packages(&mut manifest, http_client)?;
    }

    manifest.persist().map_err(|e| anyhow::anyhow!(e))?;

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
        latest_stable_version: String,
    }

    let info = http_client.get::<PackageInfo>(&format!("/{}/info", id))?;
    Ok(info.latest_stable_version)
}
