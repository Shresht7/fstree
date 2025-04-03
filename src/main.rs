mod cli;

fn main() {
    let args = cli::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
    }
}

fn run(args: &cli::Args) -> std::io::Result<()> {
    let walker = ignore::WalkBuilder::new(&args.path).build();
    for path in walker {
        match path {
            Ok(path) => {
                let prefix = "  ".repeat(path.depth());
                println!("{prefix}{}", path.file_name().to_string_lossy())
            }
            Err(_) => {} // Ignore errors in traversal for now
        }
    }
    Ok(())
}
