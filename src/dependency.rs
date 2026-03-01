use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use colored::Colorize;
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use tar::Archive;
use tempfile::NamedTempFile;

use crate::http_client::HttpClient;
use crate::lock::LockFile;
use crate::manifest::{MANIFEST_FILENAME, PluginManifest};

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
}

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub is_dev: bool,
    pub deps: Vec<String>,
    pub installed: bool,
    pub failed: bool,
}

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub name: String,
    pub version: String,
    pub integrity: String,
    pub success: bool,
    #[allow(unused)]
    pub error: Option<String>,
}

pub struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
    installed: HashSet<String>,
    failed: HashSet<String>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            installed: HashSet::new(),
            failed: HashSet::new(),
        }
    }

    pub fn add_dependency(&mut self, dep: Dependency) {
        let key = format!("{}@{}", dep.name, dep.version);
        self.nodes.entry(key.clone()).or_insert(DependencyNode {
            name: dep.name,
            version: dep.version,
            is_dev: dep.is_dev,
            deps: Vec::new(),
            installed: false,
            failed: false,
        });
    }

    pub fn mark_installed(&mut self, name: &str, version: &str) {
        let key = format!("{}@{}", name, version);
        if let Some(node) = self.nodes.get_mut(&key) {
            node.installed = true;
            node.failed = false;
        }
        self.installed.insert(key);
        self.failed.remove(&format!("{}@{}", name, version));
    }

    pub fn mark_failed(&mut self, name: &str, version: &str) {
        let key = format!("{}@{}", name, version);
        if let Some(node) = self.nodes.get_mut(&key) {
            node.failed = true;
        }
        self.failed.insert(key);
    }

    pub fn add_deps_for(&mut self, name: &str, version: &str, deps: Vec<(String, String)>) {
        let key = format!("{}@{}", name, version);

        let parent_is_dev = self.nodes.get(&key).map(|n| n.is_dev).unwrap_or(false);

        let mut new_nodes: Vec<(String, DependencyNode)> = Vec::new();

        if let Some(node) = self.nodes.get_mut(&key) {
            for (dep_name, dep_version) in deps {
                let dep_key = format!("{}@{}", dep_name, dep_version);
                node.deps.push(dep_key.clone());

                new_nodes.push((
                    dep_key,
                    DependencyNode {
                        name: dep_name,
                        version: dep_version,
                        is_dev: parent_is_dev,
                        deps: Vec::new(),
                        installed: false,
                        failed: false,
                    },
                ));
            }
        }

        for (key, node) in new_nodes {
            self.nodes.entry(key).or_insert(node);
        }
    }

    pub fn kahn_next_level(&self) -> Vec<(String, String, bool)> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        for (key, node) in &self.nodes {
            if node.installed || node.failed {
                continue;
            }
            in_degree.entry(key.clone()).or_insert(0);
            for dep_key in &node.deps {
                if !self.installed.contains(dep_key) {
                    *in_degree.entry(key.clone()).or_insert(0) += 1;
                }
            }
        }

        in_degree
            .into_iter()
            .filter(|(_, degree)| *degree == 0)
            .filter_map(|(key, _)| {
                self.nodes.get(&key).map(|node| (node.name.clone(), node.version.clone(), node.is_dev))
            })
            .collect()
    }

    pub fn has_pending(&self) -> bool {
        self.nodes.values().any(|n| !n.installed && !n.failed)
    }

    pub fn get_failed(&self) -> Vec<(String, String)> {
        self.nodes.values().filter(|n| n.failed).map(|n| (n.name.clone(), n.version.clone())).collect()
    }
}

pub struct DependencyInstaller {
    http_client: HttpClient,
    deps_dir: PathBuf,
    lock_file: LockFile,
    lock_path: PathBuf,
}

impl DependencyInstaller {
    pub fn new(http_client: HttpClient, deps_dir: Option<&Path>) -> Result<Self> {
        let base_dir = match deps_dir {
            Some(dir) => dir.to_path_buf(),
            None => std::env::current_dir()?,
        };

        let (lock_path, resolved_deps_dir) = Self::find_lock_file(&base_dir, 12)?
            .ok_or_else(|| anyhow::anyhow!("Youre not in vayload plugin project"))?;

        let lock_file = LockFile::load(&lock_path)?;

        Ok(Self {
            http_client,
            deps_dir: resolved_deps_dir,
            lock_file,
            lock_path,
        })
    }

    fn find_lock_file(start: &Path, max_depth: usize) -> Result<Option<(PathBuf, PathBuf)>> {
        let mut current = start;
        let mut depth = 0;

        while depth < max_depth {
            let candidate = current.join(crate::lock::LOCK_FILENAME);

            if candidate.exists() {
                return Ok(Some((candidate, current.to_path_buf())));
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }

            depth += 1;
        }

        Ok(None)
    }

