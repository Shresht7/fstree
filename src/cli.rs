use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(default_value = ".")]
    pub path: std::path::PathBuf,

    #[clap(short, long, default_value = "|-- ")]
    pub prefix: String,
}

pub fn parse() -> Args {
    Args::parse()
}
