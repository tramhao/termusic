// Read and write vorbiscomment metadata

//use lewton::header::CommentHeader;
use anyhow::Result;
use ogg::writing::PacketWriteEndInfo;
use ogg::{Packet, PacketReader, PacketWriter};
use std::convert::TryInto;
use std::io::{Cursor, Read, Seek};

pub type CommentHeader = lewton::header::CommentHeader;

//type VorbisComments = CommentHeader;
#[allow(clippy::new_ret_no_self)]
pub trait VorbisComments {
    fn from(vendor: String, comment_list: Vec<(String, String)>) -> CommentHeader;
    fn new() -> CommentHeader;
    fn get_tag_names(&self) -> Vec<String>;
    fn get_tag_single(&self, tag: &str) -> Option<String>;
    fn get_tag_multi(&self, tag: &str) -> Vec<String>;
    fn clear_tag(&mut self, tag: &str);
    fn add_tag_single(&mut self, tag: &str, value: &str);
    fn add_tag_multi(&mut self, tag: &str, values: &[&str]);
    fn get_vendor(&self) -> String;
    fn set_vendor(&mut self, vend: &str);
}

impl VorbisComments for CommentHeader {
    fn from(vendor: String, comment_list: Vec<(String, String)>) -> CommentHeader {
        Self { vendor, comment_list }
    }

    fn new() -> CommentHeader {
        Self {
            vendor: "".to_string(),
            comment_list: Vec::new(),
        }
    }

    fn get_tag_names(&self) -> Vec<String> {
        let mut names = self
            .comment_list
            .iter()
            .map(|comment| comment.0.to_lowercase())
            .collect::<Vec<String>>();
        names.sort_unstable();
        names.dedup();
        names
    }

    fn get_tag_single(&self, tag: &str) -> Option<String> {
        let tags = self.get_tag_multi(tag);
        if tags.is_empty() {
            None
        } else {
            Some(tags[0].to_string())
        }
    }

    fn get_tag_multi(&self, tag: &str) -> Vec<String> {
        self.comment_list
            .clone()
            .iter()
            .filter(|comment| comment.0.to_lowercase() == tag.to_string().to_lowercase())
            .map(|comment| comment.1.clone())
            .collect::<Vec<String>>()
    }

    fn clear_tag(&mut self, tag: &str) {
        self.comment_list
            .retain(|comment| comment.0.to_lowercase() != tag.to_string().to_lowercase());
    }

    fn add_tag_single(&mut self, tag: &str, value: &str) {
        self.comment_list
            .push((tag.to_string().to_lowercase(), value.to_string()));
    }

    fn add_tag_multi(&mut self, tag: &str, values: &[&str]) {
        for value in values.iter() {
            self.comment_list
                .push((tag.to_string().to_lowercase(), (*value).to_string()));
        }
    }

    fn get_vendor(&self) -> String {
        self.vendor.to_string()
    }

    fn set_vendor(&mut self, vend: &str) {
        self.vendor = vend.to_string();
    }
}

#[allow(unused)]
pub fn make_comment_header(header: &CommentHeader) -> Vec<u8> {
    //Signature
    let start = [3_u8, 118, 111, 114, 98, 105, 115];

    //Vendor number of bytes as u32
    let vendor = header.vendor.as_bytes();
    let vendor_len: u32 = vendor.len().try_into().unwrap_or_default();

    //end byte
    let end: u8 = 1;

    let mut new_packet: Vec<u8> = vec![];

    //write start
    new_packet.extend(start.iter().copied());

    //write vendor
    new_packet.extend(vendor_len.to_le_bytes().iter().copied());
    new_packet.extend(vendor.iter().copied());

    //write number of comments
    let comment_nbr: u32 = header.comment_list.len().try_into().unwrap_or_default();
    new_packet.extend(comment_nbr.to_le_bytes().iter().copied());

    let mut commentstrings: Vec<String> = vec![];
    //write each comment
    for comment in &header.comment_list {
        commentstrings.push(format!("{}={}", comment.0, comment.1));
        //let commenstrings.last().as_bytes();
        let comment_len: u32 = commentstrings
            .last()
            .unwrap_or(&String::from(""))
            .as_bytes()
            .len()
            .try_into()
            .unwrap_or_default();
        new_packet.extend(comment_len.to_le_bytes().iter().copied());
        new_packet.extend(
            commentstrings
                .last()
                .unwrap_or(&String::from(""))
                .as_bytes()
                .iter()
                .copied(),
        );
    }
    new_packet.push(end);
    //println!("{:?}",new_packet);
    new_packet
}

#[allow(unused)]
pub fn read_comment_header<T: Read + Seek>(f_in: T) -> Result<CommentHeader> {
    let mut reader = PacketReader::new(f_in);

    let packet: Packet = reader.read_packet_expected()?;
    let stream_serial = packet.stream_serial();

    let mut packet: Packet = reader.read_packet_expected()?;
    //println!("{:?}",packet.data);
    while packet.stream_serial() != stream_serial {
        packet = reader.read_packet_expected()?;
        //println!("{:?}",packet.data);
    }
    Ok(lewton::header::read_header_comment(&packet.data)?)
    //println!("{:?}", comment_hdr);
    // comment_hdr
}

#[allow(unused)]
pub fn replace_comment_header<T: Read + Seek>(f_in: T, new_header: &CommentHeader) -> Cursor<Vec<u8>> {
    let new_comment_data = make_comment_header(new_header);

    let f_out_ram: Vec<u8> = vec![];
    let mut f_out = Cursor::new(f_out_ram);

    let mut reader = PacketReader::new(f_in);
    let mut writer = PacketWriter::new(&mut f_out);

    let mut header_done = false;
    loop {
        let rp = reader.read_packet();
        match rp {
            Ok(r) => {
                match r {
                    Some(mut packet) => {
                        let inf = if packet.last_in_stream() {
                            PacketWriteEndInfo::EndStream
                        } else if packet.last_in_page() {
                            PacketWriteEndInfo::EndPage
                        } else {
                            PacketWriteEndInfo::NormalPacket
                        };
                        if !header_done {
                            let comment_hdr = lewton::header::read_header_comment(&packet.data);
                            match comment_hdr {
                                Ok(_hdr) => {
                                    // This is the packet to replace
                                    packet.data = new_comment_data.clone();
                                    header_done = true;
                                },
                                Err(_error) => {},
                            }
                        }
                        let lastpacket = packet.last_in_stream() && packet.last_in_page();
                        let stream_serial = packet.stream_serial();
                        let absgp_page = packet.absgp_page();
                        let _drop = writer.write_packet(packet.data.into_boxed_slice(), stream_serial, inf, absgp_page);
                        if lastpacket {
                            break;
                        }
                    },
                    // End of stream
                    None => break,
                }
            },
            Err(error) => {
                println!("Error reading packet: {:?}", error);
                break;
            },
        }
    }
    let _drop = f_out.seek(std::io::SeekFrom::Start(0));
    f_out
}
