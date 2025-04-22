use std::io;

use crate::cli::Args;
use crate::helpers;
use crate::tree::TreeNode;

pub trait Formatter {
    fn format(
        &self,
        node: &TreeNode,
        args: &Args,
        stats: &crate::stats::Statistics,
    ) -> io::Result<String>;
}

pub struct TextFormatter;

impl TextFormatter {
    fn format_node(&self, node: &TreeNode, prefix: &str, is_last: bool, args: &Args) -> String {
        let mut output = String::new();

        let branch = if is_last {
            &args.last_prefix
        } else {
            &args.prefix
        };
        let display_name = if node.is_dir {
            format!("{}/", node.name)
        } else {
            node.name.clone()
        };

        let mut line = format!("{}{}{}", prefix, branch, display_name);

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

        let child_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        for (i, child) in node.children.iter().enumerate() {
            output.push_str(&self.format_node(
                child,
                &child_prefix,
                i == node.children.len() - 1,
                args,
            ));
        }

        output
    }
}

impl Formatter for TextFormatter {
    fn format(
        &self,
        node: &TreeNode,
        args: &Args,
        stats: &crate::stats::Statistics,
    ) -> io::Result<String> {
        let mut output = self.format_node(node, "", true, args);

        if args.summary {
            output.push('\n');
            output.push_str(&stats.to_string());
        }

        Ok(output)
    }
}

pub fn get_formatter(format: &OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Text => Box::new(TextFormatter),
        // OutputFormat::Json => Box::new(JsonFormatter),
        // OutputFormat::Xml => Box::new(XmlFormatter),
    }
}

#[derive(Clone)]
pub enum OutputFormat {
    Text,
    // Json,
    // Xml,
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
