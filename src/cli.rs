use clap::Parser;

use crate::formatter::OutputFormat;
use crate::helpers;

/// Command line arguments for the fstree utility
///
/// This struct holds the configuration options that can be passed
/// to the program through command line arguments.
#[derive(Parser)]
#[command(author, version, about)]
pub struct Args {
    /// The directory path to generate the tree from
    #[clap(default_value = ".")]
    pub root: std::path::PathBuf,

    /// Show full path for each file entry
    #[clap(short, long)]
    pub full_path: bool,

    /// The prefix string to use for each level of the tree
    #[clap(short, long, default_value = "├── ")]
    pub prefix: String,

    /// The prefix string to use for the last entry of each branch
    #[clap(short, long, default_value = "└── ")]
    pub last_prefix: String,

    #[clap(short, long, default_value = "│   ")]
    pub child_prefix: String,

    /// Show all files and directories, including hidden files
    #[clap(short = 'a', long, alias = "all")]
    pub show_all: bool,

    /// Show only files that match the pattern (glob syntax)
    #[clap(short, long, alias = "pattern")]
    pub include: Option<String>,

    /// Exclude files that match the pattern (glob syntax)
    #[clap(short, long)]
    pub exclude: Option<String>,

    /// Custom ignore files
    #[clap(long, alias = "ignore-file")]
    pub ignore: Vec<String>,

    /// Show only directories
    #[clap(long, aliases = ["dir", "folder"])]
    pub directory: bool,

    /// Show directory and file count summary
    #[clap(short = 'r', long, alias = "report")]
    pub summary: bool,

    /// Show the filesize next to the name
    #[clap(short, long, alias = "filesize")]
    pub size: bool,

    /// The format to use for the filesize. e.g. Bytes (B), KiloBytes (KB), MegaBytes (MB), GigaBytes (GB) etc.
    #[clap(long, default_value = "bytes")]
    pub size_format: helpers::bytes::Format,

    /// The maximum depth to recurse
    #[clap(short = 'd', long, aliases = ["depth", "level"])]
    pub max_depth: Option<usize>,

    /// The output format to use (text, json, xml)
    #[clap(long, default_value = "text")]
    pub format: OutputFormat,

    /// Disable ANSI colors
    #[clap(long, alias="plain", default_value_t = std::env::var("NO_COLOR").is_ok())]
    pub no_color: bool,
}

/// Parses command line arguments into the Args struct
pub fn parse() -> Args {
    Args::parse()
}
