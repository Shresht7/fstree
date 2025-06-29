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
    // Parse the command-line arguments
    let args = cli::parse();

    // Merge configurations from command-line arguments and configuration file
    let cfg = setup_configuration(args);

    if let Err(e) = run(&cfg) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Sets up the configuration for the application
fn setup_configuration(args: cli::Args) -> config::Config {
    if args.no_config {
        // If `no_config` is set, use only the command-line arguments
        ConfigBuilder::from(args).build()
    } else {
        // Load the configuration file and merge it with the command-line arguments
        let config_file = config::load_file();
        ConfigBuilder::from(args).merge(config_file.into()).build()
    }
}

/// Implementation of the main run logic of the command-line
fn run(cfg: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the path actually exists
    if !std::fs::metadata(&cfg.root).is_ok() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path does not exist: {}", &cfg.root.display().to_string()),
        )));
    }

    // Build the tree
    let mut builder = tree::TreeBuilder::new(cfg)?;
    let tree = builder.build(&cfg.root)?;

    // Format and print the tree
    let formatter = formatter::get_formatter(&cfg.format);
    let output = formatter.format(&tree, cfg, builder.get_stats())?;
    println!("{}", output);

    Ok(())
}
