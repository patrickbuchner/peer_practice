use std::fs;
use std::path::{Path, PathBuf};

use crate::input::config::current::{Config, Envelope};
use crate::run;
use clap::{Parser, Subcommand};
use config::ConfigEnvelope;
use eyre::{Context, eyre};

pub mod config;

#[derive(Debug, Parser)]
#[command(
    name = "peer-practice",
    version,
    about = "The server backend for a peer-practice service."
)]
pub struct App {
    #[command(subcommand)]
    command: Commands,
}
impl App {
    pub async fn run(self) -> eyre::Result<()> {
        match self.command {
            Commands::Generate { path, force } => {
                generate_default_file(&path, force)?;
                println!("Wrote default config to {}", path.display());
                Ok(())
            }
            Commands::Run { config } => {
                let file_cfg = read_config_file(&config)
                    .with_context(|| format!("Failed to read {}", config.display()))?;

                run(file_cfg).await
            }
            Commands::Show { config } => {
                let file_cfg = read_config_file(&config)
                    .with_context(|| format!("Failed to read {}", config.display()))?;
                println!("{}", toml::to_string_pretty(&file_cfg)?);

                Ok(())
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate a default TOML configuration file
    Generate {
        /// Where to write the file
        #[arg(value_name = "FILE")]
        path: PathBuf,

        /// Overwrite the file if it exists
        #[arg(long)]
        force: bool,
    },

    /// Run with merged config from (precedence) TOML
    Run {
        /// TOML config file path
        #[arg(long, value_name = "FILE")]
        config: PathBuf,
    },

    /// Show current configuration (differences from defaults by default)
    Show {
        /// TOML config file path
        #[arg(long, value_name = "FILE")]
        config: PathBuf,
    },
}

fn generate_default_file(path: &Path, force: bool) -> eyre::Result<()> {
    if path.exists() && !force {
        return Err(eyre!(
            "Refusing to overwrite existing file: {} (use --force)",
            path.display()
        ));
    }

    let default = Envelope::default();
    let file = toml::to_string_pretty(&default)?;

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    fs::write(path, file)?;
    Ok(())
}

fn read_config_file(path: &Path) -> eyre::Result<Config> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    ConfigEnvelope::config(&data)
        .with_context(|| format!("Failed to parse config file in {}", path.display()))
}
