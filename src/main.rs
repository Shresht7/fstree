mod cli;

fn main() {
    let args = cli::parse();
    if let Err(e) = run(&args) {
        eprintln!("Error: {e}");
    }
}

fn run(args: &cli::Args) -> std::io::Result<()> {
    let mut output = String::new();

    let walker = ignore::WalkBuilder::new(&args.path).build();
    for path in walker {
        match path {
            Ok(path) => {
                let prefix = "  ".repeat(path.depth());
                output.push_str(&format!("{prefix}{}\n", path.file_name().to_string_lossy()));
            }
            Err(_) => {} // Ignore errors in traversal for now
        }
    }

    println!("{}", output);
    Ok(())
}
