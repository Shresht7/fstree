//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

mod cli;
mod config;
mod filter;
mod formatter;
mod helpers;
mod stats;
mod tree;

use std::path::PathBuf;

use formatter::OutputFormat;

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

/// The main entrypoint of the application
fn main() {
    // Load configuration from file and command-line arguments
    let file_config = config::load();
    let cli_args = cli::parse();

    // Merge configurations
    let app_config = merge_configs(file_config, cli_args);

    if let Err(e) = run(&app_config) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Merges settings from the config file and CLI arguments.
///
/// CLI arguments take precedence over the config file, which takes precedence over defaults.
fn merge_configs(file: config::FileConfig, cli: cli::Args) -> AppConfig {
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

/// Implementation of the main run logic of the command-line
fn run(args: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the path actually exists
    if !std::fs::metadata(&args.root).is_ok() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path does not exist: {}", &args.root.display().to_string()),
        )));
    }

    // Build the tree
    let mut builder = tree::TreeBuilder::new(args)?;
    let tree = builder.build(&args.root)?;

    // Format and print the tree
    let formatter = formatter::get_formatter(&args.format);
    let output = formatter.format(&tree, args, builder.get_stats())?;
    println!("{}", output);

    Ok(())
}