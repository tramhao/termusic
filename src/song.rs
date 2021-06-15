use anyhow::Result;

pub struct Song {
    pub file: String,
}

impl Song {
    pub fn load(file: String) -> Result<Self> {
        Ok(Self { file })
    }
}
