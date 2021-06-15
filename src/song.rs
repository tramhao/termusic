use anyhow::Result;
use std::fmt;

#[derive(Clone)]
pub struct Song {
    pub file: String,
}

impl Song {
    pub fn load(file: String) -> Result<Self> {
        Ok(Self { file })
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", self.file, self.file,)
    }
}
