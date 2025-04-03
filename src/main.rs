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
                output.push_str(&format_entry(prefix, &entry));
            }
            Err(_) => {} // Ignore errors in traversal for now
        }
    }

    println!("{}", output);
    Ok(())
}

fn format_entry(prefix: String, entry: &ignore::DirEntry) -> String {
    format!("{prefix}{}\n", entry.file_name().to_string_lossy())
}
