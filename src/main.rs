//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

use std::error::Error;

mod cli;
mod filter;
mod formatter;
mod helpers;
mod stats;
mod tree;

/// The main entrypoint of the application
fn main() {
    let args = cli::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Implementation of the main run logic of the command-line
fn run(args: &cli::Args) -> Result<(), Box<dyn Error>> {
    // Check if the path actually exists
    if !std::fs::exists(&args.root)? {
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
    let output = formatter.format(&tree, args)?;
    println!("{}", output);

    // Print summary if requested
    if args.summary {
        println!("\n{}", builder.get_stats());
    }

    Ok(())
}
