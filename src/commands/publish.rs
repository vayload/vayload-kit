use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::multipart::{Form, Part};
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

use crate::http_client::HttpClient;
use crate::manifest::{MANIFEST_FILENAME, PluginAccess, PluginManifest};
use crate::packager::PluginPackager;
use crate::utils;

pub fn publish_plugin(
    directory: &Option<String>,
    access: Option<PluginAccess>,
    dry_run: bool,
    http_client: &HttpClient,
) -> Result<()> {
    let dir_path = utils::get_directory(directory).map_err(|e| anyhow::anyhow!(e))?;
    let dir_path = dir_path.canonicalize().context("Failed to canonicalize directory path")?;

    // Check if manifest file exists
    let manifest_path = dir_path.join(MANIFEST_FILENAME);
    if !manifest_path.exists() {
        anyhow::bail!(
            "Is not possible to publish a plugin without a manifest file: {}, {}",
            MANIFEST_FILENAME,
            "Please create a manifest file before publishing or verify the directory path."
        );
    }

    println!("Manifest path: {}", manifest_path.display());
    let manifest = PluginManifest::load_from(&manifest_path);
    let manifest = manifest.map_err(|e| anyhow::anyhow!(e))?;
    manifest.validate().map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "{} Publishing {}@{}",
        "📦".bold(),
        manifest.name.cyan(),
        manifest.version.yellow()
    );

    let packager = PluginPackager::new();
    let filename = format!("{}@{}.tar.gz", manifest.name, manifest.version);
    let package_path = std::env::temp_dir().join(filename.clone());

    packager.pack(&dir_path, &package_path, true)?;

    if dry_run {
        println!("{} Dry run mode enabled, skipping upload, only intent", "⚠".yellow());
    } else {
        upload_plugin(&filename, &package_path, access.unwrap_or_default(), http_client)?;
        println!("{} Published successfully!", "✅".green());
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct PluginResponse {
    pub name: String,
}

fn upload_plugin(_filename: &str, path: &PathBuf, access: PluginAccess, http_client: &HttpClient) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    let form = Form::new()
        .part("file", Part::file(path)?)
        .part("access", Part::bytes(access.as_str().to_string().into_bytes()));

    spinner.set_style(ProgressStyle::with_template("{spinner:.cyan} {msg}")?);
    spinner.set_message("Uploading plugin...");
    spinner.enable_steady_tick(Duration::from_millis(100));

    let response = http_client.post_multipart::<PluginResponse>("/plugins/publish", form);

    spinner.finish_and_clear();

    match response {
        Ok(data) => {
            println!(
                "{} Plugin '{}' published successfully",
                "✓".bright_purple(),
                data.name.bold().blue()
            );
            Ok(())
        },
        Err(e) => {
            println!("{} Upload failed", "✗".bright_red());
            Err(e.into())
        },
    }
}
