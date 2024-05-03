//! Some radio open streams to test:
//! - <https://live.musopen.org:8085/streamvbr0>, at the time of writing "icy-metaint" is "1600" and actually sets titles
//! - <http://war.str3am.com:7780/WUISRIS-2>, at the time of writing "icy-metaint" is "1600" and does not set titles (empty titles or 0 metadata)
//!
//! extra icy resources
//! - <https://cast.readme.io/docs/icy#metadata>
//! - <https://gist.github.com/niko/2a1d7b2d109ebe7f7ca2f860c3505ef0>

use std::{io::Read, num::NonZeroU16};

pub(super) struct FilterOutIcyMetadata<T: Read, F: Fn(&str)> {
    /// The inner stream
    inner: T,
    /// The "icy-metaint" header's value
    icy_metaint: NonZeroU16,
    /// The callback to set the title
    cb: F,
    /// Remaining bytes until another metadata chunk
    remaing_bytes: usize,
    // /// Total bytes read from the inner stream, useful for debugging, otherwise unused
    // total_read: usize,
}

impl<T: Read, F: Fn(&str)> FilterOutIcyMetadata<T, F> {
    pub fn new(inner: T, cb: F, icy_metaint: NonZeroU16) -> Self {
        Self {
            inner,
            cb,
            icy_metaint,
            remaing_bytes: icy_metaint.get() as usize,
            // total_read: 0,
        }
    }
}

impl<T: Read, F: Fn(&str)> Read for FilterOutIcyMetadata<T, F> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // no metadata to handle yet
        if self.remaing_bytes > 0 {
            // get how much bytes to read at most
            let to_read_bytes = self.remaing_bytes.min(buf.len());
            // read the bytes with a truncated buffer
            let read_bytes = self.inner.read(&mut buf[..to_read_bytes])?;
            self.remaing_bytes -= read_bytes;

            // self.total_read += read_bytes;
            // trace!("now position is {:08x}", self.total_read);

            return Ok(read_bytes);
        }
        // beyond here we handle metdata as the next byte

        // buffer for the icy metadata length byte
        let mut length = [0; 1];
        self.inner.read_exact(&mut length)?;
        // self.total_read += 1;
        // trace!("at position {:08x}", self.total_read);
        let length = (length[0] as usize) * 16;
        trace!("ICY METADATA LENGTH {}", length);

        // dont try to do any metadata parsing if there is none
        if length != 0 {
            // buffer for the icy metadata
            let mut metadata = Vec::with_capacity(length);

            // pulling in 16 bytes as that is the stepping in icy-metadata (and more efficient than 1 at a time)
            // can this be improved further?
            let mut buffer = [0; 16];
            while metadata.len() < length {
                self.inner.read_exact(&mut buffer)?;

                metadata.extend_from_slice(&buffer);
            }

            // self.total_read += length;
            // trace!("after metadata at position {:08x}", self.total_read);

            // debug print the parsed buffer as a string
            trace!("buffer {:#?}", String::from_utf8_lossy(&metadata));

            if let Some(title) = find_title_metadata(&metadata) {
                debug!("Found a new Radio Title: {:#?}", title);
                (self.cb)(title);
            }
        }

        // only do the casting once
        let icy_metaint = self.icy_metaint.get() as usize;

        let to_read_bytes = icy_metaint.min(buf.len());
        let read_bytes = self.inner.read(&mut buf[..to_read_bytes])?;
        // only set "remaining_bytes" once instead of before and after the above "read"
        self.remaing_bytes = icy_metaint - read_bytes;
        // self.total_read += read_bytes;

        Ok(read_bytes)
    }
}

/// Parse icy radio metadata from bytes and return a reference to it
fn find_title_metadata(metadata: &[u8]) -> Option<&str> {
    let metadata_string = std::str::from_utf8(metadata).unwrap_or("");
    if !metadata_string.is_empty() {
        // some reference https://cast.readme.io/docs/icy#metadata
        const STREAM_TITLE_KEYWORD: &str = "StreamTitle='";
        const STREAM_TITLE_END_KEYWORD: &str = "\';";
        if let Some(index) = metadata_string.find(STREAM_TITLE_KEYWORD) {
            let left_index = index + 13;
            let stream_title_substring = &metadata_string[left_index..];
            if let Some(right_index) = stream_title_substring.find(STREAM_TITLE_END_KEYWORD) {
                return Some(&stream_title_substring[..right_index]);
            }
        }
    }

    None
}

#[cfg(test)]
mod test {
    mod find_title_metadata {
        use super::super::*;

