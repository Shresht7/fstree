use std::io;

use crate::config::Config;
use crate::helpers;
use crate::helpers::ansi::{Ansi, AnsiString};
use crate::tree::{NodeType, TreeNode};

/// Defines the interface for different output formatters
pub trait Formatter {
    /// Formats the given tree node into a string representation
    fn format(
        &self,
        node: &TreeNode,
        cfg: &Config,
        stats: &crate::stats::Statistics,
    ) -> io::Result<String>;
}

/// Implements text-based tree formatting
pub struct TextFormatter;

impl TextFormatter {
    /// Recursively formats a tree node and its children
    ///
    /// `prefix`: The indentation string for the current level. (Used in recursive calls)
    /// `is_last`: True if the node is the last child of its parent, influencing branch characters
    /// `cfg`: The configuration that control formatting options
    fn format_node(&self, node: &TreeNode, prefix: &str, is_last: bool, cfg: &Config) -> String {
        let mut output = String::new();

        // Determine the correct branch character (├── or └──)
        let branch = if is_last {
            &cfg.last_prefix
        } else {
            &cfg.prefix
        };

        // Determine the display name based on the node type
        let display_name = self.format_display_name(node, cfg, !cfg.no_color);

        // Construct the current line with prefix, branch, and name
        let mut line = format!("{prefix}{branch}{display_name}");

        // Add file size if requested
        if cfg.size {
            if let Some(size) = node.size {
                line.push_str(&format!(
                    " ({})",
                    helpers::bytes::format(size, &cfg.size_format)
                ));
            }
        }

        output.push_str(&line);
        output.push('\n');

        // Determine the prefix for children based on whether the current node is the last
        let child_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{}{}", prefix, &cfg.child_prefix)
        };

        // Recursively format children
        for (i, child) in node.children.iter().enumerate() {
            output.push_str(&self.format_node(
                child,
                &child_prefix,
                i == node.children.len() - 1, // Check if this child is the last
                cfg,
            ));
        }

        output
    }

    /// Returns the display name for a `TreeNode` based on its type
    fn format_display_name(&self, node: &TreeNode, cfg: &Config, ansi: bool) -> String {
        let name = if cfg.full_path {
            node.path.to_string_lossy().to_string()
        } else {
            node.name.clone()
        };

        match node.node_type {
            NodeType::File => {
                if ansi {
                    name.ansi(&[Ansi::BrightWhite])
                } else {
                    name
                }
            }
            NodeType::Directory => {
                if ansi {
                    format!(" {name} ").ansi(&[Ansi::Bold, Ansi::BgYellow])
                } else {
                    format!("{name}/")
                }
            }
            NodeType::SymbolicLink => {
                let target = std::fs::read_link(&node.path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "<unreadable>".to_string());
                if ansi {
                    format!("{name} -> {target}").ansi(&[Ansi::BrightCyan])
                } else {
                    format!("{name} -> {target}")
                }
            }
        }
    }
}

impl Formatter for TextFormatter {
    /// Formats the entire tree, handling the root node specially (no initial indentation)
    fn format(
        &self,
        node: &TreeNode,
        cfg: &Config,
        stats: &crate::stats::Statistics,
    ) -> io::Result<String> {
        let mut output = String::new();

        // Handle the root node without any prefix/indentation
        let mut line = self.format_display_name(node, cfg, !cfg.no_color);

        // Add file size to root if requested
        if cfg.size {
            if let Some(size) = node.size {
                line.push_str(&format!(
                    " ({})",
                    helpers::bytes::format(size, &cfg.size_format)
                ));
            }
        }

        output.push_str(&line);
        output.push('\n');

        // Recursively format children of the root node
        // The initial prefix for children is an empty string, as they will handle their own indentation
        for (i, child) in node.children.iter().enumerate() {
            output.push_str(&self.format_node(
                child,
                "", // Children of the root start with no prefix, format_node handles their indentation
                i == node.children.len() - 1,
                cfg,
            ));
        }

        // Append summary if requested
        if cfg.summary {
            output.push('\n');
            output.push_str(&stats.to_string());
        }

        Ok(output)
    }
}

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(
        &self,
        node: &TreeNode,
        _cfg: &Config,
        stats: &crate::stats::Statistics,
    ) -> io::Result<String> {
        let output = serde_json::json!({
            "root": node,
            "stats": stats,
        });
        serde_json::to_string_pretty(&output).map_err(io::Error::other)
    }
}

/// Returns the appropriate formatter based on the requested output format
pub fn get_formatter(format: &OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Text => Box::new(TextFormatter),
        OutputFormat::Json => Box::new(JsonFormatter),
    }
}

/// Defines the supported output formats for the tree
#[derive(Clone, Debug, serde::Deserialize)]
pub enum OutputFormat {
    Text,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            e => Err(format!("Unknown output format: {e}")),
        }
    }
}
