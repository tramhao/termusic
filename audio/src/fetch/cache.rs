use anyhow::Error;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};
use termusiclib::utils::get_app_config_path;
pub struct Cache {
    cache_dir: String,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    pub fn new() -> Self {
        let mut cache_dir = std::env::temp_dir().to_string_lossy().to_string();
        if let Ok(dir) = get_app_config_path() {
            cache_dir = format!("{}/cache", dir.to_string_lossy());
        }
        Self { cache_dir }
    }

    pub fn save_file<F: Read>(&self, name: &str, contents: &mut F) -> Result<(), Error> {
        if self.is_file_cached(name) {
            return Ok(());
        }
        let mut file = File::create(format!("{}/{}", self.cache_dir, name))?;
        io::copy(contents, &mut file)?;
        Ok(())
    }

    pub fn is_file_cached(&self, name: &str) -> bool {
        Path::new(&format!("{}/{}", self.cache_dir, name)).exists()
    }

    pub fn open_file(&self, name: &str) -> Result<File, Error> {
        Ok(File::open(format!("{}/{}", self.cache_dir, name))?)
    }
}
