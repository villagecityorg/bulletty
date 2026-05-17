use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::{Error, Parser, Subcommand};
use serde_json;
use tracing::{error, info};

use crate::core::config::{Config, ConfigStore, VReaderRole};
use crate::core::library::data::opml;
use crate::core::library::feeditem::FeedItem;
use crate::core::library::feedlibrary::FeedLibrary;
use crate::dirs::Directories;

#[derive(Parser, Clone)]
#[command(name = "vreader")]
#[command(version, about = "Your TUI feed reader — family edition with roles", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Disable all hooks defined in config
    #[arg(long)]
    pub no_hooks: bool,

    /// Operational role: operator | parent | kid (env: VREADER_ROLE)
    #[arg(long, env = "VREADER_ROLE")]
    pub role: Option<String>,
}

impl Cli {
    /// Resolve role from CLI flag / env var, or return None if unset.
    /// main.rs handles fallback to config file role.
    pub fn effective_role(&self) -> VReaderRole {
        self.role
            .as_deref()
            .and_then(|s| s.parse::<VReaderRole>().ok())
            .unwrap_or(VReaderRole::Operator)
    }
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// List all feeds and categories
    List,
    /// Add new feed
    Add {
        /// The ATOM/RSS feed URL
        url: String,
        #[arg()]
        /// The category to add under, if none is passed, it will be added to General
        category: Option<String>,
    },
    /// Update all feeds
    Update,
    /// Delete a feed
    Delete {
        /// The feed identifier (can be url, title or slug)
        ident: String,
    },
    /// Show important directories
    Dirs {
        #[command(subcommand)]
        subcmd: Option<DirsCommands>,
    },
    /// Import a list of feed sources through OPML
    Import {
        /// The filepath of the OPML file
        opml_file: String,
    },
    /// Export all your sources to an OPML file
    Export {
        /// The filepath of the OPML file
        opml_file: String,
    },
    /// Start or manage the VReader API server
    Server {
        #[command(subcommand)]
        cmd: ServerCommands,
    },
    /// Manage users and invite keys (operator only)
    User {
        #[command(subcommand)]
        cmd: UserCommands,
    },
    /// Sync feeds with the VReader API server
    Sync {
        /// Push local feeds (OPML export) to the server
        #[arg(long)]
        push: bool,
        /// Pull feeds (OPML import) from the server
        #[arg(long)]
        pull: bool,
        /// API server base URL (overrides config)
        #[arg(long)]
        url: Option<String>,
        /// Admin API key (overrides config)
        #[arg(long)]
        api_key: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum ServerCommands {
    /// Start the API server (long-running process)
    Start,
    /// Show server config and status
    Status,
}

#[derive(Subcommand, Clone)]
pub enum UserCommands {
    /// Generate an invite code for a new user (registers it on the server)
    Invite {
        /// Role for the new user: admin or kid
        #[arg(long, default_value = "admin")]
        role: String,
        /// Days until the invite expires
        #[arg(long, default_value_t = 30)]
        days: u32,
    },
    /// List registered users from the server
    List,
    /// Revoke a user's access
    Revoke {
        /// The user ID to revoke
        id: String,
    },
}

#[derive(Subcommand, Clone)]
pub enum DirsCommands {
    /// Show or update the library path
    Library {
        /// New path for the library directory
        path: Option<PathBuf>,
    },
    /// Show the logs directory
    Logs,
    /// Show the local config directory
    LocalConfig,
}

pub fn run_main_cli(
    cli: Cli,
    dirs: &Directories,
    config: &mut Config,
    config_store: &ConfigStore,
) -> color_eyre::Result<()> {
    info!("Initializing CLI");

    match &cli.command {
        Some(Commands::List) => command_list(&cli, &config.datapath),
        Some(Commands::Add { url, category }) => command_add(&cli, url, category, &config.datapath),
        Some(Commands::Update) => command_update(&cli, &config.datapath),
        Some(Commands::Delete { ident }) => command_delete(&cli, ident, &config.datapath),
        Some(Commands::Dirs { subcmd }) => command_dirs(&cli, subcmd, dirs, config, config_store),
        Some(Commands::Import { opml_file }) => command_import(&cli, opml_file, &config.datapath),
        Some(Commands::Export { opml_file }) => command_export(&cli, opml_file, &config.datapath),
        // Server commands are dispatched in main.rs — but handle here too for safety
        Some(Commands::Server { .. }) => {
            info!("Server command dispatched via CLI (should be handled in main.rs)");
            Ok(())
        }
        Some(Commands::User { cmd }) => command_user(cmd, config),
        Some(Commands::Sync { push, pull, url, api_key }) => {
            command_sync(*push, *pull, url.as_deref(), api_key.as_deref(), config, &config.datapath)
        }
        None => Ok(()),
    }
}

fn command_list(_cli: &Cli, data_dir: &Path) -> color_eyre::Result<()> {
    let library = FeedLibrary::new(data_dir);

    println!("Feeds Registered\n\n");
    for category in library.feedcategories.iter() {
        println!("{}", category.title);
        for feed in category.feeds.iter().as_ref() {
            println!("\t-> {}: {}", feed.title, feed.slug);
        }
        println!();
    }

    Ok(())
}

fn command_add(
    _cli: &Cli,
    url: &str,
    category: &Option<String>,
    data_dir: &Path,
) -> color_eyre::Result<()> {
    let mut library = FeedLibrary::new(data_dir);
    match library.add_feed_from_url(url, category) {
        Ok(feed) => {
            info!("Feed added: {}", feed.title);
            println!("Feed added: {}", feed.title);
        }
        Err(err) => {
            error!("{err}");
            println!("{err}");
        }
    }

    Ok(())
}

fn command_update(_cli: &Cli, data_dir: &Path) -> color_eyre::Result<()> {
    let library = FeedLibrary::new(data_dir);

    for category in library.feedcategories.iter() {
        for feed in category.feeds.iter() {
            info!("Updating {}", feed.title);
            println!("Updating {}", feed.title);
            library
                .data
                .update_feed_entries(&category.title, feed, None)?;
        }
    }

    Ok(())
}

fn confirm_delete(title: &str) -> Result<bool, Error> {
    print!("Are you sure you want to delete '{title}'? That can't be reverted. [y/N] ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    let normalized_input = choice.trim().to_lowercase();
    Ok(matches!(normalized_input.as_str(), "y" | "yes"))
}

fn command_delete(_cli: &Cli, ident: &str, data_dir: &Path) -> color_eyre::Result<()> {
    let library = FeedLibrary::new(data_dir);

    let matches: Vec<&FeedItem> = library.get_matching_feeds(ident);
    let matches_len = matches.len();

    match matches_len {
        0 => {
            info!("No matching feeds exist");
            println!("No matching feeds exist");
        }
        1 => {
            let matched = matches[0];
            if confirm_delete(&matched.title)? {
                library.delete_feed(&matched.slug, &matched.category)?;
                info!("Feed deleted: {}", &matched.title);
                println!("Feed deleted: {}", &matched.title);
            } else {
                info!("Feed was not deleted: {}", &matched.title);
                println!("Feed was not deleted: {}", &matched.title);
            }
        }
        _ => {
            println!("There were {} feeds found with that identifier:", {
                matches_len
            });
            let iter = matches.iter().enumerate();
            for (i, feed) in iter {
                println!("\t-> {}) {}/{}", i + 1, &feed.category, &feed.title);
            }
            print!("Which one would you like to delete? ");
            io::stdout().flush()?;

            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;

            let normalized_input = choice.trim();

            match normalized_input.parse::<usize>() {
                Ok(ind) => {
                    if ind >= 1 && ind <= matches_len {
                        let title =
                            format!("{}/{}", &matches[ind - 1].category, &matches[ind - 1].title);
                        if confirm_delete(&title)? {
                            library.delete_feed(
                                &matches[ind - 1].slug,
                                &matches[ind - 1].category,
                            )?;
                            info!("Feed deleted: {title}");
                            println!("Feed deleted: {title}");
                        } else {
                            info!("Feed was not deleted: {title}");
                            println!("Feed was not deleted: {title}");
                        }
                    } else {
                        println!("Invalid selection");
                    }
                }
                Err(_) => {
                    println!("Invalid selection");
                }
            }
        }
    }

    Ok(())
}

fn command_dirs(
    _cli: &Cli,
    subcmd: &Option<DirsCommands>,
    dirs: &Directories,
    config: &mut Config,
    config_store: &ConfigStore,
) -> color_eyre::Result<()> {
    match subcmd {
        Some(DirsCommands::Library { path }) => {
            if let Some(new_path) = path {
                config.datapath = new_path.clone();
                config_store.save(config)?;
                println!("Updated library path to: {}", config.datapath.display());
            } else {
                println!("Library path: {}", config.datapath.display());
            }
        }
        Some(DirsCommands::Logs) => {
            println!("Logs directory: {}", dirs.log().display());
        }
        Some(DirsCommands::LocalConfig) => {
            println!("Config directory: {}", dirs.config().display());
        }
        None => {
            println!("Config directory: {}", dirs.config().display());
            println!("Library path: {}", config.datapath.display());
            println!("Logs directory: {}", dirs.log().display());
            println!("Identity directory: {}", dirs.identity().display());
        }
    }

    Ok(())
}

fn command_import(
    _cli: &Cli,
    opml_file: &str,
    data_dir: &Path,
) -> color_eyre::Result<()> {
    let feeds = opml::get_opml_feeds(opml_file)?;
    let mut library = FeedLibrary::new(data_dir);

    for feed in feeds {
        match library.add_feed_from_url(&feed.url, &feed.category) {
            Ok(f) => println!("Imported: {}", f.title),
            Err(e) => println!("Skipped {}: {}", feed.url, e),
        }
    }

    Ok(())
}

fn command_export(
    _cli: &Cli,
    opml_file: &str,
    data_dir: &Path,
) -> color_eyre::Result<()> {
    let library = FeedLibrary::new(data_dir);
    opml::save_opml(&library.feedcategories, opml_file)?;
    println!("Exported feeds to: {opml_file}");
    Ok(())
}

// ─── User commands ─────────────────────────────────────────────────────

fn command_user(cmd: &UserCommands, config: &Config) -> color_eyre::Result<()> {
    match cmd {
        UserCommands::Invite { role, days } => command_user_invite(role, *days, config),
        UserCommands::List => command_user_list(config),
        UserCommands::Revoke { id } => command_user_revoke(id, config),
    }
}

fn api_base(config: &Config) -> color_eyre::Result<String> {
    config
        .vccread
        .as_ref()
        .and_then(|v| v.api_url.as_deref())
        .map(|s| s.trim_end_matches('/').to_string())
        .or_else(|| {
            // Derive from master_opml_url if api_url not set
            config.vccread.as_ref().and_then(|v| {
                v.master_opml_url.as_deref().and_then(|url| {
                    if let Some(pos) = url.find("/opml") {
                        Some(url[..pos].to_string())
                    } else {
                        None
                    }
                })
            })
        })
        .ok_or_else(|| color_eyre::eyre::eyre!(
            "API URL not configured. Set [vccread] api_url in config or use --url flag with sync command."
        ))
}

fn api_key(config: &Config) -> Option<String> {
    config.vccread.as_ref().and_then(|v| v.api_key.clone())
}

fn api_post(path: &str, body: &serde_json::Value, api_url: &str, key: &str) -> color_eyre::Result<serde_json::Value> {
    let url = format!("{}{}", api_url, path);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", key))
        .json(body)
        .send()?;

    let status = resp.status();
    let json: serde_json::Value = resp.json()?;

    if !status.is_success() {
        let err_msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        return Err(color_eyre::eyre::eyre!("API error ({}): {}", status.as_u16(), err_msg));
    }
    Ok(json)
}

fn api_get(path: &str, api_url: &str, key: &str) -> color_eyre::Result<serde_json::Value> {
    let url = format!("{}{}", api_url, path);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", key))
        .send()?;

    let status = resp.status();
    let json: serde_json::Value = resp.json()?;

    if !status.is_success() {
        let err_msg = json.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        return Err(color_eyre::eyre::eyre!("API error ({}): {}", status.as_u16(), err_msg));
    }
    Ok(json)
}

fn command_user_invite(role: &str, days: u32, config: &Config) -> color_eyre::Result<()> {
    let base = api_base(config)?;
    let key = api_key(config).ok_or_else(|| {
        color_eyre::eyre::eyre!("API key not configured. Set [vccread] api_key in config.")
    })?;

    let body = serde_json::json!({
        "role": role,
        "expires_in_days": days,
    });

    let resp = api_post("/v1/invites", &body, &base, &key)?;

    let code = resp["code"].as_str().unwrap_or("?");
    let expires = resp["expires_at"].as_str().unwrap_or("?");

    println!("\n═══════════════════════════════════════════");
    println!("  Invite Code Generated");
    println!("═══════════════════════════════════════════");
    println!("  Code:    {}", code);
    println!("  Role:    {}", role);
    println!("  Expires: {}", expires);
    println!();
    println!("  Share this code with the new user.");
    println!("  They can register at the VReader client.");
    println!("═══════════════════════════════════════════\n");

    Ok(())
}

fn command_user_list(config: &Config) -> color_eyre::Result<()> {
    let base = api_base(config)?;
    let key = api_key(config).ok_or_else(|| {
        color_eyre::eyre::eyre!("API key not configured. Set [vccread] api_key in config.")
    })?;

    let resp = api_get("/v1/users", &base, &key)?;

    let users = resp.as_array().ok_or_else(|| {
        color_eyre::eyre::eyre!("Unexpected response format")
    })?;

    if users.is_empty() {
        println!("No registered users.");
        return Ok(());
    }

    println!("\nRegistered Users:");
    println!("───────────────────────────────────────────────");
    for user in users {
        let id = user["id"].as_str().unwrap_or("?");
        let name = user["name"].as_str().unwrap_or("?");
        let role = user["role"].as_str().unwrap_or("?");
        let active = user["active"].as_bool().unwrap_or(false);
        let created = user["created_at"].as_str().unwrap_or("?");
        println!("  {} ({}): {} — {} — created {}", id, role, name, if active { "active" } else { "revoked" }, created);
    }
    println!();

    Ok(())
}

fn command_user_revoke(user_id: &str, config: &Config) -> color_eyre::Result<()> {
    let base = api_base(config)?;
    let key = api_key(config).ok_or_else(|| {
        color_eyre::eyre::eyre!("API key not configured. Set [vccread] api_key in config.")
    })?;

    let body = serde_json::json!({});
    let path = format!("/v1/users/{}/revoke", user_id);
    let resp = api_post(&path, &body, &base, &key)?;

    println!("User {} revoked successfully.", resp["user_id"].as_str().unwrap_or(user_id));
    Ok(())
}

// ─── Sync commands ─────────────────────────────────────────────────────

fn command_sync(
    push: bool,
    pull: bool,
    cli_url: Option<&str>,
    cli_key: Option<&str>,
    config: &Config,
    data_dir: &Path,
) -> color_eyre::Result<()> {
    // Use CLI flag override, then config
    let base = match cli_url {
        Some(u) => u.trim_end_matches('/').to_string(),
        None => api_base(config)?,
    };
    let key = match cli_key {
        Some(k) => k.to_string(),
        None => api_key(config).ok_or_else(|| {
            color_eyre::eyre::eyre!("API key not configured. Use --api-key or set [vccread] api_key in config.")
        })?,
    };

    if push {
        // Export local library → POST to server
        let library = FeedLibrary::new(data_dir);

        // Export feeds to temp OPML file
        let opml_path = "./_sync_temp.opml";
        opml::save_opml(&library.feedcategories, opml_path)?;
        let opml_content = std::fs::read_to_string(opml_path)?;
        std::fs::remove_file(opml_path)?;

        let body = serde_json::json!({ "opml": opml_content });
        let resp = api_post("/v1/feeds/sync", &body, &base, &key)?;
        let count = resp["feeds_imported"].as_u64().unwrap_or(0);
        println!("Pushed {} feeds to server.", count);

    } else if pull {
        // GET OPML from server → import locally
        use crate::core::library::data::opml;

        let url = format!("{}/v1/opml", base);
        let client = reqwest::blocking::Client::new();
        // OPML endpoint is public — no auth needed
        let resp = client.get(&url).send()?;
        if !resp.status().is_success() {
            return Err(color_eyre::eyre::eyre!("Server returned {}", resp.status()));
        }
        let opml_content = resp.text()?;

        // Write to temp file and import
        let tmp_path = "./_sync_pull.opml";
        std::fs::write(tmp_path, &opml_content)?;
        let feeds = opml::get_opml_feeds(tmp_path)?;
        std::fs::remove_file(tmp_path)?;

        let mut library = FeedLibrary::new(data_dir);
        let mut imported = 0;
        for feed in &feeds {
            match library.add_feed_from_url(&feed.url, &feed.category) {
                Ok(f) => {
                    info!("Synced: {}", f.title);
                    imported += 1;
                }
                Err(e) => info!("Skipped (already exists or error): {}", e),
            }
        }
        println!("Pulled {} feeds from server ({} new).", feeds.len(), imported);

    } else {
        println!("Use --push or --pull to specify sync direction.");
    }

    Ok(())
}
