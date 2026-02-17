use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

pub fn clean_cache() -> Result<()> {
    println!("{}", "🧹 Cleaning Vayload cache and artifacts...".bold().cyan());
    println!();

    let mut cleaned_items: Vec<(String, String)> = Vec::new();
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let paths_to_clean =
        vec![(".vk", "Cache directory"), ("target", "Build artifacts"), ("node_modules", "Node modules")];

    for (path_name, description) in paths_to_clean {
        let path = current_dir.join(path_name);

        if path.exists() {
            match fs::remove_dir_all(&path) {
                Ok(_) => {
                    cleaned_items.push((path_name.to_string(), description.to_string()));
                    println!("{} Removed {}", "✓".green(), path_name.cyan());
                },
                Err(e) => {
                    println!("{} Failed to remove {}: {}", "⚠".yellow(), path_name.cyan(), e);
                },
            }
        }
    }

    let lockfile = current_dir.join("vayload.lock");
    if lockfile.exists() {
        if let Err(e) = fs::remove_file(&lockfile) {
            println!("{} Failed to remove lockfile: {}", "⚠".yellow(), e);
        } else {
            cleaned_items.push(("vayload.lock".to_string(), "Lock file".to_string()));
            println!("{}", "✓ Removed vaload.lock".green());
        }
    }

    println!();

    if cleaned_items.is_empty() {
        println!("{} Nothing to clean", "📭".yellow());
    } else {
        let total_size: usize = cleaned_items.iter().len();
        println!(
            "{} Cleaned {} item(s)",
            "✅".green(),
            total_size.to_string().green().bold()
        );
    }

    Ok(())
}
