use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::http_client::HttpClient;

pub fn audit_dependencies(http_client: &HttpClient) -> Result<()> {
    println!("{}", "🔍 Scanning for vulnerabilities...".bold().cyan());
    println!();

    let manifest_path = Path::new("plugin.json5");

    if !manifest_path.exists() {
        anyhow::bail!("No plugin.json5 found. Are you in a Vayload project?");
    }

    let content = fs::read_to_string(manifest_path).context("Failed to read plugin.json5")?;
    let manifest: serde_json::Value = json5::from_str(&content).context("Failed to parse plugin.json5")?;

    let mut all_deps: Vec<(String, String, bool)> = Vec::new();

    if let Some(deps) = manifest.get("dependencies").and_then(|d| d.as_object()) {
        for (name, version) in deps {
            all_deps.push((name.clone(), version.as_str().unwrap_or("*").to_string(), false));
        }
    }

    if let Some(dev_deps) = manifest.get("dev-dependencies").and_then(|d| d.as_object()) {
        for (name, version) in dev_deps {
            all_deps.push((name.clone(), version.as_str().unwrap_or("*").to_string(), true));
        }
    }

    if all_deps.is_empty() {
        println!("{} No dependencies to audit", "✅".green());
        return Ok(());
    }

    println!("{} Checking {} packages...", "📋".bold(), all_deps.len());
    println!();

    let mut vulnerabilities_found = false;
    let mut checked = 0;

    for (name, version, is_dev) in &all_deps {
        checked += 1;

        match check_vulnerability(name, http_client) {
            Ok(Some(vulns)) => {
                vulnerabilities_found = true;
                println!(
                    "{} {}@{} ( {})",
                    "⚠️".red().bold(),
                    name.cyan(),
                    version.yellow(),
                    if *is_dev { "dev" } else { "prod" }
                );

                for vuln in vulns {
                    println!(
                        "{}",
                        format!("  [{}] {}", vuln.severity.to_uppercase().red(), vuln.title).red()
                    );
                    println!("{}", format!("    ID: {}", vuln.id).bright_black());
                    if let Some(desc) = &vuln.description {
                        println!("{}", format!("    {}", desc).bright_black());
                    }
                    if let Some(patched) = &vuln.patched_versions {
                        println!("{}", format!("    Patched in: {}", patched).green());
                    }
                    println!();
                }
            },
            Ok(None) => {
                print!(".");
            },
            Err(_) => {
                print!("?");
            },
        }
    }

    println!();
    println!();

    if vulnerabilities_found {
        println!("{}", "❌ Vulnerabilities found!".red().bold());
        println!("{}", "Please update your dependencies using 'vk update'".yellow());
    } else {
        println!("{} No vulnerabilities found!", "✅".green().bold());
        println!("{} {} packages audited successfully", "✓".green(), checked);
    }

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct VulnerabilityResponse {
    vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, serde::Deserialize)]
struct Vulnerability {
    id: String,
    title: String,
    #[serde(rename = "severity")]
    severity: String,
    #[serde(rename = "description")]
    description: Option<String>,
    #[serde(rename = "patched_versions")]
    patched_versions: Option<String>,
}

fn check_vulnerability(package: &str, http_client: &HttpClient) -> Result<Option<Vec<Vulnerability>>> {
    match http_client.get::<VulnerabilityResponse>(&format!("/audit/{}", package)) {
        Ok(response) => {
            if response.vulnerabilities.is_empty() {
                Ok(None)
            } else {
                Ok(Some(response.vulnerabilities))
            }
        },
        Err(_) => Ok(None),
    }
}
