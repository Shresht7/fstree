use clap::Parser;

/// Command line arguments for the fstree utility
///
/// This struct holds the configuration options that can be passed
/// to the program through command line arguments.
#[derive(Parser)]
pub struct Args {
    /// The directory path to generate the tree from
    #[clap(default_value = ".")]
    pub path: std::path::PathBuf,

    /// The prefix string to use for each level of the tree
    #[clap(short, long, default_value = "├── ")]
    pub prefix: String,

    /// Include hidden files
    #[clap(short = 'a', long, alias = "all")]
    pub hidden: bool,

    /// The maximum depth to recurse
    #[clap(short = 'd', long)]
    pub max_depth: Option<usize>,
}

/// Parses command line arguments into the Args struct
pub fn parse() -> Args {
    Args::parse()
}

// └── for the last entry
