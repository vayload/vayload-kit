use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::blocking::multipart::{Form, Part};
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::http_client::HttpClient;
use crate::manifest::{PluginAccess, PluginManifest};
use crate::utils::{create_zip, format_bytes};

pub fn publish_plugin(
    directory: &Option<String>,
    access: Option<PluginAccess>,
    dry_run: bool,
    http_client: &HttpClient,
) -> Result<()> {
    let dir_path = if let Some(dir) = directory {
        Path::new(dir).to_path_buf()
    } else {
        std::env::current_dir()?
    };

    let dir_path = dir_path.canonicalize().context("Failed to canonicalize directory path")?;

    let manifest_path = dir_path.join("plugin.json5");
    let manifest = read_manifest(&manifest_path)?;

    println!(
        "{} Publishing {}@{}",
        "📦".bold(),
        manifest.name.cyan(),
        manifest.version.yellow()
    );

    let (zip_data, _checksum) = create_zip(&dir_path).context("Failed to create ZIP archive")?;

    println!("{} Package created ({})", "✓".green(), format_bytes(zip_data.len()));

    if dry_run {
        println!("{} Dry run mode enabled, skipping upload, only intent", "⚠".yellow());
    } else {
        upload_plugin(&manifest.name, &zip_data, access.unwrap_or_default(), http_client)?;
        println!("{} Published successfully!", "✅".green());
    }

    Ok(())
}

fn read_manifest(path: &Path) -> Result<PluginManifest> {
    let content = fs::read_to_string(path).context("Plugin need plugin.json5 for publishing")?;

    let manifest: PluginManifest = json5::from_str(&content).context("Failed to parse plugin.json5")?;

    if manifest.version.is_empty() {
        anyhow::bail!("Manifest missing required field: version");
    }
    if manifest.name.is_empty() {
        anyhow::bail!("Manifest missing required field: name");
    }

    Ok(manifest)
}

#[derive(Debug, Deserialize)]
pub struct PluginResponse {
    pub name: String,
    pub slug: String,
}

fn upload_plugin(id: &str, zip_data: &[u8], access: PluginAccess, http_client: &HttpClient) -> Result<()> {
    let form = Form::new()
        .part(
            "file",
            Part::bytes(zip_data.to_vec()).file_name(format!("{}.zip", id)).mime_str("application/zip")?,
        )
        .part("access", Part::bytes(access.as_str().to_string().into_bytes()));

    let response = http_client.post_multipart::<PluginResponse>("/plugins/publish", form);

    match response {
        Ok(data) => {
            println!(
                "Plugin '{}' published successfuly with id: {}",
                data.name.bold().blue(),
                data.slug.cyan()
            );
            Ok(())
        },
        Err(e) => Err(e.into()),
    }
}
