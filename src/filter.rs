use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::config::Config;

pub struct FileFilter {
    root: PathBuf,
    only_directories: bool,
    show_all: bool,
    include_pattern: Option<GlobMatcher>,
    exclude_pattern: Option<GlobMatcher>,
    ignorer: Gitignore,
}

impl FileFilter {
    pub fn new(args: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Compile pattern matchers
        let include_pattern = args
            .include
            .as_ref()
            .map(|pat| Glob::new(pat))
            .transpose()?
            .map(|g| g.compile_matcher());

        let exclude_pattern = args
            .exclude
            .as_ref()
            .map(|pat| Glob::new(pat))
            .transpose()?
            .map(|g| g.compile_matcher());

        Ok(Self {
            root: args.root.clone(),
            only_directories: args.directory,
            show_all: args.show_all,
            include_pattern,
            exclude_pattern,
            ignorer: Self::setup_gitignore(&args.root, &args.ignore)?,
        })
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
        Ok(builder.build()?)
    }

    pub fn should_include(&self, entry: &std::fs::DirEntry) -> bool {
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

    pub fn filter_entries(&self, path: &Path) -> std::io::Result<Vec<std::fs::DirEntry>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| self.should_include(entry))
            .collect())
    }
}
