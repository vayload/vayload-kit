use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::dependency::DependencyInstaller;
use crate::http_client::HttpClient;
use crate::lock::LockFile;
use crate::manifest::PluginManifest;
use crate::utils::parse_package;

pub fn add_dependency(packages: &[String], is_dev: bool, http_client: &HttpClient) -> Result<()> {
    if packages.is_empty() {
        return Err(anyhow::anyhow!("No packages specified, skiping"));
    }

    let mut manifest = PluginManifest::load().map_err(|e| anyhow::anyhow!(e))?;

    let deps: &mut HashMap<String, String> = if is_dev {
        manifest.dev_dependencies.get_or_insert_with(HashMap::new)
    } else {
        &mut manifest.dependencies
    };

    let mut packages_with_versions: Vec<(String, String)> = Vec::new();
    let lock_path = Path::new(crate::lock::LOCK_FILENAME);
    let lock_file = LockFile::load(lock_path)?;

    let parsed_packages: Vec<(String, Option<String>)> = packages.iter().map(|p| parse_package(p)).collect();

    let mut names_need_info: Vec<String> = Vec::new();
    for (name, version) in &parsed_packages {
        let need_info = if let Some(v) = version {
            !lock_file.has_package(name, v)
        } else {
            lock_file.get_any_version(name).is_none()
        };
        if need_info {
            names_need_info.push(name.clone());
        }
    }

    let client = http_client.clone();
    let package_info_map: Arc<Mutex<HashMap<String, Option<String>>>> = Arc::new(Mutex::new(HashMap::new()));

    if !names_need_info.is_empty() {
        let handles: Vec<thread::JoinHandle<()>> = names_need_info
            .into_iter()
            .map(|name| {
                let client = client.clone();
                let package_info_map = Arc::clone(&package_info_map);

                thread::spawn(move || {
                    let info = fetch_package_info(&name, &client);
                    let mut map = package_info_map.lock().unwrap();
                    map.insert(name, info);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    let package_info_map = package_info_map.lock().unwrap();

    for (name, version) in parsed_packages {
        print!("{} Adding {}", "📦".bold(), name.cyan());
        if let Some(ref v) = version {
            print!("@{}", v.yellow());
        }
        if is_dev {
            print!(" as dev dependency");
        }
        println!();

        let final_version = match version {
            Some(v) => {
                if lock_file.has_package(&name, &v) {
                    println!("  {} Using locked version: {}", "→".bright_black(), v.green());
                    v
                } else {
                    v
                }
            },
            None => {
                if let Some(locked_version) = lock_file.get_any_version(&name) {
                    println!(
                        "  {} Using locked version: {}",
                        "→".bright_black(),
                        locked_version.green()
                    );
                    locked_version
                } else {
                    match package_info_map.get(&name) {
                        Some(Some(info)) => {
                            println!("  {} Latest stable version: {}", "→".bright_black(), info.green());
                            info.clone()
                        },
                        Some(None) => {
                            println!("  {} Could not fetch latest version, using *", "⚠".yellow());
                            "*".to_string()
                        },
                        None => "*".to_string(),
                    }
                }
            },
        };

        deps.insert(name.clone(), final_version.clone());
        packages_with_versions.push((name, final_version));
    }

    manifest.persist().map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "{} {} package(s) added to {}",
        "✓".green(),
        packages_with_versions.len().to_string().cyan(),
        if is_dev { "dev-dependencies" } else { "dependencies" }
    );

    println!();
    install_packages(&packages_with_versions, is_dev, http_client)?;

    Ok(())
}

fn install_packages(packages: &[(String, String)], is_dev: bool, http_client: &HttpClient) -> Result<()> {
    let mut installer = DependencyInstaller::new(http_client.clone(), None)?;

    for (name, version) in packages {
        if installer.has_package_in_lock(name, version) {
            println!("{} {}@{} already installe", "✓".green(), name.cyan(), version.yellow());
            continue;
        }

        installer.install_package(name, version, is_dev)?;
    }

    Ok(())
}

fn fetch_package_info(id: &str, http_client: &HttpClient) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct InfoResponse {
        latest_stable_version: String,
    }

    http_client.get::<InfoResponse>(&format!("/{}/info", id)).ok().map(|r| r.latest_stable_version)
}
