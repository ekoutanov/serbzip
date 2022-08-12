//! Common utilities used by integration tests.

use std::fs;
use std::path::{Path, PathBuf};
use rand::RngCore;

/// A path to a temporary file that will be deleted (if the file was created) when the path
/// is eventually dropped.
#[derive(Debug)]
pub struct TempPath {
    path_buf: PathBuf
}

impl TempPath {
    /// Returns a random path in the system's temp directory.
    pub fn with_extension(extension: &str) -> Self {
        let path_buf = std::env::temp_dir();
        let random = rand::thread_rng().next_u64();
        let path_buf = path_buf.join(format!("test-{random:X}.{extension}"));
        Self { path_buf }
    }
}

impl AsRef<Path> for TempPath {
    fn as_ref(&self) -> &Path {
        self.path_buf.as_path()
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(self);
    }
}