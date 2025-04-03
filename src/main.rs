mod cli;

fn main() {
    let args = cli::parse();
    println!("{}", &args.path.display());
}
