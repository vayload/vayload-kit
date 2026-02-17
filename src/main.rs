use anyhow::Result;
use clap::{
    FromArgMatches, Parser, Subcommand,
    builder::{
        Styles,
        styling::{AnsiColor, Effects, RgbColor},
    },
};
use colored::Colorize;
use std::sync::Arc;

mod commands;
mod config;
mod http_client;
mod manifest;
mod types;
mod utils;

#[cfg(feature = "full")]
mod auth;
#[cfg(feature = "full")]
mod credentials_manager;

#[cfg(feature = "full")]
use crate::credentials_manager::{CredentialManager, RawCredentials};

use crate::{config::AppConfig, http_client::HttpClient, manifest::PluginAccess};

#[derive(Parser)]
#[command(
    name = "vk",
    version,
    about = "Vayload Kit (vk) - Development kit for creating and managing Vayload plugins"
)]
struct AppCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Update dependencies")]
    Update {
        #[arg(help = "Optional package name to update. If omitted, updates all dependencies.")]
        package: Option<String>,
    },

    #[command(about = "Publish a plugin to the registry")]
    Publish {
        #[arg(
            short,
            long,
            help = "Directory of the plugin to publish (defaults to current directory)"
        )]
        directory: Option<String>,

        #[arg(short, long, value_parser = ["public", "private"], help = "Set package visibility")]
        access: Option<PluginAccess>,

        #[arg(long = "dry-run", help = "Simulate publishing without uploading")]
        dry_run: bool,
    },

    #[command(about = "Install a plugin")]
    Install {
        #[arg(help = "Name of the plugin to install")]
        package: String,

        #[arg(long, default_value = "./plugins", help = "Target directory for installation")]
        dir: String,
    },

    #[command(about = "Scan dependencies for known vulnerabilities")]
    Audit,

    #[command(about = "List installed dependencies")]
    List {
        #[arg(long, help = "Limit dependency tree depth")]
        depth: Option<usize>,
    },

    #[cfg(feature = "full")]
    #[command(about = "Initialize a new Vayload project")]
    Init {
        #[arg(short = 'y', long, help = "Skip prompts and use defaults")]
        yes: bool,
    },

    #[cfg(feature = "full")]
    #[command(about = "Add a dependency to the project")]
    Add {
        #[arg(help = "Package name (optionally with version, e.g. serde@1.0.0)")]
        package: String,

        #[arg(long, help = "Add as a development dependency")]
        dev: bool,
    },

    #[cfg(feature = "full")]
    #[command(about = "Remove a dependency from the project")]
    Remove {
        #[arg(help = "Package name to remove")]
        package: String,
    },

    #[cfg(feature = "full")]
    #[command(about = "Clean cache and build artifacts")]
    Clean,

    #[cfg(feature = "full")]
    #[command(about = "Authenticate with the Vayload registry")]
    Login {
        #[arg(short, long, help = "Username for authentication")]
        username: Option<String>,

        #[arg(short, long, help = "Password for authentication")]
        password: Option<String>,

        #[arg(
            short,
            long,
            value_parser = ["google", "github"],
            conflicts_with_all = ["username", "password"],
            help = "Authenticate using OAuth provider"
        )]
        oauth: Option<String>,
    },

    #[cfg(feature = "full")]
    #[command(about = "Show currently authenticated user")]
    Whoami,

    #[cfg(feature = "full")]
    #[command(about = "Logout and remove local credentials")]
    Logout,
}

fn main() {
    println!();
    if let Err(err) = run() {
        eprintln!("{} {}\n", "error:".red().bold(), err);
        std::process::exit(1);
    }

    println!();
}

