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
    for entry in walker {
        match entry {
            Ok(entry) => {
                let prefix = args.prefix.repeat(entry.depth());
                output.push_str(&format!(
                    "{prefix}{}\n",
                    entry.file_name().to_string_lossy()
                ));
            }
            Err(_) => {} // Ignore errors in traversal for now
        }
    }

    println!("{}", output);
    Ok(())
}
