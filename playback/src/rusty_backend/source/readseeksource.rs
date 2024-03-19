use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};
use std::path::Path;

use symphonia::core::io::MediaSource;

#[derive(Debug)]
pub struct ReadSeekSource<T: Read + Seek + Send> {
    inner: T,
    len: Option<u64>,
    pub extension: Option<String>,
}

pub trait FileExt {
    fn get_file_ext(&self) -> Option<String>;
}

pub trait Source: MediaSource + FileExt + Debug {
    fn as_media_source(self: Box<Self>) -> Box<dyn MediaSource>;
}

impl<T: Read + Seek + Send> ReadSeekSource<T> {
    pub fn new(inner: T, len: Option<u64>, extension: Option<String>) -> Self {
        ReadSeekSource {
            inner,
            len,
            extension,
        }
    }
}

#[allow(dead_code)]
impl ReadSeekSource<BufReader<File>> {
    pub fn from_path(path: &Path) -> Self {
        let file = File::open(path).unwrap();
        let file_len = file.metadata().ok().map(|m| m.len());

        let extension = path.extension().map(|e| e.to_string_lossy().to_string());
        let reader = BufReader::new(file);
        Self::new(reader, file_len, extension)
    }
}

impl<T: Read + Seek + Send + Sync> MediaSource for ReadSeekSource<T> {
    fn is_seekable(&self) -> bool {
        self.len.is_some()
    }

    fn byte_len(&self) -> Option<u64> {
        self.len
    }
}

impl<T: Read + Seek + Send> Read for ReadSeekSource<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Seek + Send> Seek for ReadSeekSource<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

impl<T: Read + Seek + Send> FileExt for ReadSeekSource<T> {
    fn get_file_ext(&self) -> Option<String> {
        self.extension.clone()
    }
}

impl<T: Read + Seek + Send + Sync + Debug + 'static> Source for ReadSeekSource<T> {
    fn as_media_source(self: Box<Self>) -> Box<dyn MediaSource> {
        self
    }
}
