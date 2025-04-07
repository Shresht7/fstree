//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

use std::fs;

mod cli;

/// The main entrypoint of the application
fn main() {
    let args = cli::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Processes the directory tree and generates the formatted output
///
/// # Arguments
///
/// * `args` - Command line arguments containing path and formatting options
///
/// # Returns
///
/// * `std::io::Result<()>` - Success or IO error during directory traversal
fn run(args: &cli::Args) -> std::io::Result<()> {
    println!("{}", args.path.display()); // Root
    walk(&args.path, "", args)?; // Traverse down the tree
    Ok(())
}

fn walk<P: AsRef<std::path::Path>>(path: P, prefix: &str, args: &cli::Args) -> std::io::Result<()> {
    // Read the directory entries
    let entries = fs::read_dir(&path)?.collect::<Result<Vec<_>, _>>()?;

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let is_dir = file_type.is_dir();

        let branch = if is_last { "└── " } else { &args.prefix };
        let display_name = if is_dir {
            format!("{}/", file_name)
        } else {
            file_name
        };

        println!("{}{}{}", prefix, branch, display_name);

        // If it is a directory, recursively process it
        if is_dir {
            // Respect max-depth, if specified
            if let Some(max_depth) = args.max_depth {
                let current_depth = prefix.matches("│   ").count();
                if current_depth >= max_depth {
                    continue;
                }
            }

            // Create new prefix for children
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            walk(&path, &child_prefix, args)?;
        }
    }
    Ok(())
}
