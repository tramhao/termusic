use std::io::{BufRead, Error, Read, Seek, SeekFrom};
use symphonia::core::io::MediaSource;

pub struct SeekableBufReader<B> {
    buffer: B,
}

impl<B: Cache> SeekableBufReader<B> {
    pub fn new(buffer: B) -> Self {
        SeekableBufReader { buffer }
    }
}

impl<B: Cache> Read for SeekableBufReader<B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let position = self.buffer.position();
        let cache = self.buffer.slice(position, position + buf.len());
        let amt = cache.len();

        buf[..amt].copy_from_slice(cache);
        self.consume(amt);
        Ok(amt)
    }
}

impl<B: Cache> Seek for SeekableBufReader<B> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Error> {
        self.buffer.seek(pos)
    }
}

impl<B: Cache> BufRead for SeekableBufReader<B> {
    fn fill_buf(&mut self) -> Result<&[u8], Error> {
        Ok(self
            .buffer
            .slice(self.buffer.position(), self.buffer.available()))
    }

    #[allow(unused_must_use)]
    fn consume(&mut self, amt: usize) {
        self.buffer.seek(SeekFrom::Current(amt as i64));
    }
}

pub trait Cache: Read + Seek {
    fn available(&self) -> usize;

    fn position(&self) -> usize;

    fn get(&mut self, index: usize) -> Option<&u8>;

    fn slice(&mut self, from: usize, to: usize) -> &[u8];

    fn cache_to_index(&mut self, index: usize);

    fn cache_to_end(&mut self);
}

impl<R> MediaSource for SeekableBufReader<R>
where
    R: Read + Seek + Send + Sync + Cache,
{
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        // Some(self.buffer.available() as u64)
        None
    }
}
