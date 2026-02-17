use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::http_client::HttpClient;
use crate::types::DownloadMeta;
use crate::utils::{extract_zip, format_bytes, parse_package};

pub fn install_plugin(package: &str, plugins_dir: &str, http_client: &HttpClient) -> Result<()> {
    let (id, version) = parse_package(package);

    print!("{} Installing {}", "📦".bold(), id.cyan());
    if let Some(v) = &version {
        print!("@{}", v.yellow());
    }
    println!();

    let plugins_path = Path::new(plugins_dir);
    fs::create_dir_all(plugins_path).context("Failed to create plugins directory")?;

    let (zip_data, meta) = download_plugin(&id, version.as_deref(), http_client)?;

    println!(
        "{} Downloaded {}@{} ({})",
        "✓".green(),
        meta.id.cyan(),
        meta.version.yellow(),
        format_bytes(zip_data.len())
    );

    if let Some(checksum) = &meta.checksum {
        println!("{} Checksum verified: {}", "✓".green(), checksum.bright_black());
    }

    let plugin_path = plugins_path.join(&id);

    if plugin_path.exists() {
        fs::remove_dir_all(&plugin_path).context("Failed to remove old version")?;
    }

    fs::create_dir_all(&plugin_path).context("Failed to create plugin directory")?;

    extract_zip(&zip_data, &plugin_path).context("Failed to extract plugin")?;

    println!(
        "{} Installed to {}",
        "✅".green(),
        plugin_path.display().to_string().bright_black()
    );

    Ok(())
}

fn download_plugin(id: &str, version: Option<&str>, http_client: &HttpClient) -> Result<(Vec<u8>, DownloadMeta)> {
    let mut url = format!("/plugins/{id}/download");
    if let Some(v) = version {
        url.push_str(&format!("?version={}", v));
    }

    let response = http_client.get_raw(&url)?;
    let checksum = response.headers().get("X-Checksum").and_then(|v| v.to_str().ok()).map(String::from);

    let plugin_version = response
        .headers()
        .get("X-Plugin-Version")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .or_else(|| version.map(String::from))
        .unwrap_or_else(|| "unknown".to_string());

    let meta = DownloadMeta { id: id.to_string(), version: plugin_version, checksum };

    let total_size = response.content_length();

    let pb = if let Some(size) = total_size {
        let pb = ProgressBar::new(size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:30.cyan/blue}] {percent}% ({bytes}/{total_bytes}) {elapsed}")
                .unwrap()
                .progress_chars("█░"),
        );
        pb.set_message("Downloading");
        Some(pb)
    } else {
        println!("Downloading (unknown size)...");
        None
    };

    let start = Instant::now();
    let mut buffer = Vec::new();

    use std::io::Read;
    let mut reader = response;
    let mut chunk = vec![0u8; 32 * 1024]; // 32KB chunks

    loop {
        match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                if let Some(ref pb) = pb {
                    pb.inc(n as u64);
                }
            },
            Err(e) => return Err(e.into()),
        }
    }

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!("{} Download completed in {:.2}s", "✓".green(), elapsed);

    Ok((buffer, meta))
}
