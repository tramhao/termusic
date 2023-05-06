use std::io::{self, ErrorKind, Read, SeekFrom};
use std::sync::mpsc;

pub struct ReadableReciever {
    rx: mpsc::Receiver<bytes::Bytes>,
    /// buffer of all received data, used for seeking
    buffer: Vec<u8>,
    offset: usize,
}

impl ReadableReciever {
    pub fn new(rx: mpsc::Receiver<bytes::Bytes>) -> Self {
        Self {
            rx,
            buffer: Vec::new(),
            offset: 0,
        }
    }
}

impl Read for ReadableReciever {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let needed = buf.len();
        let mut unread_buffer = self.buffer.len() - self.offset;
        if needed <= unread_buffer {
            // fill buf from buffer
            buf.clone_from_slice(&self.buffer[self.offset..self.offset + needed]);
            self.offset += needed;
            return Ok(needed);
        }

        // get extra bytes, and put them in the buffer
        // if no bytes are gotten this or the next call to read
        // will return 0 indicating end of file
        let res = self.rx.recv();
        if res.is_err() {
            buf[..unread_buffer]
                .clone_from_slice(&self.buffer[self.offset..self.offset + unread_buffer]);
            self.offset += unread_buffer;
            return Ok(unread_buffer);
        }

        let bytes = res.unwrap();
        self.buffer.extend_from_slice(&bytes);
        unread_buffer += bytes.len();

        let read = if needed <= unread_buffer {
            // got what we needed
            buf.clone_from_slice(&self.buffer[self.offset..self.offset + needed]);
            needed
        } else {
            // less bytes then needed, return what we got do not block
            buf[..unread_buffer]
                .clone_from_slice(&self.buffer[self.offset..self.offset + unread_buffer]);
            unread_buffer
        };
        self.offset += read;
        Ok(read)
    }
}

impl std::io::Seek for ReadableReciever {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        use SeekFrom::*;
        match pos {
            Current(p) if self.offset as i64 + p > self.buffer.len() as i64 => Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "cannot seek after end of reader",
            )),
            Current(p) if self.offset as i64 + p < 0 => Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "cannot seek before start of reader",
            )),
            Start(p) if p > self.buffer.len() as u64 => Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "cannot seek after end of reader",
            )),
            End(p) if self.buffer.len() as i64 + p < 0 => Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "cannot seek before start of reader",
            )),
            End(p) if p > 0 => Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "cannot seek after end of reader",
            )),

            Start(p) => {
                self.offset = p as usize;
                Ok(self.offset as u64)
            }
            Current(p) => {
                self.offset = (self.offset as i64 + p) as usize;
                Ok(self.offset as u64)
            }
            End(p) => {
                self.offset = (self.offset as i64 + p) as usize;
                Ok(self.offset as u64)
            }
        }
    }
}