fn run() -> Result<()> {
    use clap::CommandFactory;

    let orange = RgbColor(234, 88, 12);

    let styles = Styles::styled()
        .header(AnsiColor::BrightBlack.on_default())
        .usage(AnsiColor::BrightBlack.on_default())
        .literal(orange.on_default())
        .placeholder(AnsiColor::BrightBlack.on_default()) // más sobrio
        .error(AnsiColor::BrightRed.on_default() | Effects::BOLD)
        .valid(AnsiColor::BrightGreen.on_default())
        .invalid(AnsiColor::BrightRed.on_default());

    let matches = AppCli::command().styles(styles).get_matches();

    let cli = AppCli::from_arg_matches(&matches)?;
    let config = AppConfig::load()?;

    let http_client = setup_client(&config)?;

    match cli.command {
        Commands::Update { package } => commands::update::update_dependencies(package.as_deref(), &http_client)?,
        Commands::Install { package, dir } => commands::install::install_plugin(&package, &dir, &http_client)?,
        Commands::Publish { directory, access, dry_run } => {
            commands::publish::publish_plugin(&directory, access, dry_run, &http_client)?
        },
        Commands::List { depth } => commands::list::list_dependencies(depth)?,
        Commands::Audit => commands::audit::audit_dependencies(&http_client)?,

        #[cfg(feature = "full")]
        cmd @ (Commands::Add { .. }
        | Commands::Init { .. }
        | Commands::Remove { .. }
        | Commands::Clean
        | Commands::Login { .. }
        | Commands::Whoami
        | Commands::Logout) => handle_full_commands(cmd, &http_client)?,
    }
    Ok(())
}

fn setup_client(config: &AppConfig) -> Result<HttpClient> {
    #[cfg(feature = "full")]
    {
        let km = Arc::new(CredentialManager::new()?);
        let registry_url = config.server.registry_url.clone();
        setup_interactive_http_client(registry_url, km)
    }

    #[cfg(not(feature = "full"))]
    {
        use anyhow::Context;

        let token =
            std::env::var("VK_API_TOKEN").context("VK_API_TOKEN environment variable is required for CI/CD mode")?;

        HttpClient::new_with_token(config.server.registry_url.clone(), token)
    }
}

#[cfg(feature = "full")]
fn handle_full_commands(command: Commands, client: &HttpClient) -> Result<()> {
    let km = Arc::new(CredentialManager::new()?);
    let auth_handler = auth::AuthCommands::new(km.clone(), client.clone());

    match command {
        Commands::Init { yes } => commands::init::init_project(yes)?,
        Commands::Add { package, dev } => commands::add::add_dependency(&package, dev, client)?,
        Commands::Remove { package } => commands::remove::remove_dependency(&package)?,
        Commands::Clean => commands::clean::clean_cache()?,
        Commands::Login { username, password, oauth } => {
            if let Some(o) = oauth {
                auth_handler.login_with_oauth(&o)?;
            } else {
                auth_handler.login_with_password(username, password)?;
            }
        },
        Commands::Whoami => auth_handler.whoami()?,
        Commands::Logout => auth_handler.logout()?,
        _ => unreachable!(),
    }
    Ok(())
}

#[cfg(feature = "full")]
fn setup_interactive_http_client(api_url: String, km: Arc<CredentialManager>) -> Result<HttpClient> {
    let mut http_client = HttpClient::new(api_url)?;
    let fresh_client = http_client.clone();

    http_client.set_auth_fn(move || {
        use crate::auth::OAuthDataResponse;

        if km.is_refresh_token_expired() {
            return None;
        }
        if !km.is_access_token_expired() {
            return km.get_access_token().ok();
        }

        let refresh_token = km.get_refresh_token().ok()?;
        let response = fresh_client
            .post::<OAuthDataResponse, _>(
                "/auth/refresh-token",
                &serde_json::json!({ "refresh_token": refresh_token }),
            )
            .ok()?;

        km.store_tokens(RawCredentials::new(
            response.access_token.clone(),
            response.refresh_token.clone(),
            response.expires_in as u64,
        ))
        .ok()?;

        Some(response.access_token)
    });

    Ok(http_client)
}
