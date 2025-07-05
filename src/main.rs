//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

use crate::config::ConfigBuilder;

mod cli;
mod config;
mod filter;
mod formatter;
mod helpers;
mod stats;
mod tree;

/// The main entrypoint of the application
fn main() {
    // Parse command-line arguments
    let args = cli::parse();

    // Load settings from the configuration file, if available
    let config_file = config::load_file();

    // Merge configurations, with command-line arguments taking precedence
    let cfg = setup_configuration(args, config_file);

    // Execute the main application logic
    if let Err(e) = run(&cfg) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Sets up the application configuration by merging command-line arguments
/// with a configuration file.
///
/// If the `--no-config` flag is present, only command-line arguments are used.
fn setup_configuration(args: cli::Args, config_file: config::FileConfig) -> config::Config {
    if args.no_config {
        // If `no_config` is set, use only the command-line arguments.
        ConfigBuilder::from(args).build()
    } else {
        // Otherwise, merge the configurations together.
        ConfigBuilder::from(args).merge(config_file.into()).build()
    }
}

/// Executes the main logic of the application.
fn run(cfg: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure the root path exists before proceeding
    if !cfg.root.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path does not exist: {}", &cfg.root.display()),
        )));
    }

    // Build the directory tree
    let mut builder = tree::TreeBuilder::new(cfg)?;
    let tree = builder.build(&cfg.root)?;

    // Format and print the tree to the standard output
    let formatter = formatter::get_formatter(&cfg.format);
    let output = formatter.format(&tree, cfg, builder.get_stats())?;
    println!("{}", output);

    Ok(())
}
