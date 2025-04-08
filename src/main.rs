//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

use std::fs;

use ignore::gitignore::{Gitignore, GitignoreBuilder};

mod cli;

/// The main entrypoint of the application
fn main() {
    let args = cli::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Implementation of the main run logic of the command-line
///
/// # Arguments
///
/// * `args` - Command line arguments containing path and formatting options
///
/// # Returns
///
/// * `std::io::Result<()>` - Success or IO error during directory traversal
fn run(args: &cli::Args) -> std::io::Result<()> {
    // Print the root
    println!("{}", args.path.display());

    // Setup ignore rules
    let ignorer = setup_gitignore(&args.path).unwrap_or_else(|_| Gitignore::empty());

    // Traverse down the tree
    walk(&args.path, "", args, &ignorer)?;
    Ok(())
}

/// Sets up gitignore handling for the given root path
fn setup_gitignore<P: AsRef<std::path::Path>>(root: P) -> Result<Gitignore, ignore::Error> {
    let root = root.as_ref();

    // Instantiate the ignore::GitignoreBuilder
    let mut builder = GitignoreBuilder::new(root);

    // Ignore the .git folder
    builder.add_line(None, ".git")?;

    // Add the project's .gitignore file if it exists
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        builder.add(gitignore_path);
    }

    // Build the gitignore handler, falling back to an empty one on error
    Ok(builder.build()?)
}

/// Recursively walks through the directory structure and prints it
///
/// # Arguments
///
/// * `path` - Current directory path to process
/// * `prefix` - String prefix for the current level of indentation
/// * `args` - Command line arguments with options
/// * `ignorer` - Gitignore handler to check if files should be ignored
///
/// # Returns
///
/// * `std::io::Result<()>` - Success or IO error during traversal
fn walk<P: AsRef<std::path::Path>>(
    path: P,
    prefix: &str,
    args: &cli::Args,
    ignorer: &Gitignore,
) -> std::io::Result<()> {
    // Read the directory entries
    let entries = fs::read_dir(&path)?.collect::<Result<Vec<_>, _>>()?;

    // Iterate over each entry in the directory
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let is_dir = file_type.is_dir();

        // Skip this entry if the path matches an ignored pattern
        if let Ok(rel_path) = path.strip_prefix(&args.path) {
            if ignorer.matched(rel_path, is_dir).is_ignore() {
                continue;
            }
        }

        // Determine the branch symbol based on whether this is the last entry
        let branch = if is_last { "└── " } else { &args.prefix };

        // Format the display name, appending a slash for directories
        let display_name = if is_dir {
            format!("{}/", file_name)
        } else {
            file_name
        };

        // Print the current entry with the appropriate prefix and branch symbol
        println!("{}{}{}", prefix, branch, display_name);

        // If the entry is a directory, recursively process its contents
        if is_dir {
            // Respect max-depth, if specified in the arguments
            if let Some(max_depth) = args.max_depth {
                let current_depth = prefix.matches("│   ").count();
                if current_depth >= max_depth {
                    continue;
                }
            }

            // Create a new prefix for the child entries
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            // Recursively walk the child directory
            walk(&path, &child_prefix, args, ignorer)?;
        }
    }
    Ok(())
}
