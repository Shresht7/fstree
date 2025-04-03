use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(default_value = ".")]
    pub path: std::path::PathBuf,
}

pub fn parse() -> Args {
    Args::parse()
}
