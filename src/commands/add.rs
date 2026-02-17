use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::http_client::HttpClient;
use crate::utils::parse_package;

pub fn add_dependency(package: &str, is_dev: bool, http_client: &HttpClient) -> Result<()> {
    let (id, version) = parse_package(package);

    print!("{} Adding {}", "📦".bold(), id.cyan());
    if let Some(v) = &version {
        print!("@{}", v.yellow());
    }
    if is_dev {
        print!(" as dev dependency");
    }
    println!();

    let manifest_path = Path::new("plugin.json5");

    if !manifest_path.exists() {
        println!("{} No plugin.json5 found. Running init...", "⚠".yellow());
        anyhow::bail!("Run 'vk init' first to create a project");
    }

    let content = fs::read_to_string(manifest_path).context("Failed to read plugin.json5")?;
    let mut manifest: serde_json::Value = json5::from_str(&content).context("Failed to parse plugin.json5")?;

    let deps_key = if is_dev { "dev-dependencies" } else { "dependencies" };
    if manifest.get(deps_key).is_none() {
        manifest[deps_key] = serde_json::json!({});
    }

    let deps = manifest.get_mut(deps_key).unwrap().as_object_mut().context("Failed to get dependencies object")?;

    if let Some(v) = version {
        deps.insert(id.clone(), serde_json::json!(v));
    } else {
        let latest_version = fetch_latest_version(&id, http_client)?;
        deps.insert(id.clone(), serde_json::json!(latest_version));
        println!("{} Latest version: {}", "📌".green(), latest_version.yellow());
    }

    fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?).context("Failed to write plugin.json5")?;

    println!("{} Added {} to {}", "✅".green(), id.cyan(), deps_key.green());

    Ok(())
}

fn fetch_latest_version(id: &str, http_client: &HttpClient) -> Result<String> {
    #[derive(serde::Deserialize)]
    struct PackageInfo {
        #[serde(rename = "latestVersion")]
        latest_version: String,
    }

    match http_client.get::<PackageInfo>(&format!("/packages/{}", id)) {
        Ok(info) => Ok(info.latest_version),
        Err(_) => Ok("*".to_string()),
    }
}
