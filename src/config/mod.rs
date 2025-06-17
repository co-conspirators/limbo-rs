use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

mod types;
pub use types::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Override general.debug setting
    #[arg(long)]
    pub debug: Option<bool>,

    /// Override general.timeFormat (12h or 24h)
    #[arg(long)]
    pub time_format: Option<String>,

    /// Override general.unit (metric or imperial)
    #[arg(long)]
    pub unit: Option<String>,

    /// Override general.lat
    #[arg(long)]
    pub lat: Option<f64>,

    /// Override general.lon
    #[arg(long)]
    pub lon: Option<f64>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let args = Args::parse();
        let mut builder = ConfigBuilder::builder();

        // 1. Start with default values (optional)
        builder = builder.add_source(Config::defaults());

        // 2. Load from config files in ~/.config/limbo/
        let config_dir = dirs::config_dir()
            .map(|p| p.join("limbo"))
            .or_else(|| dirs::home_dir().map(|p| p.join(".config").join("limbo")))
            .ok_or_else(|| ConfigError::Message("Could not determine config directory".into()))?;

        // If specific config file is provided via CLI, use only that
        if let Some(config_path) = args.config {
            builder = builder.add_source(File::from(config_path).required(true));
        } else {
            // Otherwise, try to load config files in order of preference
            // Later files override earlier ones
            let config_files = [
                ("config", true),        // config.{json,toml,yaml} - required
                ("config.local", false), // config.local.{json,toml,yaml} - optional
            ];

            for (base_name, required) in config_files {
                // Try each format
                for ext in ["json", "toml", "yaml", "yml"] {
                    let path = config_dir.join(format!("{}.{}", base_name, ext));
                    if path.exists() {
                        builder = builder.add_source(File::from(path).required(required));
                        break; // Use the first format found for each base name
                    }
                }
            }
        }

        // 3. Load from environment variables with LIMBO_ prefix
        // This will parse nested values like LIMBO_GENERAL__DEBUG=true
        builder = builder.add_source(
            Environment::with_prefix("LIMBO")
                .separator("__") // Use double underscore for nested fields
                .try_parsing(true), // Parse strings to appropriate types
        );

        // 4. Apply command-line overrides
        if let Some(debug) = args.debug {
            builder = builder.set_override("general.debug", debug)?;
        }
        if let Some(time_format) = args.time_format {
            builder = builder.set_override("general.timeFormat", time_format)?;
        }
        if let Some(unit) = args.unit {
            builder = builder.set_override("general.unit", unit)?;
        }
        if let Some(lat) = args.lat {
            builder = builder.set_override("general.lat", lat)?;
        }
        if let Some(lon) = args.lon {
            builder = builder.set_override("general.lon", lon)?;
        }

        // Build and deserialize
        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