    pub fn has_package_in_lock(&self, name: &str, version: &str) -> bool {
        self.lock_file.has_package(name, version)
    }

    pub fn install_all(&mut self, manifest: &PluginManifest, include_dev: bool) -> Result<Vec<InstallResult>> {
        let mut graph = DependencyGraph::new();

        for (name, version) in &manifest.dependencies {
            if self.lock_file.has_package(name, version) {
                graph.mark_installed(name, version);
                continue;
            }
            graph.add_dependency(Dependency { name: name.clone(), version: version.clone(), is_dev: false });
        }

        #[allow(clippy::collapsible_if)]
        if include_dev {
            if let Some(dev_deps) = &manifest.dev_dependencies {
                for (name, version) in dev_deps {
                    if self.lock_file.has_package(name, version) {
                        graph.mark_installed(name, version);
                        continue;
                    }
                    graph.add_dependency(Dependency { name: name.clone(), version: version.clone(), is_dev: true });
                }
            }
        }

        self.install_graph(&mut graph)
    }

    pub fn install_package(&mut self, name: &str, version: &str, is_dev: bool) -> Result<Vec<InstallResult>> {
        let mut graph = DependencyGraph::new();

        if self.lock_file.has_package(name, version) {
            println!("{} {}@{} already installed", "✓".green(), name.cyan(), version.yellow());
            return Ok(Vec::new());
        }

        graph.add_dependency(Dependency { name: name.to_string(), version: version.to_string(), is_dev });

        self.install_graph(&mut graph)
    }

    fn install_graph(&mut self, graph: &mut DependencyGraph) -> Result<Vec<InstallResult>> {
        let all_results = Arc::new(Mutex::new(Vec::new()));
        let mut level_num = 0;

        while graph.has_pending() {
            let level = graph.kahn_next_level();

            if level.is_empty() {
                let failed = graph.get_failed();
                if failed.is_empty() {
                    anyhow::bail!("Circular dependency detected");
                } else {
                    break;
                }
            }

            level_num += 1;
            println!(
                "{} Level {} ({} packages in parallel)",
                "⬇".blue(),
                level_num,
                level.len()
            );

            let handles: Vec<_> = level
                .iter()
                .map(|(name, version, is_dev)| {
                    let name = name.clone();
                    let version = version.clone();
                    let is_dev = *is_dev;
                    let http_client = self.http_client.clone();
                    let deps_dir = self.deps_dir.clone();
                    let all_results = Arc::clone(&all_results);

                    thread::spawn(move || {
                        let result = install_single(&name, &version, is_dev, &deps_dir, &http_client);

                        let mut results = all_results.lock().unwrap();
                        match result {
                            Ok((integrity, resolved_version, deps)) => {
                                println!("  {} {}@{}", "✓".green(), name.cyan(), resolved_version.yellow());
                                results.push(InstallResult {
                                    name: name.clone(),
                                    version: resolved_version.clone(),
                                    integrity,
                                    success: true,
                                    error: None,
                                });
                                Some((name, resolved_version, is_dev, deps))
                            },
                            Err(e) => {
                                println!(
                                    "  {} {}@{} - {}",
                                    "✗".red(),
                                    name.cyan(),
                                    version.yellow(),
                                    e.to_string().red()
                                );
                                results.push(InstallResult {
                                    name: name.clone(),
                                    version: version.clone(),
                                    integrity: String::new(),
                                    success: false,
                                    error: Some(e.to_string()),
                                });
                                None
                            },
                        }
                    })
                })
                .collect();

            let mut installed_this_round = Vec::new();

            for handle in handles {
                if let Some((name, version, is_dev, deps)) = handle.join().unwrap_or(None) {
                    installed_this_round.push((name, version, is_dev, deps));
                }
            }

            for (name, version, _is_dev, deps) in installed_this_round {
                graph.mark_installed(&name, &version);
                graph.add_deps_for(&name, &version, deps);
            }

            for result in all_results.lock().unwrap().iter() {
                if !result.success {
                    graph.mark_failed(&result.name, &result.version);
                }
            }
        }

        let results = all_results.lock().unwrap().clone();

        let success_count = results.iter().filter(|r| r.success).count();
        let fail_count = results.len() - success_count;

        for result in &results {
            if result.success {
                self.lock_file.add_package(&result.name, &result.version, &result.integrity, false);
            }
        }

        if !results.is_empty() {
            self.lock_file.save(&self.lock_path)?;
        }

        println!();
        if fail_count > 0 {
            println!(
                "{} {} succeeded, {} failed",
                "⚠".yellow(),
                success_count.to_string().green(),
                fail_count.to_string().red()
            );
        } else if success_count > 0 {
            println!("{} {} package(s) installed", "✅".green(), success_count);
        } else {
            println!("{} All dependencies already installed", "ℹ".bright_black());
        }

        Ok(results)
    }
}

