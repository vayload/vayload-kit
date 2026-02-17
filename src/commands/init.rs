use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Input;
use std::fs;

use crate::manifest::{FileSystemPermission, Limits, NetworkPermission, Permissions, PluginManifest};

pub fn init_project(_yes: bool) -> Result<()> {
    println!("{}", "🚀 Initializing Vayload plugin...".cyan().bold());

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let plugin_name = current_dir.file_name().and_then(|n| n.to_str()).unwrap_or("my-project").to_string();

    let name: String = Input::new()
        .with_prompt("Plugin name")
        .default(plugin_name)
        .interact_text()
        .context("Failed to read plugin name")?;

    let description: String = Input::new()
        .with_prompt("Description")
        .default(format!("A Vayload plugin"))
        .interact_text()
        .context("Failed to read description")?;

    let author: String = Input::new().with_prompt("Author").interact_text().context("Failed to read author")?;

    let mut project = PluginManifest::default();
    project.name = name.clone().to_lowercase().replace(" ", "-");
    project.display_name = name.clone();
    project.version = "0.1.0".to_string();
    project.description = description.clone();
    project.author = author;
    project.permissions = Some(Permissions::new(
        FileSystemPermission::default(),
        NetworkPermission::new(vec!["jsonplaceholder.typicode.com".to_string()], false),
        Limits::default(),
    ));

    let manifest_path = current_dir.join("plugin.json5");
    fs::write(&manifest_path, serde_json::to_string_pretty(&project)?).context("Failed to write plugin.json5")?;

    let src_dir = current_dir.join("src");
    fs::create_dir_all(&src_dir).context("Failed to create src directory")?;

    let readme_content = format!(
        "# {}\n\n{}\n\n## Getting Started\n\n1. Run `vk install` to install dependencies\n2. Build your plugin\n3. Publish with `vk publish`\n",
        name, description
    );
    fs::write(current_dir.join("README.md"), readme_content).context("Failed to write README.md")?;

    let gitignore_content = "target/\n*.lock\n.vk/\n.env\n";
    fs::write(current_dir.join(".vkignore"), gitignore_content).context("Failed to write .vkignore")?;

    let entry_content = format!(
        r#"
       	--- @type HttpClient
        local http = require("vayload:http")

        local response, err = http.get("https://jsonplaceholder.typicode.com/todos")
        if err == nil and response then
            print(response.body)
        end
        "#
    );
    fs::write(src_dir.join("src/main.lua"), entry_content).context("Failed to write entry file")?;

    println!("\n{}", "✅ Project initialized successfully!".green().bold());
    println!(
        "{} Created {}",
        "📄".green(),
        manifest_path.display().to_string().cyan()
    );
    println!("{} Created {}", "📁".green(), src_dir.display().to_string().cyan());
    println!(
        "{} Created {}",
        "📝".green(),
        current_dir.join("README.md").display().to_string().cyan()
    );
    println!(
        "{} Created {}",
        "📝".green(),
        current_dir.join(".vkignore").display().to_string().cyan()
    );
    println!(
        "Created Entry file in {}",
        current_dir.join("src/main.lua").display().to_string().cyan()
    );

    Ok(())
}
