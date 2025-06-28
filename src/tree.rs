use crate::cli::Args;
use crate::filter::FileFilter;
use crate::stats::Statistics;

/// Represents a node in the file system tree.
///
/// Each `TreeNode` contains information about a file or directory, including its name,
/// path, whether it is a directory, its size (if applicable), and its children nodes
/// (if it is a directory).
pub struct TreeNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub children: Vec<TreeNode>,
}

pub struct TreeBuilder<'a> {
    pub args: &'a Args,
    file_filter: FileFilter,
    pub stats: Statistics,
    root: std::path::PathBuf,
}

impl<'a> TreeBuilder<'a> {
    /// Creates a new `TreeBuilder` with the provided command line arguments.
    pub fn new(args: &'a Args) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            root: args.root.clone(),
            args,
            file_filter: FileFilter::new(args)?,
            stats: Statistics::default(),
        })
    }

    /// Builds a `TreeNode` representing the file system structure starting from the given path.
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
                    .strip_prefix(&self.root)
                    .map(|p| {
                        p.components()
                            .filter(|c| matches!(c, std::path::Component::Normal(_)))
                            .count()
                    })
                    .unwrap_or(0);
                if current_depth < max_depth {
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

    /// Reads the directory entries at the specified path and builds `TreeNode` children for each entry.
    fn read_dir<P: AsRef<std::path::Path>>(&mut self, path: P) -> std::io::Result<Vec<TreeNode>> {
        let entries = self.file_filter.filter_entries(path.as_ref())?;
        let mut children = Vec::new();

        for entry in entries {
            if let Ok(child) = self.build(entry.path()) {
                children.push(child);
            }
        }

        Ok(children)
    }

    /// Returns the statistics collected during the tree building process.
    pub fn get_stats(&self) -> &Statistics {
        &self.stats
    }
}
