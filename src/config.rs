//! Handles loading configuration from a file.
//!
//! This module defines the structure for the configuration file and provides
//! a function to load it from a standard location (`~/.config/fstree/config.json`).

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli;
use crate::formatter::OutputFormat;
use crate::helpers;

/// Represents the final, merged configuration from all sources.
pub struct AppConfig {
    pub root: PathBuf,
    pub full_path: bool,
    pub prefix: String,
    pub last_prefix: String,
    pub child_prefix: String,
    pub show_all: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub ignore: Vec<String>,
    pub directory: bool,
    pub summary: bool,
    pub size: bool,
    pub size_format: helpers::bytes::Format,
    pub max_depth: Option<usize>,
    pub format: OutputFormat,
    pub no_color: bool,
}

/// Merges settings from the config file and CLI arguments.
///
/// CLI arguments take precedence over the config file, which takes precedence over defaults.
pub fn merge_configs(file: FileConfig, cli: cli::Args) -> AppConfig {
    AppConfig {
        // CLI > File > Default
        root: cli.root.unwrap_or_else(|| PathBuf::from(".")),

        // CLI flags are booleans, so they are always present
        full_path: cli.full_path || file.full_path.unwrap_or(false),
        show_all: cli.show_all || file.show_all.unwrap_or(false),
        directory: cli.directory || file.directory.unwrap_or(false),
        summary: cli.summary || file.summary.unwrap_or(false),
        size: cli.size || file.size.unwrap_or(false),
        no_color: cli.no_color || file.no_color.unwrap_or(false),

        // CLI > File > Default
        prefix: cli
            .prefix
            .or(file.prefix)
            .unwrap_or_else(|| "├── ".to_string()),
        last_prefix: cli
            .last_prefix
            .or(file.last_prefix)
            .unwrap_or_else(|| "└── ".to_string()),
        child_prefix: cli
            .child_prefix
            .or(file.child_prefix)
            .unwrap_or_else(|| "│   ".to_string()),

        // These are optional and can remain None
        include: cli.include.or(file.include),
        exclude: cli.exclude.or(file.exclude),
        max_depth: cli.max_depth.or(file.max_depth),

        // CLI > File > Default
        ignore: cli.ignore.or(file.ignore).unwrap_or_default(),
        size_format: cli
            .size_format
            .or(file.size_format)
            .unwrap_or(helpers::bytes::Format::Bytes),
        format: cli.format.or(file.format).unwrap_or(OutputFormat::Text),
    }
}

/// Represents the structure of the configuration file.
///
/// Fields are optional, allowing users to only specify the settings
/// they want to override.
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct FileConfig {
    pub full_path: Option<bool>,
    pub prefix: Option<String>,
    pub last_prefix: Option<String>,
    pub child_prefix: Option<String>,
    pub show_all: Option<bool>,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub ignore: Option<Vec<String>>,
    pub directory: Option<bool>,
    pub summary: Option<bool>,
    pub size: Option<bool>,
    pub size_format: Option<helpers::bytes::Format>,
    pub max_depth: Option<usize>,
    pub format: Option<OutputFormat>,
    pub no_color: Option<bool>,
}

/// Returns the path to the configuration file.
///
/// The path is standardized to `~/.config/fstree/config.json` for all platforms.
fn get_config_path() -> Option<PathBuf> {
    // Using `home::home_dir()` would be simpler but adds a dependency.
    // This manual implementation is a good compromise.
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    };

    home_dir.map(|dir| {
        Path::new(&dir)
            .join(".config")
            .join("fstree")
            .join("config.json")
    })
}

/// Loads the configuration from the file system.
///
/// Reads and parses the JSON configuration file. If the file doesn't exist,
/// is inaccessible, or contains invalid JSON, it returns a default, empty configuration.
pub fn load() -> FileConfig {
    if let Some(path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(&path) {
            // Ignore empty or whitespace-only config files
            if content.trim().is_empty() {
                return FileConfig::default();
            }
            // Attempt to parse the config, printing an error if it fails.
            match serde_json::from_str(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse config file at '{}': {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }
    FileConfig::default()
}
