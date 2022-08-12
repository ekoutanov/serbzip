//! Common utilities used by integration tests.

use std::path::PathBuf;
use rand::RngCore;

/// Generates a random file in the system's temp directory.
pub fn generate_random_path(extension: &str) -> PathBuf {
    let path_buf = std::env::temp_dir();
    let random = rand::thread_rng().next_u64();
    path_buf.join(format!("test-{random:X}.{extension}"))
}