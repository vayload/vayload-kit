use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Input;
use std::fs;

use crate::{
    manifest::{FileSystemPermission, Limits, MANIFEST_FILENAME, NetworkPermission, Permissions, PluginManifest},
    utils,
};

pub fn init_project(yes: bool, directory: &Option<String>) -> Result<()> {
    let dir_path = utils::get_directory(directory).map_err(|e| anyhow::anyhow!(e))?;

    // If current directory already has a manifest file, skip initialization
    let manifest_path = dir_path.join(MANIFEST_FILENAME);
    if manifest_path.exists() {
        return Err(anyhow::anyhow!("Plugin manifest already exists, skipping"));
    }

    println!("{}", "🚀 Initializing Vayload plugin...".cyan().bold());

    let plugin_name = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("my-project").to_string();

    let name: String = if yes {
        plugin_name.clone()
    } else {
        Input::new()
            .with_prompt("Plugin name")
            .default(plugin_name)
            .interact_text()
            .context("Failed to read plugin name")?
    };

    let description: String = if yes {
        "A Vayload plugin".to_string()
    } else {
        Input::new()
            .with_prompt("Description")
            .default("A Vayload plugin".to_string())
            .interact_text()
            .context("Failed to read description")?
    };

    let author: String = if yes {
        "author".to_string()
    } else {
        Input::new()
            .with_prompt("Author")
            .default("author".to_string())
            .interact_text()
            .context("Failed to read author")?
    };

    let mut project = PluginManifest::default();
    project.set_name(name.clone());
    project.description = description.clone();
    project.author = author;
    project.permissions = Some(Permissions::new(
        FileSystemPermission::default(),
        NetworkPermission::new(vec!["jsonplaceholder.typicode.com".to_string()], false),
        Limits::default(),
    ));

    project.set_path(&manifest_path);

    project.persist().map_err(|e| anyhow::anyhow!("Failed to write manifest file: {}", e))?;

    let src_dir = dir_path.join("src");
    fs::create_dir_all(dir_path.join("src")).context("Failed to create src directory")?;

    let readme_content = format!(
        "# {}\n\n{}\n\n## Getting Started\n\n1. Run `vk install` to install dependencies\n2. Build your plugin\n3. Publish with `vk publish`\n",
        name, description
    );
    fs::write(dir_path.join("README.md"), readme_content).context("Failed to write README.md")?;
    fs::write(dir_path.join(".vkignore"), "target/\n*.lock\n.vk/\n.env\n").context("Failed to write .vkignore")?;

    let entry_content = r#"
       	local kernel = require("vhost:kernel")
        local http = require("vhost:http")

        kernel.routes.get("/todos", function(req, res)
            local response, err = http.get("https://jsonplaceholder.typicode.com/todos")
            if err == nil and response then
                res:send(response.body)
            end
        end)

        kernel.routes.get("/hello", function(req, res)
            res:send("Hello, World!")
        end)

        "#
    .to_string();

    let entry_file = dir_path.join("src/main.lua").canonicalize().map_err(|e| anyhow::anyhow!(e))?;
    fs::write(&entry_file, entry_content)?;

    println!("\n{}", "✅ Project init successfully!\n".green().bold());
    println!(
        "{} Created {}",
        "📄".green(),
        manifest_path.display().to_string().cyan()
    );
    println!("{} Created {}", "📁".green(), src_dir.display().to_string().cyan());
    println!(
        "{} Created {}",
        "📝".green(),
        dir_path.join("README.md").display().to_string().cyan()
    );
    println!(
        "{} Created {}",
        "📝".green(),
        dir_path.join(".vkignore").display().to_string().cyan()
    );

    println!("\nCreated Entry file in {}", entry_file.display().to_string().cyan());

    Ok(())
}
