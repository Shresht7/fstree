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
pub struct Config {
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
pub fn merge_configs(file: FileConfig, cli: cli::Args) -> Config {
    Config {
        // CLI > File > Default
        root: cli.root.unwrap_or_else(|| PathBuf::from(".")),

        // CLI flags are booleans, so they are always present
        full_path: merge_options(cli.full_path, file.full_path, false),
        show_all: merge_options(cli.show_all, file.show_all, false),
        directory: merge_options(cli.directory, file.directory, false),
        summary: merge_options(cli.summary, file.summary, false),
        size: merge_options(cli.size, file.size, false),
        no_color: merge_options(cli.no_color, file.no_color, false),

        // CLI > File > Default
        prefix: merge_options(cli.prefix, file.prefix, "├── ".to_string()),
        last_prefix: merge_options(cli.last_prefix, file.last_prefix, "└── ".to_string()),
        child_prefix: merge_options(cli.child_prefix, file.child_prefix, "│   ".to_string()),

        // These are optional and can remain None
        include: merge_options(Some(cli.include), Some(file.include), None),
        exclude: merge_options(Some(cli.exclude), Some(file.exclude), None),
        max_depth: merge_options(Some(cli.max_depth), Some(file.max_depth), None),

        // CLI > File > Default
        ignore: merge_options(cli.ignore, file.ignore, Vec::new()),
        size_format: merge_options(
            cli.size_format,
            file.size_format,
            helpers::bytes::Format::Bytes,
        ),
        format: merge_options(cli.format, file.format, OutputFormat::Text),
    }
}

fn merge_options<T: Clone>(cli: Option<T>, file: Option<T>, default: T) -> T {
    cli.or(file).unwrap_or(default)
}

impl From<cli::Args> for Config {
    fn from(value: cli::Args) -> Self {
        return merge_configs(FileConfig::default(), value);
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
