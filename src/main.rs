//! A command-line utility that displays directory structure in a tree-like format
//!
//! This program walks through directories and displays their contents in a
//! hierarchical tree structure, similar to the Unix tree command.

mod cli;

/// The main entrypoint of the application
fn main() {
    let args = cli::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
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
    let mut output = String::new();

    let walker = ignore::WalkBuilder::new(&args.path)
        .hidden(!args.hidden)
        .build();
    for entry in walker {
        match entry {
            Ok(entry) => {
                let prefix = args.prefix.repeat(entry.depth());
                output.push_str(&format_entry(prefix, &entry));
            }
            Err(_) => {} // Ignore errors in traversal for now
        }
    }

    println!("{}", output);
    Ok(())
}

/// Formats a single directory entry with the appropriate prefix
///
/// # Arguments
///
/// * `prefix` - The string prefix to use for the current tree level
/// * `entry` - The directory entry to format
///
/// # Returns
///
/// * String - The formatted entry string with appropriate prefix and newline
fn format_entry(prefix: String, entry: &ignore::DirEntry) -> String {
    let display = if entry.file_type().is_some_and(|f| f.is_dir()) {
        format!("{}/", entry.file_name().to_string_lossy())
    } else {
        entry.file_name().to_string_lossy().to_string()
    };
    format!("{prefix}{}\n", display)
}
