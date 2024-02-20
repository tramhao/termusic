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
    /// Create a new Buffered-Source with a given custom size
    pub fn new(file: File, size: usize) -> Self {
        let mut is_seekable = false;
        let mut byte_len = None;

        let mut buf_size = size;

        if let Ok(metadata) = file.metadata() {
            // If the file's metadata is available, and the file is a regular file (i.e., not a FIFO,
            // etc.), then the MediaSource will be seekable. Otherwise assume it is not. Note that
            // metadata() follows symlinks.
            is_seekable = metadata.is_file();
            let byte_len_local = metadata.len();
            byte_len = Some(byte_len_local);

            let byte_len_local = usize::try_from(byte_len_local).unwrap_or(usize::MAX);

            // only allocate file_size if lower than requested buffer, as the other buffer space would be wasted memory
            if byte_len_local < buf_size {
                buf_size = byte_len_local;
            }
        }

        let inner = BufReader::with_capacity(buf_size, file);

        trace!("Buffer capacity {}", inner.capacity());

        BufferedSource {
            inner,
            is_seekable,
            byte_len,
        }
    }

    /// Create a new Buffered-Source with default [`BUF_SIZE`]
    pub fn new_default_size(file: File) -> Self {
        Self::new(file, BUF_SIZE)
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
            // the following uses i64::try_from, because of a clippy lint and falls back to a normal seek instead of multiple (and more complex) seeking
            if v >= old_pos {
                let Ok(offset) = i64::try_from(v - old_pos) else {
                    // fallback, return normal seek because otherwise it would mean to doing 2 seeks, which would read unnecessary data, which may be discarded right away
                    return self.inner.seek(pos);
                };
                // seek forward
                self.inner.seek_relative(offset)?;
            } else {
                let Ok(offset) = i64::try_from(old_pos - v) else {
                    // fallback, return normal seek because otherwise it would mean to doing 2 seeks, which would read unnecessary data, which may be discarded right away
                    return self.inner.seek(pos);
                };
                // seek backward
                self.inner.seek_relative(-offset)?;
            }

            return self.inner.stream_position();
        }

        // fallback
        self.inner.seek(pos)
    }
}
