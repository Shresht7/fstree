/// Statistics collected during tree traversal
#[derive(Default)]
pub struct Statistics {
    /// The total count of directories
    dirs: usize,
    /// The total count of files
    files: usize,
}

impl Statistics {
    pub fn add_dirs(&mut self, n: usize) {
        self.dirs += n
    }

    pub fn add_files(&mut self, n: usize) {
        self.files += n
    }
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} directories, {} files", self.dirs, self.files)
    }
}
