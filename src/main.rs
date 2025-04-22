//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

use globset::Glob;

mod cli;
mod formatter;
mod helpers;
mod ignore;
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
fn run(args: &cli::Args) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the path actually exists
    if !std::fs::exists(&args.root)? {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path does not exist: {}", &args.root.display().to_string()),
        )));
    }

    // Compile pattern matcher
    let include_pattern = if let Some(pat) = &args.include {
        Some(Glob::new(pat)?.compile_matcher())
    } else {
        None
    };
    let exclude_pattern = if let Some(pat) = &args.exclude {
        Some(Glob::new(pat)?.compile_matcher())
    } else {
        None
    };

    // Setup ignore rules
    let ignorer = ignore::setup_gitignore(&args.root, &args.ignore)
        .unwrap_or_else(|_| ignore::Gitignore::empty());

    // Initialize statistics
    let mut stats = stats::Statistics::default();

    // Build the tree
    let mut builder = tree::TreeBuilder::new(
        args,
        &include_pattern,
        &exclude_pattern,
        &ignorer,
        &mut stats,
    );
    let tree = builder.build(&args.root)?;

    // Format and print the tree
    let formatter = formatter::get_formatter(&args.format);
    let output = formatter.format(&tree, args)?;
    println!("{}", output);

    // Print summary if requested
    if args.summary {
        println!("\n{}", stats);
    }

    Ok(())
}
