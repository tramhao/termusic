use std::io::{self, Read, Seek, SeekFrom};

const OK: u16 = 200;
const PARTIAL_CONTENT: u16 = 206;

pub struct HttpStreamReader {
    url: String,
    // username: String,
    // password: Option<String>,
    agent: ureq::Agent,
    pub start: u64,
    pub end: u64,
}

impl HttpStreamReader {
    pub fn new(
        url: String,
        // username: String,
        // password: Option<String>,
        // agent: ureq::Agent,
    ) -> Self {
        let agent = ureq::AgentBuilder::new().build();
        let res = agent
            .head(&url[..])
            // .set(
            //     "Authorization",
            //     &basic_auth::encode(&username[..], password.clone())[..],
            // )
            .call()
            .unwrap();
        let len = res
            .header("Content-Length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap();
        eprintln!("len of stream is: {len}");
        eprintln!("url is {url}");

        HttpStreamReader {
            url,
            // username,
            // password,
            agent,
            start: 0,
            end: len as u64 - 1,
        }
    }
}

impl Read for HttpStreamReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.start > self.end {
            Ok(0)
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(buf.len() as u64, self.end - self.start + 1);
            let res = self
                .agent
                .get(&self.url)
                // .set(
                //     "Authorization",
                //     &basic_auth::encode(&self.username[..], self.password.clone())[..],
                // )
                .set("Range", &format!("bytes={}-{}", prev_start, self.start - 1))
                .call()
                .unwrap();
            let status = res.status();
            if status == OK || status == PARTIAL_CONTENT {
                res.into_reader().read(buf)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unexpected server response: {}", status),
                ))
            }
        }
    }
}

impl Seek for HttpStreamReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => {
                self.start = offset;
                Ok(self.start)
            }
            SeekFrom::End(offset) => {
                if offset.is_negative() {
                    let offset_abs = offset.abs() as u64;
                    if self.end >= offset_abs {
                        self.start = self.end - offset_abs;
                        Ok(self.start)
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "It's an error to seek before byte 0",
                        ))
                    }
                } else {
                    self.start = self.end + offset as u64;
                    Ok(self.start)
                }
            }
            SeekFrom::Current(offset) => {
                if offset.is_negative() {
                    let offset_abs = offset.abs() as u64;
                    if self.start >= offset_abs {
                        self.start = self.start - offset_abs;
                        Ok(self.start)
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "It's an error to seek before byte 0",
                        ))
                    }
                } else {
                    self.start = self.start + offset as u64;
                    Ok(self.start)
                }
            }
        }
    }
}
