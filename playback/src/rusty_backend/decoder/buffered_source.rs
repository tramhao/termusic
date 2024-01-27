use std::fs::File;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};

use symphonia::core::io::MediaSource;

/// Buffer capacity in bytes
///
/// 1024 * 1024 * 4 = 4 MiB
///
/// [`BufReader`] default size is 8 Kib
const BUF_SIZE: usize = 1024 * 1024 * 4;

/// A [`MediaSource`] for [`File`] but using a [`BufReader`] (Buffered file source)
pub struct BufferedSource {
    /// The inner reader
    inner: BufReader<File>,
    /// Cache the [`MediaSource::is_seekable`] call
    is_seekable: bool,
    /// Cache the [`MediaSource::byte_len`] call
    byte_len: Option<u64>,
}

impl BufferedSource {
    pub fn new(file: File) -> Self {
        let mut is_seekable = false;
        let mut byte_len = None;

        if let Ok(metadata) = file.metadata() {
            // If the file's metadata is available, and the file is a regular file (i.e., not a FIFO,
            // etc.), then the MediaSource will be seekable. Otherwise assume it is not. Note that
            // metadata() follows symlinks.
            is_seekable = metadata.is_file();
            byte_len = Some(metadata.len());
        }

        let inner = BufReader::with_capacity(BUF_SIZE, file);

        BufferedSource {
            inner,
            is_seekable,
            byte_len,
        }
    }
}

impl MediaSource for BufferedSource {
    fn is_seekable(&self) -> bool {
        self.is_seekable
    }

    fn byte_len(&self) -> Option<u64> {
        self.byte_len
    }
}

impl Read for BufferedSource {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for BufferedSource {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        // normal seek always flushes the buffer
        // use relative seek for common cases
        if let SeekFrom::Current(v) = pos {
            self.inner.seek_relative(v)?;
            return self.inner.stream_position();
        }
        if let SeekFrom::Start(v) = pos {
            let old_pos = self.inner.stream_position()?;
            if v >= old_pos {
                // seek forward
                self.inner.seek_relative((v - old_pos) as i64)?;
            } else {
                // seek backward
                self.inner.seek_relative(-((old_pos - v) as i64))?;
            }

            return self.inner.stream_position();
        }

        // fallback
        self.inner.seek(pos)
    }
}
