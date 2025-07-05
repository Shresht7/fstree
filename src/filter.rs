use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::config::Config;

/// A filter for file system entries.
///
/// This filter is responsible for determining which files and directories should be
/// included in the output, based on the user's configuration.
pub struct FileFilter {
    root: PathBuf,
    only_directories: bool,
    show_all: bool,
    include_pattern: Option<GlobMatcher>,
    exclude_pattern: Option<GlobMatcher>,
    ignorer: Gitignore,
}

impl FileFilter {
    /// Creates a new `FileFilter` with the given configuration.
    pub fn new(cfg: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            root: cfg.root.clone(),
            only_directories: cfg.directory,
            show_all: cfg.show_all,
            include_pattern: Self::compile_glob(&cfg.include)?,
            exclude_pattern: Self::compile_glob(&cfg.exclude)?,
            ignorer: Self::setup_gitignore(&cfg.root, &cfg.ignore)?,
        })
    }

    /// Compiles a glob pattern into a `GlobMatcher`
    fn compile_glob(pattern: &Option<String>) -> Result<Option<GlobMatcher>, globset::Error> {
        pattern
            .as_ref()
            .map(|pat| Glob::new(pat))
            .transpose()
            .map(|g| g.map(|glob| glob.compile_matcher()))
    }

    /// Filters a directory's entries, returning a vector of included entries.
    pub fn filter_entries(&self, path: &Path) -> std::io::Result<Vec<std::fs::DirEntry>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| self.should_include(entry))
            .collect())
    }

    /// Sets up gitignore handling for the given root path
    fn setup_gitignore(root: &Path, ignore_files: &[String]) -> Result<Gitignore, ignore::Error> {
        // Instantiate the ignore::GitignoreBuilder
        let mut builder = GitignoreBuilder::new(root);

        // Ignore the .git folder
        builder.add_line(None, ".git")?;

        // Add the project's .gitignore file if it exists
        let gitignore_path = root.join(".gitignore");
        if gitignore_path.exists() {
            builder.add(gitignore_path);
        }

        // Add custom ignore files
        for ignore in ignore_files {
            let path = root.join(ignore);
            if path.exists() {
                builder.add(path);
            }
        }

        // Build the gitignore handler
        builder.build()
    }

    /// Checks if a given directory entry should be included in the output.
    fn should_include(&self, entry: &std::fs::DirEntry) -> bool {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => return false,
        };

        let is_dir = file_type.is_dir();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Directory-only filter
        if self.only_directories && !is_dir {
            return false;
        }

        // Include pattern filter (skip directories)
        if let Some(pattern) = &self.include_pattern {
            if !is_dir && !pattern.is_match(&file_name) {
                return false;
            }
        }

        // Exclude pattern filter (skip directories)
        if let Some(pattern) = &self.exclude_pattern {
            if !is_dir && pattern.is_match(&file_name) {
                return false;
            }
        }

        // Gitignore filter
        if !self.show_all {
            if let Ok(rel_path) = entry.path().strip_prefix(&self.root) {
                if self.ignorer.matched(rel_path, is_dir).is_ignore() {
                    return false;
                }
            }
        }

        true
    }
}
