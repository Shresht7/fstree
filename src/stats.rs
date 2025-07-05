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
    /// Adds a specified number of directories to the total count.
    pub fn add_dirs(&mut self, n: usize) {
        self.dirs += n
    }

    /// Adds a specified number of files to the total count.
    pub fn add_files(&mut self, n: usize) {
        self.files += n
    }

    /// Adds a specified number of bytes to the total size.
    pub fn add_byte_size(&mut self, n: u64) {
        self.bytes += n;
    }
}

// Implement the display trait for Statistics. This is what is show as the summary report
impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} directories, {} files ({} bytes)",
            self.dirs, self.files, self.bytes
        )
    }
}
