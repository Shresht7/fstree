use globset::GlobMatcher;
use ignore::gitignore::Gitignore;

use crate::stats::Statistics;

pub struct TreeNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub children: Vec<TreeNode>,
}

pub struct TreeBuilder<'a> {
    pub args: &'a crate::cli::Args,
    pub include_pattern: &'a Option<GlobMatcher>,
    pub exclude_pattern: &'a Option<GlobMatcher>,
    pub ignorer: &'a Gitignore,
    pub stats: Statistics,
}

impl<'a> TreeBuilder<'a> {
    pub fn new(
        args: &'a crate::cli::Args,
        include_pattern: &'a Option<GlobMatcher>,
        exclude_pattern: &'a Option<GlobMatcher>,
        ignorer: &'a Gitignore,
    ) -> Self {
        Self {
            args,
            include_pattern,
            exclude_pattern,
            ignorer,
            stats: Statistics::default(),
        }
    }

    pub fn build<P: AsRef<std::path::Path>>(&mut self, path: P) -> std::io::Result<TreeNode> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let is_dir = metadata.is_dir();
        let size = if !is_dir { Some(metadata.len()) } else { None };
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let mut node = TreeNode {
            name,
            path: path.to_path_buf(),
            is_dir,
            size,
            children: Vec::new(),
        };

        if is_dir {
            self.stats.add_dirs(1);
            // Only traverse directory if within max_depth
            if let Some(max_depth) = self.args.max_depth {
                let current_depth = path
                    .components()
                    .filter(|c| matches!(c, std::path::Component::Normal(_)))
                    .count();
                if current_depth <= max_depth {
                    node.children = self.read_dir(path)?;
                }
            } else {
                node.children = self.read_dir(path)?;
            }
        } else {
            self.stats.add_files(1);
            if let Some(size) = size {
                self.stats.add_byte_size(size);
            }
        }

        Ok(node)
    }

    fn read_dir<P: AsRef<std::path::Path>>(&mut self, path: P) -> std::io::Result<Vec<TreeNode>> {
        let entries = self.filter_entries(path.as_ref())?;
        let mut children = Vec::new();

        for entry in entries {
            if let Ok(child) = self.build(entry.path()) {
                children.push(child);
            }
        }

        Ok(children)
    }

    fn filter_entries(&self, path: &std::path::Path) -> std::io::Result<Vec<std::fs::DirEntry>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| {
                let file_type = match entry.file_type() {
                    Ok(ft) => ft,
                    Err(_) => return false,
                };
                let file_name = entry.file_name().to_string_lossy().to_string();
                let is_dir = file_type.is_dir();

                if self.args.directory && !is_dir {
                    return false;
                }

                if let Some(pattern) = self.include_pattern {
                    if !is_dir && !pattern.is_match(&file_name) {
                        return false;
                    }
                }

                if let Some(pattern) = self.exclude_pattern {
                    if !is_dir && pattern.is_match(&file_name) {
                        return false;
                    }
                }

                if !self.args.show_all {
                    if let Ok(rel_path) = entry.path().strip_prefix(&self.args.root) {
                        if self.ignorer.matched(rel_path, is_dir).is_ignore() {
                            return false;
                        }
                    }
                }

                true
            })
            .collect())
    }

    pub fn get_stats(&self) -> &Statistics {
        &self.stats
    }
}
