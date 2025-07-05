//! This module manages the application's configuration.
//!
//! It handles loading, merging, and providing access to configuration settings
//! from various sources, including a configuration file and command-line arguments.

use serde::Deserialize;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::cli;
use crate::formatter::OutputFormat;
use crate::helpers::{
    self,
    ansi::{Ansi, AnsiString},
};

/// Represents the final, merged configuration from all sources
pub struct Config {
    /// The root directory to start the tree from
    pub root: PathBuf,
    /// Whether to show the full_path for each entry
    pub full_path: bool,
    /// The prefix string to use for each level of the tree
    pub prefix: String,
    /// The prefix string to use for the last entry of each branch
    pub last_prefix: String,
    /// The prefix string to use for child entries
    pub child_prefix: String,
    /// Whether to show all files and directories, including hidden files
    pub show_all: bool,
    /// A pattern to include files that match the glob syntax
    pub include: Option<String>,
    /// A pattern to exclude files that match the glob syntax
    pub exclude: Option<String>,
    /// Custom ignore files
    pub ignore: Vec<String>,
    /// Whether to show only directories
    pub directory: bool,
    /// Whether to show a summary of directory and file counts
    pub summary: bool,
    /// Whether to show the filesize next to the name
    pub size: bool,
    /// The format to use for the filesize (e.g., Bytes, KiloBytes, etc.)
    pub size_format: helpers::bytes::Format,
    /// The maximum depth to traverse the directory tree
    pub max_depth: Option<usize>,
    /// The output format for the tree (e.g., text, json, etc.)
    pub format: OutputFormat,
    /// Whether to disable ANSI colors in the output
    pub no_color: bool,
}

impl Default for Config {
    /// Provides a default configuration, which is used as a base.
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            full_path: false,
            prefix: "├── ".to_string(),
            last_prefix: "└── ".to_string(),
            child_prefix: "│   ".to_string(),
            show_all: false,
            include: None,
            exclude: None,
            ignore: Vec::new(),
            directory: false,
            summary: false,
            size: false,
            size_format: helpers::bytes::Format::Bytes,
            max_depth: None,
            format: OutputFormat::Text,
            no_color: std::env::var("NO_COLOR").is_ok(),
        }
    }
}

/// A builder for constructing a `Config` instance.
///
/// This builder allows for layered configuration, where settings from different
/// sources can be merged. Command-line arguments take precedence over file-based
/// settings.
#[derive(Default, Debug)]
pub struct ConfigBuilder {
    pub root: Option<PathBuf>,
    pub full_path: bool,
    pub prefix: Option<String>,
    pub last_prefix: Option<String>,
    pub child_prefix: Option<String>,
    pub show_all: bool,
    pub include: Option<String>,
    pub exclude: Option<String>,
    pub ignore: Option<Vec<String>>,
    pub directory: bool,
    pub summary: bool,
    pub size: bool,
    pub size_format: Option<helpers::bytes::Format>,
    pub max_depth: Option<usize>,
    pub format: Option<OutputFormat>,
    pub no_color: bool,
}

impl ConfigBuilder {
    /// Merges another ConfigBuilder into self, prioritizing values from `self`.
    /// This effectively means `self` (e.g., CLI) overrides `other` (e.g., file).
    pub fn merge(mut self, other: ConfigBuilder) -> Self {
        self.root = self.root.or(other.root);
        self.full_path = self.full_path || other.full_path;
        self.prefix = self.prefix.or(other.prefix);
        self.last_prefix = self.last_prefix.or(other.last_prefix);
        self.child_prefix = self.child_prefix.or(other.child_prefix);
        self.show_all = self.show_all || other.show_all;
        self.include = self.include.or(other.include);
        self.exclude = self.exclude.or(other.exclude);
        self.ignore = self.ignore.or(other.ignore);
        self.directory = self.directory || other.directory;
        self.summary = self.summary || other.summary;
        self.size = self.size || other.size;
        self.size_format = self.size_format.or(other.size_format);
        self.max_depth = self.max_depth.or(other.max_depth);
        self.format = self.format.or(other.format);
        self.no_color = self.no_color || other.no_color;
        self
    }

