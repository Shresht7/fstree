use crate::filter::FileFilter;
use crate::stats::Statistics;
use globset::Glob;

pub struct TreeNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub children: Vec<TreeNode>,
}

pub struct TreeBuilder<'a> {
    pub args: &'a crate::cli::Args,
    file_filter: FileFilter,
    pub stats: Statistics,
}

impl<'a> TreeBuilder<'a> {
    pub fn new(args: &'a crate::cli::Args) -> Result<Self, Box<dyn std::error::Error>> {
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
            args,
            file_filter: FileFilter::new(
                args.root.clone(),
                args.directory,
                args.show_all,
                include_pattern,
                exclude_pattern,
                &args.ignore,
            )?,
            stats: Statistics::default(),
        })
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
        let entries = self.file_filter.filter_entries(path.as_ref())?;
        let mut children = Vec::new();

        for entry in entries {
            if let Ok(child) = self.build(entry.path()) {
                children.push(child);
            }
        }

        Ok(children)
    }

    pub fn get_stats(&self) -> &Statistics {
        &self.stats
    }
}
