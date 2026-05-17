use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;
use etcetera::BaseStrategy;

use crate::core::config::VReaderRole;
use crate::core::defs::{CONFIG_PATH, DATA_DIR, IDENTITY_DIR, LOG_BASE_DIR, LOG_SUBDIR};

pub struct Directories {
    log: PathBuf,
    default_data: PathBuf,
    config: PathBuf,
    identity: PathBuf,
}

impl Directories {
    /// Build directories for a specific role.
    ///
    /// Role is appended as a suffix: `~/.config/vreader-operator/`
    /// or just `~/.config/vreader/` for the default (operator).
    pub fn new(role: &VReaderRole) -> Result<Self> {
        let current_strategy = etcetera::choose_base_strategy()?;
        let suffix = role_dir_suffix(role);

        let config = current_strategy
            .config_dir()
            .join(format!("{CONFIG_PATH}{suffix}"));

        let identity = config.join(IDENTITY_DIR);

        let default_data = current_strategy
            .data_dir()
            .join(format!("{DATA_DIR}{suffix}"));

        let log = current_strategy
            .state_dir()
            .unwrap_or_else(|| current_strategy.cache_dir())
            .join(format!("{LOG_BASE_DIR}{suffix}"))
            .join(LOG_SUBDIR);

        // Ensure identity dir exists (even if unused today; future vchat.email IDP will write here)
        std::fs::create_dir_all(&identity)?;

        Ok(Self {
            log,
            default_data,
            config,
            identity,
        })
    }

    pub fn log(&self) -> &Path {
        &self.log
    }

    pub fn default_data(&self) -> &Path {
        &self.default_data
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn identity(&self) -> &Path {
        &self.identity
    }
}

fn role_dir_suffix(role: &VReaderRole) -> &'static str {
    match role {
        VReaderRole::Operator => "", // bare: ~/.config/vreader/
        VReaderRole::Parent => "-parents",
        VReaderRole::Kid => "-kids",
    }
}
