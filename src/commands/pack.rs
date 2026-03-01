use anyhow::{Context, Result};
use colored::Colorize;

use crate::manifest::{MANIFEST_FILENAME, PluginManifest};
use crate::packager::PluginPackager;
use crate::utils;

pub fn pack_plugin(directory: &Option<String>) -> Result<()> {
    let dir_path = utils::get_directory(directory).map_err(|e| anyhow::anyhow!(e))?;

    let dir_path = dir_path.canonicalize().context("Failed to canonicalize directory path")?;

    let manifest_path = dir_path.join(MANIFEST_FILENAME);
    if !manifest_path.exists() {
        anyhow::bail!(
            "Is not possible to pack a plugin without a manifest file: {}, {}",
            MANIFEST_FILENAME,
            "Please create a manifest file before packing or verify the directory path."
        );
    }

    println!("Manifest path: {}", manifest_path.display());

    let manifest = PluginManifest::load_from(&manifest_path);
    let manifest = manifest.map_err(|_| anyhow::anyhow!("Plugin need manifest file for packing"))?;
    manifest.validate().map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "{} Packing {}@{}",
        "📦".bold(),
        manifest.name.cyan(),
        manifest.version.yellow()
    );

    let packager = PluginPackager::new();
    let filename = format!("{}@{}.tar.gz", manifest.name, manifest.version);
    let tar_path = dir_path.join(filename);

    let output_path = packager.pack(&dir_path, &tar_path, false)?;

    println!("Package created in {}", output_path.display().to_string().cyan());

    Ok(())
}
