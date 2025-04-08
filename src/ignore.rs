use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Sets up gitignore handling for the given root path
pub fn setup_gitignore<P: AsRef<std::path::Path>>(root: P) -> Result<Gitignore, ignore::Error> {
    let root = root.as_ref();

    // Instantiate the ignore::GitignoreBuilder
    let mut builder = GitignoreBuilder::new(root);

    // Ignore the .git folder
    builder.add_line(None, ".git")?;

    // Add the project's .gitignore file if it exists
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        builder.add(gitignore_path);
    }

    // Build the gitignore handler, falling back to an empty one on error
    Ok(builder.build()?)
}