type InstallSingleResult = (String, String, Vec<(String, String)>);

fn install_single(
    name: &str,
    version: &str,
    _is_dev: bool,
    deps_dir: &Path,
    http_client: &HttpClient,
) -> Result<InstallSingleResult> {
    fs::create_dir_all(deps_dir).context("Failed to create .deps directory")?;

    let (integrity, resolved_version, temp_path) = download_and_verify(name, Some(version), http_client)?;

    let pkg_dir = deps_dir.join(format!("{}@{}", name, resolved_version));

    if pkg_dir.exists() {
        fs::remove_dir_all(&pkg_dir).context("Failed to remove old version")?;
    }

    fs::create_dir_all(&pkg_dir).context("Failed to create package directory")?;

    extract_tar_gz(&temp_path, &pkg_dir)?;

    let manifest_path = pkg_dir.join(MANIFEST_FILENAME);
    let deps = if manifest_path.exists() {
        read_deps_from_manifest(&manifest_path)?
    } else {
        Vec::new()
    };

    fs::remove_file(&temp_path).ok();

    Ok::<InstallSingleResult, anyhow::Error>((integrity, resolved_version, deps))
}

fn download_and_verify(id: &str, version: Option<&str>, http_client: &HttpClient) -> Result<(String, String, PathBuf)> {
    let mut url = format!("/plugins/{}/download", id);
    if let Some(v) = version {
        url.push_str(&format!("?version={}", v));
    }

    #[allow(unused)]
    #[derive(serde::Deserialize)]
    struct DownloadResponse {
        id: String,
        name: String,
        version: String,
        latest_version: String,
        artifact: Artifact,
    }

    #[allow(unused)]
    #[derive(serde::Deserialize)]
    struct Artifact {
        url: String,
        expires_at: u64,
        size_bytes: u64,
        integrity: String,
        algorithm: String,
    }

    let download_meta: DownloadResponse = http_client.get(&url)?;

    let resolved_version = match version {
        Some(version) => version.to_string(),
        None => download_meta.latest_version.clone(),
    };

    let response = http_client.get_raw(&download_meta.artifact.url)?;

    let expected_integrity = &download_meta.artifact.integrity;
    let algorithm = &download_meta.artifact.algorithm;

    let mut temp_file = NamedTempFile::new().context("Failed to create temp file")?;
    let mut hasher = Sha256::new();
    let mut chunk = [0u8; 64 * 1024];
    let mut reader = response;

    loop {
        let n = reader.read(&mut chunk).context("Failed to read response")?;
        if n == 0 {
            break;
        }
        temp_file.write_all(&chunk[..n]).context("Failed to write to temp file")?;
        hasher.update(&chunk[..n]);
    }

    temp_file.flush().context("Failed to flush temp file")?;

    let computed_hash = match algorithm.as_str() {
        "sha256" => format!("sha256-{}", URL_SAFE_NO_PAD.encode(hasher.finalize())),
        _ => format!("sha256-{}", URL_SAFE_NO_PAD.encode(hasher.finalize())),
    };

    if computed_hash != *expected_integrity {
        anyhow::bail!(
            "Integrity mismatch for {}@{}: expected {}, got {}",
            id,
            resolved_version,
            expected_integrity,
            computed_hash
        );
    }

    let temp_path = temp_file.into_temp_path();
    let temp_path_buf: PathBuf = temp_path.to_path_buf();
    temp_path.keep()?;

    Ok((computed_hash, resolved_version, temp_path_buf))
}

fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = fs::File::open(archive_path).context("Failed to open archive")?;
    let tar = GzDecoder::new(file);
    let mut archive = Archive::new(tar);

    for entry in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry.context("Error reading tar entry")?;
        entry.unpack_in(dest_dir).context("Error extracting file")?;
    }

    Ok(())
}

fn read_deps_from_manifest(path: &Path) -> Result<Vec<(String, String)>> {
    let content = fs::read_to_string(path).context("Failed to read plugin.json")?;
    let manifest: PluginManifest = serde_json::from_str(&content)?;

    let mut deps = Vec::new();
    for (name, version) in &manifest.dependencies {
        deps.push((name.clone(), version.clone()));
    }

    Ok(deps)
}
