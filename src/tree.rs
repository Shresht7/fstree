use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::Config;
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
    pub path: PathBuf,
    pub node_type: NodeType,
    pub size: Option<u64>,
    pub children: Vec<TreeNode>,
}

/// A builder for constructing a file system tree
///
/// This builder walks a directory and constructs a `TreeNode` representation of the
/// file system, based on the provided configuration.
pub struct TreeBuilder<'a> {
    /// The configuration to use for the tree building process
    cfg: &'a Config,
    /// The root path from which the tree is built
    root: std::path::PathBuf,
    /// The file filter used to determine which files and directories to include in the tree
    file_filter: FileFilter,
    /// A set of visited paths to avoid processing the same path multiple times
    visited: HashSet<PathBuf>,
    /// The statistics collected during the tree building process
    stats: Statistics,
}

impl<'a> TreeBuilder<'a> {
    /// Creates a new `TreeBuilder` with the given configuration
    pub fn new(cfg: &'a Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            root: cfg.root.clone(),
            cfg,
            file_filter: FileFilter::new(cfg)?,
            stats: Statistics::default(),
            visited: HashSet::new(),
        })
    }

    /// Builds a `TreeNode` from the given path
    ///
    /// This method recursively walks the file system from the specified path and
    /// constructs a tree of `TreeNode` objects.
    pub fn build(&mut self, path: &Path) -> std::io::Result<TreeNode> {
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
            NodeType::Directory => self.process_directory(path, &mut node)?,
            NodeType::File | NodeType::SymbolicLink => {
                self.stats.add_files(1);
                if let Some(size) = size {
                    self.stats.add_byte_size(size);
                }
            }
        }

        Ok(node)
    }

    /// Returns a reference to the statistics collected during the tree build
    pub fn get_stats(&self) -> &Statistics {
        &self.stats
    }

    /// Processes a directory, reading its entries and recursively building the tree
    fn process_directory(&mut self, path: &Path, node: &mut TreeNode) -> std::io::Result<()> {
        // Check to see if we have already visited this directory (e.g. cyclic symlink)
        let canonical_path = path.canonicalize()?;
        if self.visited.contains(&canonical_path) {
            return Ok(()); // if we have, then skip processing it again
        }
        self.visited.insert(canonical_path); // Track that we've visited this path

        self.stats.add_dirs(1);

        // If we are still within the specified max-depth, keep recursing
        if self.is_within_max_depth(path) {
            node.children = self.read_dir(path)?;
        }

        Ok(())
    }

    /// Reads the entries of a directory and builds a vector of `TreeNode` children
    fn read_dir(&mut self, path: &Path) -> std::io::Result<Vec<TreeNode>> {
        self.file_filter
            .filter_entries(path)?
            .into_iter()
            .map(|entry| self.build(&entry.path()))
            .collect()
    }

    /// Checks if the current path is within the configured maximum depth
    fn is_within_max_depth(&self, path: &Path) -> bool {
        if self.cfg.max_depth.is_none() {
            return true;
        }

        let max_depth = self.cfg.max_depth.unwrap();
        let current_depth = path
            .strip_prefix(&self.root)
            .map(|p| p.components().count())
            .unwrap_or(0);

        current_depth < max_depth
    }
}
