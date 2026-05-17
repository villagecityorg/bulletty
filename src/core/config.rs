use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::core::defs::CONFIG_FILE;
use crate::core::hooks::AppHooks;

/// VReader operational role.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VReaderRole {
    Operator,
    Parent,
    Kid,
}

impl Default for VReaderRole {
    fn default() -> Self {
        Self::Operator
    }
}

impl std::str::FromStr for VReaderRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "operator" => Ok(Self::Operator),
            "parent" | "parents" => Ok(Self::Parent),
            "kid" | "kids" => Ok(Self::Kid),
            _ => Err(format!("Invalid role: {s}. Expected: operator, parent, kid")),
        }
    }
}

/// vccread.chatek.co integration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VccReadConfig {
    pub master_opml_url: Option<String>,
    /// Base URL for the VReader API, e.g. "https://vreader.chatek.co/api"
    pub api_url: Option<String>,
    #[serde(default)]
    pub auto_sync: bool,
    pub api_key: Option<String>,
}

/// LLM provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String, // "ollama" | "deepseek"
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub kid_safe_filter: bool,
}

/// vchat.email agent identity configuration (future).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VChatConfig {
    pub nats_url: Option<String>,
    pub agent_name: Option<String>,
    pub steward: Option<String>,
    pub template: Option<String>,
    #[serde(default)]
    pub auto_sync: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub datapath: PathBuf,
    #[serde(default)]
    pub hooks: Option<AppHooks>,
    #[serde(default)]
    pub role: VReaderRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vccread: Option<VccReadConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llm: Option<LlmConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vchat: Option<VChatConfig>,
}

pub struct ConfigStore {
    file_path: PathBuf,
}

impl ConfigStore {
    pub fn new(dir: &Path) -> Self {
        ConfigStore {
            file_path: dir.join(CONFIG_FILE),
        }
    }

    pub fn get_or_create(&self, default_config: impl FnOnce() -> Config) -> Result<Config> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent).wrap_err_with(|| {
                format!(
                    "Failed to create base configuration directory {}",
                    parent.to_string_lossy()
                )
            })?;
        }

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.file_path)
        {
            Ok(mut file) => {
                Self::write(&default_config(), &mut file)?;
                Ok(())
            }
            Err(err) => match err.kind() {
                ErrorKind::AlreadyExists => Ok(()),
                _ => Err(err),
            },
        }
        .wrap_err_with(|| {
            format!(
                "Failed to create default configuration file at {}",
                self.file_path.to_string_lossy()
            )
        })?;

        self.read().wrap_err_with(|| {
            format!(
                "Failed to read configuration from {}",
                self.file_path.to_string_lossy()
            )
        })
    }

    pub fn save(&self, config: &Config) -> Result<()> {
        File::create(&self.file_path)
            .and_then(|mut file| Self::write(config, &mut file))
            .wrap_err_with(|| {
                format!(
                    "Failed to save configuration to {}",
                    self.file_path.to_string_lossy()
                )
            })
    }

    fn read(&self) -> Result<Config> {
        Ok(toml::from_str(&std::fs::read_to_string(&self.file_path)?)?)
    }

    fn write(config: &Config, file: &mut File) -> std::io::Result<()> {
        file.write_all(
            &toml::to_string(config)
                .expect("configuration should be serializable")
                .into_bytes(),
        )
    }
}
