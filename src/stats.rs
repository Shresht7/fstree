use serde::Serialize;

/// Statistics collected during tree traversal
#[derive(Default, Serialize)]
pub struct Statistics {
    /// The total count of directories
    dirs: usize,
    /// The total count of files
    files: usize,
    /// The total byte count
    bytes: u64,
}

impl Statistics {
    pub fn add_dirs(&mut self, n: usize) {
        self.dirs += n
    }

    pub fn add_files(&mut self, n: usize) {
        self.files += n
    }

    pub fn add_byte_size(&mut self, n: u64) {
        self.bytes += n;
    }
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} directories, {} files ({} bytes)",
            self.dirs, self.files, self.bytes
        )
    }
}