    /// Builds the final Config struct from the ConfigBuilder, applying default values.
    pub fn build(self) -> Config {
        let defaults = Config::default();
        Config {
            root: self.root.unwrap_or(defaults.root),
            full_path: self.full_path,
            prefix: self.prefix.unwrap_or(defaults.prefix),
            last_prefix: self.last_prefix.unwrap_or(defaults.last_prefix),
            child_prefix: self.child_prefix.unwrap_or(defaults.child_prefix),
            show_all: self.show_all,
            include: self.include,
            exclude: self.exclude,
            ignore: self.ignore.unwrap_or(defaults.ignore),
            directory: self.directory,
            summary: self.summary,
            size: self.size,
            size_format: self.size_format.unwrap_or(defaults.size_format),
            max_depth: self.max_depth,
            format: self.format.unwrap_or(defaults.format),
            no_color: self.no_color || !std::io::stdout().is_terminal(),
        }
    }
}

/// Converts CLI arguments into a ConfigBuilder
impl From<cli::Args> for ConfigBuilder {
    fn from(args: cli::Args) -> Self {
        Self {
            root: args.root,
            full_path: args.full_path,
            prefix: args.prefix,
            last_prefix: args.last_prefix,
            child_prefix: args.child_prefix,
            show_all: args.show_all,
            include: args.include,
            exclude: args.exclude,
            ignore: args.ignore,
            directory: args.directory,
            summary: args.summary,
            size: args.size,
            size_format: args.size_format,
            max_depth: args.max_depth,
            format: args.format,
            no_color: args.no_color,
        }
    }
}

/// Represents the structure of the configuration file
///
/// Fields are optional, allowing users to only specify the settings
/// they want to override
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

/// Converts a `FileConfig` into a `ConfigBuilder`.
impl From<FileConfig> for ConfigBuilder {
    fn from(file_config: FileConfig) -> Self {
        Self {
            root: None, // Root is not supported in file configuration.
            full_path: file_config.full_path.unwrap_or_default(),
            prefix: file_config.prefix,
            last_prefix: file_config.last_prefix,
            child_prefix: file_config.child_prefix,
            show_all: file_config.show_all.unwrap_or_default(),
            include: file_config.include,
            exclude: file_config.exclude,
            ignore: file_config.ignore,
            directory: file_config.directory.unwrap_or_default(),
            summary: file_config.summary.unwrap_or_default(),
            size: file_config.size.unwrap_or_default(),
            size_format: file_config.size_format,
            max_depth: file_config.max_depth,
            format: file_config.format,
            no_color: file_config.no_color.unwrap_or_default(),
        }
    }
}

/// Returns the path to the configuration file.
///
/// The path is standardized to `~/.config/fstree/config.json`.
fn get_config_path() -> Option<PathBuf> {
    let home_dir = if cfg!(windows) {
        std::env::var("USERPROFILE").ok()
    } else {
        std::env::var("HOME").ok()
    }?; // Use `?` to exit early if home dir is not found.

    Some(
        Path::new(&home_dir)
            .join(".config")
            .join("fstree")
            .join("config.json"),
    )
}

/// Loads the configuration from the file system
///
/// Reads and parses the JSON configuration file. If the file doesn't exist,
/// is inaccessible, or contains invalid JSON, it returns a default, empty configuration
pub fn load_file() -> FileConfig {
    if let Some(path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(&path) {
            // Ignore empty or whitespace-only config files
            if content.trim().is_empty() {
                return FileConfig::default();
            }
            // Attempt to parse the config, printing an error if it fails
            match serde_json::from_str(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!(
                        "{} Failed to parse config file at {}: {}",
                        " Warning ".ansi(&[Ansi::BgYellow]),
                        path.display(),
                        e
                    );
                }
            }
        }
    }
    FileConfig::default()
}
