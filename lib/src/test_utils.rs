use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

/// A temporary directory that is removed when dropped.
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    pub(crate) fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "termusic-test-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        fs::create_dir(&dir).unwrap();
        Self(dir)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Connections must be dropped first, including on Windows.
        let result = fs::remove_dir_all(&self.0);
        if !thread::panicking() {
            result.unwrap();
        }
    }
}
