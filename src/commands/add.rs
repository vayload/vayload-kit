use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::encoding::json5;
use crate::http_client::HttpClient;
use crate::manifest::{MANIFEST_FILENAME, PluginManifest};
use crate::utils::parse_package;

pub fn add_dependency(package: &str, is_dev: bool, http_client: &HttpClient) -> Result<()> {
    let manifest_path = Path::new(MANIFEST_FILENAME);

    let (id, version) = parse_package(package);
    print!("{} Adding {}", "📦".bold(), id.cyan());
    if let Some(v) = &version {
        print!("@{}", v.yellow());
    }
    if is_dev {
        print!(" as dev dependency");
    }
    println!();

    let content = fs::read_to_string(manifest_path)?;
    let mut manifest: PluginManifest = json5::from_str(&content)?;

    let deps: &mut HashMap<String, String> = if is_dev {
        manifest.dev_dependencies.get_or_insert_with(HashMap::new)
    } else {
        &mut manifest.dependencies
    };

    #[allow(clippy::collapsible_if)]
    if let Some(existing_version) = deps.get(&id) {
        if let Some(ref req) = version {
            if existing_version == req {
                println!("Dependency already up to date.");
                return Ok(());
            }
        }
    }

    let final_version = match version {
        Some(v) => v,
        None => {
            let latest = fetch_latest_version(&id, http_client)?;
            println!("Latest version: {}", latest);
            latest
        },
    };

    deps.insert(id.clone(), final_version);

    fs::write(manifest_path, json5::to_string_pretty(&manifest)?)?;

    println!(
        "{} Added {} to {}",
        "✅".green(),
        id.cyan(),
        if is_dev {
            "dev-dependencies".green()
        } else {
            "dependencies".green()
        }
    );

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
