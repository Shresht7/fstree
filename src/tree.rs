use std::collections::HashSet;
use std::path::PathBuf;

use serde::Serialize;

use crate::AppConfig;
use crate::filter::FileFilter;
use crate::stats::Statistics;

/// Represents the type of a file system node
#[derive(Serialize)]
pub enum NodeType {
    File,
    Directory,
    SymbolicLink,
}

/// Represents a node in the file system tree
///
/// Each `TreeNode` contains information about a file or directory, including its name,
/// path, type, size (if applicable), and its children nodes (if it is a directory).
#[derive(Serialize)]
pub struct TreeNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub node_type: NodeType,
    pub size: Option<u64>,
    pub children: Vec<TreeNode>,
}

/// The builder responsible for constructing a file system tree based on the provided command line arguments
pub struct TreeBuilder<'a> {
    /// The command line arguments used to configure the tree building process
    pub args: &'a AppConfig,
    /// The root path from which the tree is built
    root: std::path::PathBuf,
    /// The file filter used to determine which files and directories to include in the tree
    file_filter: FileFilter,
    /// A set of visited paths to avoid processing the same path multiple times
    visited: HashSet<PathBuf>,
    /// The statistics collected during the tree building process
    pub stats: Statistics,
}

impl<'a> TreeBuilder<'a> {
    /// Creates a new `TreeBuilder` with the provided command line arguments.
    pub fn new(args: &'a AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            root: args.root.clone(),
            args,
            file_filter: FileFilter::new(args)?,
            stats: Statistics::default(),
            visited: HashSet::new(),
        })
    }

    /// Builds a `TreeNode` representing the file system structure starting from the given path.
    pub fn build<P: AsRef<std::path::Path>>(&mut self, path: P) -> std::io::Result<TreeNode> {
        let path = path.as_ref();
        let metadata = std::fs::symlink_metadata(path)?;

        let file_type = metadata.file_type();
        let node_type = if file_type.is_dir() {
            NodeType::Directory
        } else if file_type.is_symlink() {
            NodeType::SymbolicLink
        } else {
            NodeType::File
        };

        let size = if !metadata.is_dir() {
            Some(metadata.len())
        } else {
            None
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let mut node = TreeNode {
            name,
            path: path.to_path_buf(),
            node_type,
            size,
            children: Vec::new(),
        };

        match node.node_type {
            NodeType::Directory => {
                let canonical_path = path.canonicalize()?;
                if self.visited.contains(&canonical_path) {
                    return Ok(node);
                }
                self.visited.insert(canonical_path);

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
            }

            NodeType::File | NodeType::SymbolicLink => {
                self.stats.add_files(1);
                if let Some(size) = size {
                    self.stats.add_byte_size(size);
                }
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
