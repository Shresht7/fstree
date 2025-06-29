//! Describes the command-line interface

use clap::Parser;

use crate::formatter::OutputFormat;
use crate::helpers;

/// Command line arguments for the fstree utility
///
/// This struct holds the configuration options that can be passed
/// to the program through command line arguments.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// The directory path to generate the tree from
    pub root: Option<std::path::PathBuf>,

    /// Show full path for each file entry
    #[clap(short, long)]
    pub full_path: bool,

    /// The prefix string to use for each level of the tree
    #[clap(short, long)]
    pub prefix: Option<String>,

    /// The prefix string to use for the last entry of each branch
    #[clap(short, long)]
    pub last_prefix: Option<String>,

    #[clap(short, long)]
    pub child_prefix: Option<String>,

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
    pub ignore: Option<Vec<String>>,

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
    #[clap(long)]
    pub size_format: Option<helpers::bytes::Format>,

    /// The maximum depth to recurse
    #[clap(short = 'd', long, aliases = ["depth", "level"])]
    pub max_depth: Option<usize>,

    /// The output format to use (text, json, xml)
    #[clap(long)]
    pub format: Option<OutputFormat>,

    /// Disable ANSI colors
    #[clap(long, alias = "plain")]
    pub no_color: bool,

    /// Disables loading the configuration file
    #[clap(long, alias = "nocfg")]
    pub no_config: bool,
}

/// Parses command line arguments into the Args struct
pub fn parse() -> Args {
    Args::parse()
}
