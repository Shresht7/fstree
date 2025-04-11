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
    let ignorer =
        ignore::setup_gitignore(&args.root, &args.ignore).unwrap_or_else(|_| Gitignore::empty());

    // Initialize statistics
    let mut stats = stats::Statistics::default();

    // Print the root
    println!("{}", args.root.display());

    // Traverse down the tree
    walk(
        &args.root,
        "",
        args,
        &include_pattern,
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
fn walk<P: AsRef<std::path::Path>>(
    path: P,
    prefix: &str,
    args: &cli::Args,
    include_pattern: &Option<globset::GlobMatcher>,
    exclude_pattern: &Option<globset::GlobMatcher>,
    ignorer: &Gitignore,
    stats: &mut stats::Statistics,
) -> std::io::Result<()> {
    // Read the directory entries
    let entries = read_and_filter_entries(path, args, include_pattern, exclude_pattern, ignorer)?;

    // Iterate over each entry in the directory
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let file_type = entry.file_type()?;
        let is_dir = file_type.is_dir();

        let abs_path = entry.path();
        let mut file_name = entry.file_name().to_string_lossy().to_string();

        if args.full_path {
            file_name = std::path::absolute(&abs_path)
                .map(|p| p.display().to_string())
                .unwrap_or(file_name.clone());
        }

        // Update stats
        update_stats(entry, is_dir, stats);

        // Print entry
        print_entry(&file_name, &prefix, is_last, is_dir, args, entry)?;

        // Recursively walk children
        if is_dir {
            if let Some(max_depth) = args.max_depth {
                let current_depth = prefix.matches(&args.child_prefix).count();
                if current_depth >= max_depth {
                    continue;
                }
            }

            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            walk(
                &abs_path,
                &child_prefix,
                args,
                include_pattern,
                exclude_pattern,
                ignorer,
                stats,
            )?;
        }
    }

    Ok(())
}

fn read_and_filter_entries<P: AsRef<std::path::Path>>(
    path: P,
    args: &cli::Args,
    include_pattern: &Option<globset::GlobMatcher>,
    exclude_pattern: &Option<globset::GlobMatcher>,
    ignorer: &Gitignore,
) -> std::io::Result<Vec<std::fs::DirEntry>> {
    Ok(fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter(|entry| {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => return false, // Skip entry if the file-type could not be determined
            };
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_dir = file_type.is_dir();

            // Skip non-directories if --directory flag is passed
            if args.directory && !is_dir {
                return false;
            }

            // Include pattern
            if let Some(pattern) = include_pattern {
                if !is_dir && !pattern.is_match(&file_name) {
                    return false;
                }
            }

            // Exclude pattern
            if let Some(pattern) = exclude_pattern {
                if !is_dir && pattern.is_match(&file_name) {
                    return false;
                }
            }

            // Ignore files based on .gitignore rules
            if !args.show_all {
                if let Ok(rel_path) = entry.path().strip_prefix(&args.root) {
                    if ignorer.matched(rel_path, is_dir).is_ignore() {
                        return false;
                    }
                }
            }

            true
        })
        .collect::<Vec<_>>())
}

fn update_stats(entry: &fs::DirEntry, is_dir: bool, stats: &mut stats::Statistics) {
    if is_dir {
        stats.add_dirs(1);
    } else {
        stats.add_files(1);
        let size = entry.metadata().map(|m| m.file_size()).unwrap_or(0);
        stats.add_byte_size(size);
    }
}

fn print_entry(
    file_name: &str,
    prefix: &str,
    is_last: bool,
    is_dir: bool,
    args: &cli::Args,
    entry: &fs::DirEntry,
) -> std::io::Result<()> {
    let branch = if is_last {
        &args.last_prefix
    } else {
        &args.prefix
    };

    let display_name = if is_dir {
        format!("{}/", file_name)
    } else {
        file_name.to_string()
    };

    if args.size {
        let size = entry
            .metadata()
            .map(|m| helpers::bytes::format(m.len(), &args.size_format))
            .unwrap_or_else(|_| "--".into());

        println!("{}{}{} ({})", prefix, branch, display_name, size);
    } else {
        println!("{}{}{}", prefix, branch, display_name);
    }

    Ok(())
}
