pub mod app;
pub mod cli;
pub mod core;
mod dirs;
pub mod logging;
pub mod mainui;
pub mod ui;

use clap::Parser;
use color_eyre::eyre::Context;

use crate::{
    core::config::{Config, ConfigStore, VReaderRole},
    dirs::Directories,
};

pub fn run() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Step 1: Parse CLI early so we can read --role / env var
    let cli = cli::Cli::parse();

    // Step 2: Determine effective role
    // Precedence: CLI flag > env var (handled by clap) > config file > default
    let effective_role = cli.effective_role();

    // Step 3: Create role-specific directories (creates identity/ dir too)
    let dirs = Directories::new(&effective_role)
        .wrap_err("Failed to construct base directories")?;

    // Step 4: Init logging
    let _guard = logging::init(dirs.log());

    // Step 5: Load role-specific config
    let config_store = ConfigStore::new(dirs.config());
    let mut config = config_store.get_or_create(|| Config {
        datapath: dirs.default_data().into(),
        hooks: None,
        role: VReaderRole::default(),
        vccread: None,
        llm: None,
        vchat: None,
    })?;

    // CLI role flag / env var overrides config file role
    if let Some(ref cli_role) = cli.role {
        let parsed: VReaderRole = cli_role.parse().unwrap_or_default();
        if parsed != config.role {
            tracing::info!("Role overridden by CLI/env: {:?} → {:?}", config.role, parsed);
            config.role = parsed;
        }
    }

    // Step 6: Apply --no-hooks
    if cli.no_hooks {
        config.hooks = None;
    }

    // Step 7: Dispatch
    // Clone cli.command to check — avoids ownership conflict with move into run_main_cli
    let cmd = cli.command.clone();

    if let Some(cli::Commands::Server { cmd: server_cmd }) = cmd {
        match server_cmd {
            cli::ServerCommands::Start => {
                let server_cfg = vreader_server::ServerConfig::from_env();
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(vreader_server::run_server(server_cfg))
            }
            cli::ServerCommands::Status => {
                println!("VReader server status");
                println!("  Config dir: {}", dirs.config().display());
                println!("  Data dir:   {}", dirs.default_data().display());
                println!("  Identity:   {}", dirs.identity().display());
                Ok(())
            }
        }
    } else if cmd.is_none() {
        // TUI mode
        if config.role == VReaderRole::Kid {
            config.hooks = None;
        }
        mainui::run_main_ui(&config)
    } else {
        // CLI command mode — pass the original cli (command still owned)
        cli::run_main_cli(cli, &dirs, &mut config, &config_store)
    }
}

fn main() -> color_eyre::Result<()> {
    run()
}