        #[test]
        fn find_title_metadata_should_find_metadata() {
            // basic title
            let bytes = b"StreamTitle='Artist - Title';\0\0\0\0\0\0\0";

            assert_eq!(Some("Artist - Title"), find_title_metadata(bytes));

            // title with end string character
            let bytes = b"StreamTitle='Artist - Don't we need a title?';\0\0\0\0\0\0\0";

            assert_eq!(
                Some("Artist - Don't we need a title?"),
                find_title_metadata(bytes)
            );

            // basic title with no padding
            let bytes = b"StreamTitle='Artist - Title';";

            assert_eq!(Some("Artist - Title"), find_title_metadata(bytes));
        }

        #[test]
        fn find_title_metadata_should_find_empty_string() {
            let bytes = b"StreamTitle='';";

            assert_eq!(Some(""), find_title_metadata(bytes));
        }

        #[test]
        fn find_title_metadata_should_not_find_metadata_with_no_start() {
            // no `STREAM_TITLE_KEYWORD`
            let bytes = b"\0\0\0\0\0\0\0";

            assert_eq!(None, find_title_metadata(bytes));
        }

        #[test]
        fn find_title_metadata_should_not_find_metadata_with_no_end() {
            // no `STREAM_TITLE_END_KEYWORD`
            let bytes = b"StreamTitle='Artist - Title\0\0\0\0\0\0\0";

            assert_eq!(None, find_title_metadata(bytes));
        }
    }

    mod filter_icy_metadata {
        use std::io::Cursor;

        use parking_lot::Mutex;

        use super::super::*;

        #[test]
        fn should_find_metadata() {
            const TESTING_TEXT_1: &[u8] = b"StreamTitle='Testing';";
            const TESTING_TEXT_2: &[u8] = b"StreamTitle='Hello';";

            let interval: u8 = 64;

            // initial interval bytes
            let source: Vec<u8> = (0..interval)
                // then metadata length
                .chain((0..1).map(|_| 3))
                // then metadata itself
                .chain(
                    TESTING_TEXT_1
                        .iter()
                        .copied()
                        .chain((0..(3 * 16 - TESTING_TEXT_1.len())).map(|_| 0)),
                )
                // again interval data
                .chain(0..interval)
                // 0 metadata
                .chain((0..1).map(|_| 0))
                // another interval
                .chain(0..interval)
                // then metadata length
                .chain((0..1).map(|_| 3))
                // then new metadata itself
                .chain(
                    TESTING_TEXT_2
                        .iter()
                        .copied()
                        .chain((0..(3 * 16 - TESTING_TEXT_2.len())).map(|_| 0)),
                )
                // and one last interval
                .chain(0..interval)
                .collect();

            let titles = Mutex::new(Vec::new());
            let cb = |title: &str| {
                titles.lock().push(title.to_string());
            };

            let mut instance = FilterOutIcyMetadata::new(
                Cursor::new(source),
                cb,
                NonZeroU16::new(u16::from(interval)).unwrap(),
            );

            // make sure nothing was called yet
            assert_eq!(0, titles.lock().len());

            let mut buffer = [0; 128];
            // layout how the buffer should look
            let should_buffer = {
                let mut should_buffer = [0u8; 128];
                #[allow(clippy::cast_possible_truncation)]
                for (idx, v) in should_buffer[..(interval as usize)].iter_mut().enumerate() {
                    // we only iterate at most u8::MAX here
                    *v = idx as u8;
                }

                should_buffer
            };

            // test the first initial interval
            {
                // assert that "interval" amount has been read
                assert_eq!(interval as usize, instance.read(&mut buffer).unwrap());
                // assert buffer state
                assert_eq!(&buffer, &should_buffer);

                // make sure nothing was called yet
                assert_eq!(0, titles.lock().len());
            }

            // second interval, read after the title metadata
            {
                // assert that "interval" amount has been read
                assert_eq!(interval as usize, instance.read(&mut buffer).unwrap());

                // assert buffer state
                assert_eq!(&buffer, &should_buffer);

                // assert first title
                let titles_lock = titles.lock();
                assert_eq!(1, titles_lock.len());
                assert_eq!(&"Testing", &titles_lock.first().unwrap().as_str());
            }

            // third interval, after empty metadata
            {
                // assert that "interval" amount has been read
                assert_eq!(interval as usize, instance.read(&mut buffer).unwrap());

                // assert buffer state
                assert_eq!(&buffer, &should_buffer);

                // make sure nothing extra was called
                assert_eq!(1, titles.lock().len());
            }

            // fourth interval, after new title metadata
            {
                // assert that "interval" amount has been read
                assert_eq!(interval as usize, instance.read(&mut buffer).unwrap());

                // assert buffer state
                assert_eq!(&buffer, &should_buffer);

                // assert first title
                let titles_lock = titles.lock();
                assert_eq!(2, titles_lock.len());
                assert_eq!(&"Testing", &titles_lock.first().unwrap().as_str());
                assert_eq!(&"Hello", &titles_lock.get(1).unwrap().as_str());
            }
        }
    }
}
