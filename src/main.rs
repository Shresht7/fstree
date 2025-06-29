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

/// The main entrypoint of the application
fn main() {
    // Load configuration from file and command-line arguments
    let file_config = config::load();
    let cli_args = cli::parse();

    // Merge configurations
    let app_config = config::merge_configs(file_config, cli_args);

    if let Err(e) = run(&app_config) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Implementation of the main run logic of the command-line
fn run(args: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
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
