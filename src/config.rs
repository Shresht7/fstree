//! Handles loading configuration from a file.
//!
//! This module defines the structure for the configuration file and provides
//! a function to load it from a standard location (`~/.config/fstree/config.json`).

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::formatter::OutputFormat;
use crate::helpers;

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
