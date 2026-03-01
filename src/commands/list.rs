use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use std::{collections::HashMap, fs};

use crate::manifest::PluginManifest;

pub fn list_dependencies(depth: Option<usize>) -> Result<()> {
    let manifest = PluginManifest::load().map_err(|e| anyhow::anyhow!(e))?;

    println!("{}", "📦 Dependencies".bold().cyan());
    println!("{}", "═".repeat(40).bright_black());
    println!();

    let max_depth = depth.unwrap_or(usize::MAX);

    let has_deps = print_dependencies_section(&manifest.dependencies, "", max_depth)?;
    let has_dev_deps = print_dependencies_section(&manifest.dev_dependencies.unwrap_or_default(), "dev ", max_depth)?;

    if !has_deps && !has_dev_deps {
        println!("{} No dependencies found", "📭".yellow());
    }

    Ok(())
}

fn print_dependencies_section(deps: &HashMap<String, String>, prefix: &str, max_depth: usize) -> Result<bool> {
    let mut has_any = false;

    if !deps.is_empty() {
        let title = if prefix.is_empty() {
            "dependencies".to_string()
        } else {
            format!("{}dependencies", prefix)
        };
        println!("{}", title.bold().green());

        for (name, version) in deps {
            let version_str = version.as_str();
            println!(
                "{} {}",
                format!("{}{}", prefix, name).cyan(),
                format!("@{}", version_str).yellow()
            );

            if max_depth > 1 {
                print_transitive_deps(name, max_depth - 1, "  ");
            }

            has_any = true;
        }
        println!();
    }

    Ok(has_any)
}

fn print_transitive_deps(package: &str, depth: usize, indent: &str) {
    if depth == 0 {
        return;
    }

    let lock_path = Path::new("vayload.lock");
    if !lock_path.exists() {
        return;
    }

    #[allow(clippy::collapsible_if)]
    if let Ok(content) = fs::read_to_string(lock_path) {
        if let Ok(lock) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(packages) = lock.get("packages").and_then(|p| p.as_array()) {
                for pkg in packages {
                    if pkg.get("id").and_then(|i| i.as_str()) == Some(package) {
                        if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_object()) {
                            for (name, version) in deps {
                                println!(
                                    "{}{}{} @ {}",
                                    indent,
                                    "├─ ".bright_black(),
                                    name.cyan(),
                                    version.as_str().unwrap_or("*").yellow()
                                );
                                if depth > 1 {
                                    print_transitive_deps(name, depth - 1, &format!("{}  ", indent));
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }
}
