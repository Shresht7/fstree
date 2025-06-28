use std::io;

use crate::cli::Args;
use crate::helpers;
use crate::helpers::ansi::{ANSI, ANSIString};
use crate::tree::{NodeType, TreeNode};

/// Defines the interface for different output formatters
pub trait Formatter {
    /// Formats the given tree node into a string representation
    fn format(
        &self,
        node: &TreeNode,
        args: &Args,
        stats: &crate::stats::Statistics,
    ) -> io::Result<String>;
}

/// Implements text-based tree formatting
pub struct TextFormatter;

impl TextFormatter {
    /// Returns the display name for a `TreeNode` based on its type
    fn get_display_name(&self, node: &TreeNode, ansi: bool) -> String {
        match node.node_type {
            NodeType::File => {
                let name = node.name.clone();
                if ansi {
                    name.ansi(&[ANSI::BrightWhite])
                } else {
                    name
                }
            }
            NodeType::Directory => {
                if ansi {
                    format!(" {} ", node.name).ansi(&[ANSI::Bold, ANSI::BgYellow])
                } else {
                    format!("{}/", node.name)
                }
            }
            NodeType::SymbolicLink => {
                let target = std::fs::read_link(&node.path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "<unreadable>".to_string());
                if ansi {
                    format!("{} -> {}", node.name, target).ansi(&[ANSI::BrightCyan])
                } else {
                    format!("{} -> {}", node.name, target)
                }
            }
        }
    }

    /// Recursively formats a tree node and its children
    ///
    /// `prefix`: The indentation string for the current level. (Used in recursive calls)
    /// `is_last`: True if the node is the last child of its parent, influencing branch characters
    /// `args`: The command line arguments that control formatting options
    fn format_node(&self, node: &TreeNode, prefix: &str, is_last: bool, args: &Args) -> String {
        let mut output = String::new();

        // Determine the correct branch character (├── or └──)
        let branch = if is_last {
            &args.last_prefix
        } else {
            &args.prefix
        };

        // Determine the display name based on the node type
        let display_name = self.get_display_name(node, !args.no_color);

        // Construct the current line with prefix, branch, and name
        let mut line = format!("{}{}{}", prefix, branch, display_name);

        // Add file size if requested
        if args.size {
            if let Some(size) = node.size {
                line.push_str(&format!(
                    " ({})",
                    helpers::bytes::format(size, &args.size_format)
                ));
            }
        }

        output.push_str(&line);
        output.push('\n');

        // Determine the prefix for children based on whether the current node is the last
        let child_prefix = if is_last {
            format!("{}    ", prefix) // No vertical line for the last child's children
        } else {
            format!("{}│   ", prefix) // Vertical line for non-last children
        };

        // Recursively format children
        for (i, child) in node.children.iter().enumerate() {
            output.push_str(&self.format_node(
                child,
                &child_prefix,
                i == node.children.len() - 1, // Check if this child is the last
                args,
            ));
        }

        output
    }
}

impl Formatter for TextFormatter {
    /// Formats the entire tree, handling the root node specially (no initial indentation)
    fn format(
        &self,
        node: &TreeNode,
        args: &Args,
        stats: &crate::stats::Statistics,
    ) -> io::Result<String> {
        let mut output = String::new();

        // Handle the root node without any prefix/indentation
        let mut line = self.get_display_name(node, !args.no_color);

        // Add file size to root if requested
        if args.size {
            if let Some(size) = node.size {
                line.push_str(&format!(
                    " ({})",
                    helpers::bytes::format(size, &args.size_format)
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
                args,
            ));
        }

        // Append summary if requested
        if args.summary {
            output.push('\n');
            output.push_str(&stats.to_string());
        }

        Ok(output)
    }
}

/// Returns the appropriate formatter based on the requested output format
pub fn get_formatter(format: &OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Text => Box::new(TextFormatter),
        // OutputFormat::Json => Box::new(JsonFormatter), // TODO: Implement JSON formatter
        // OutputFormat::Xml => Box::new(XmlFormatter),   // TODO: Implement XML formatter
    }
}

/// Defines the supported output formats for the tree
#[derive(Clone)]
pub enum OutputFormat {
    Text,
    // Json, // TODO: JSON output
    // Xml,  // TODO: XML output
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            // "json" => Ok(Self::Json),
            // "xml" => Ok(Self::Xml),
            e => Err(format!("Unknown output format: {}", e)),
        }
    }
}
