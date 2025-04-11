//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

use std::{fs, os::windows::fs::MetadataExt};

use ::ignore::gitignore::Gitignore;
use globset::Glob;

mod cli;
mod helpers;
mod ignore;
mod stats;

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
fn run(args: &cli::Args) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the path actually exists
    if !std::fs::exists(&args.root)? {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("path does not exist: {}", &args.root.display().to_string()),
        )));
    }

    // Compile pattern matcher
    let pattern = if let Some(pat) = &args.pattern {
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
    let ignorer =
        ignore::setup_gitignore(&args.root, &args.ignore).unwrap_or_else(|_| Gitignore::empty());

    // Print the root
    println!("{}", args.root.display());

    // Initialize statistics
    let mut stats = stats::Statistics::default();

    // Traverse down the tree
    walk(
        &args.root,
        "",
        args,
        &pattern,
        &exclude_pattern,
        &ignorer,
        &mut stats,
    )?;

    // Print summary if requested
    if args.summary {
        println!("\n{}", stats);
    }

    Ok(())
}

/// Recursively walks through the directory structure and prints it
///
/// # Arguments
///
/// * `path` - Current directory path to process
/// * `prefix` - String prefix for the current level of indentation
/// * `args` - Command line arguments with options
/// * `ignorer` - Gitignore handler to check if files should be ignored
/// * `stats` - [`Statistics`] struct to track counts of directories and files
///
/// # Returns
///
/// * `std::io::Result<()>` - Success or IO error during traversal
fn walk<P: AsRef<std::path::Path>>(
    path: P,
    prefix: &str,
    args: &cli::Args,
    pattern: &Option<globset::GlobMatcher>,
    exclude_pattern: &Option<globset::GlobMatcher>,
    ignorer: &Gitignore,
    stats: &mut stats::Statistics,
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

        // Use absolute-path if `--full-path` option was passed
        let file_name = if args.full_path {
            std::path::absolute(entry.path())
                .and_then(|e| Ok(e.display().to_string()))
                .unwrap_or(file_name)
        } else {
            file_name
        };

        // Skip files if `--directory` was passed in
        if args.directory && !is_dir {
            continue;
        }

        // Check if the file matches the pattern, if a pattern is provided
        if let Some(pattern) = pattern {
            // Always include directories when using pattern matching, to maintain tree hierarchy
            if !is_dir && !pattern.is_match(&file_name) {
                continue;
            }
        }

        // Check if the file matches the exclude pattern, if provided
        if let Some(pattern) = exclude_pattern {
            // Always include directories when using pattern matching, to maintain tree hierarchy
            if !is_dir && pattern.is_match(&file_name) {
                continue;
            }
        }

        // Skip this entry if the path matches an ignored pattern
        if !args.show_all {
            if let Ok(rel_path) = path.strip_prefix(&args.root) {
                if ignorer.matched(rel_path, is_dir).is_ignore() {
                    continue;
                }
            }
        }

        // Update stats
        if is_dir {
            stats.add_dirs(1);
        } else {
            stats.add_files(1);
            let size = entry
                .metadata()
                .and_then(|x| Ok(x.file_size()))
                .unwrap_or(0);
            stats.add_byte_size(size);
        }

        // Determine the branch symbol based on whether this is the last entry
        let branch = if is_last {
            &args.last_prefix
        } else {
            &args.prefix
        };

        // Format the display name, appending a slash for directories
        let display_name = if is_dir {
            format!("{}/", file_name)
        } else {
            file_name
        };

        // Print the current entry with the appropriate prefix and branch symbol
        if !args.size {
            println!("{}{}{}", prefix, branch, display_name);
        } else {
            let bytes = entry.metadata().and_then(|e| Ok(e.len()));
            let size = match bytes {
                Ok(b) => helpers::bytes::format(b, &args.size_format),
                Err(_) => "--".into(),
            };
            println!("{}{}{} ({})", prefix, branch, display_name, size)
        }

        // If the entry is a directory, recursively process its contents
        if is_dir {
            // Respect max-depth, if specified in the arguments
            if let Some(max_depth) = args.max_depth {
                let current_depth = prefix.matches(&args.child_prefix).count();
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
            walk(
                &path,
                &child_prefix,
                args,
                pattern,
                exclude_pattern,
                ignorer,
                stats,
            )?;
        }
    }
    Ok(())
}
