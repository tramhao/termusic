// Package lyric package download lyrics from different website and embed them into mp3 file.
// lrc file is used to parse lrc file into subtitle. Similar to subtitles package
// [al:''Album where the song is from'']
// [ar:''Lyrics artist'']
// [by:''Creator of the LRC file'']
// [offset:''+/- Overall timestamp adjustment in milliseconds, + shifts time up, - shifts down'']
// [re:''The player or editor that creates LRC file'']
// [ti:''Lyrics (song) title'']
// [ve:''version of program'']
// [ti:Let's Twist Again]
// [ar:Chubby Checker oppure  Beatles, The]
// [au:Written by Kal Mann / Dave Appell, 1961]
// [al:Hits Of The 60's - Vol. 2 – Oldies]
// [00:12.00]Lyrics beginning ...
// [00:15.30]Some more lyrics ...

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::Ordering;
use std::str::FromStr;
use std::time::Duration;

lazy_static! {
    static ref LYRICS_RE: Regex = Regex::new("^[^\x00-\x08\x0A-\x1F\x7F]*$").unwrap();
    static ref TAG_RE: Regex = Regex::new(r"\[.*:.*\]").unwrap();
    static ref LINE_STARTS_WITH_RE: Regex =
        Regex::new("^\\[([^\x00-\x08\x0A-\x1F\x7F\\[\\]:]*):([^\x00-\x08\x0A-\x1F\x7F\\[\\]]*)\\]")
            .unwrap();
}

#[derive(Clone)]
pub struct Lyric {
    pub offset: i64, // positive means delay lyric
    pub lang_extension: Option<String>,
    pub unsynced_captions: Vec<UnsyncedCaption>, // USLT captions
                                                 // SyncedCaptions      []id3v2.SyncedText // SYLT captions
}

#[derive(Clone)]
pub struct UnsyncedCaption {
    time_stamp: u64,
    text: String,
}

const EOL: &str = "\n";

impl Lyric {
    // GetText will fetch lyric by time in seconds
    pub fn get_text(&self, time: i64) -> Option<String> {
        if self.unsynced_captions.is_empty() {
            return None;
        };

        // here we want to show lyric 1 second earlier
        let mut time = time * 1000 + 1000;
        time += self.offset;

        let mut text = self.unsynced_captions.get(0)?.text.clone();
        for v in self.unsynced_captions.iter() {
            if time >= v.time_stamp as i64 {
                text = v.text.clone();
            } else {
                break;
            }
        }
        Some(text)
    }

    pub fn get_index(&self, time: i64) -> Option<usize> {
        if self.unsynced_captions.is_empty() {
            return None;
        };

        // here we want to show lyric 1 second earlier
        let mut time = time * 1000 + 1000;
        time += self.offset;

        let mut index: usize = 0;
        for (i, v) in self.unsynced_captions.iter().enumerate() {
            if time >= v.time_stamp as i64 {
                index = i;
            } else {
                break;
            }
        }
        Some(index)
    }

    pub fn adjust_offset(&mut self, time: i64, offset: i64) {
        if let Some(index) = self.get_index(time) {
            if index == 0 {
                self.offset -= offset;
            } else {
                let mut v = &mut self.unsynced_captions[index];
                let adjusted_time_stamp = v.time_stamp as i64 + offset;
                v.time_stamp = match adjusted_time_stamp.cmp(&0) {
                    Ordering::Greater | Ordering::Equal => adjusted_time_stamp as u64,
                    Ordering::Less => 0,
                };
            }
        };
        // we sort the captions by time_stamp. This is to fix some lyrics downloaded are not sorted
        self.unsynced_captions
            .sort_by(|b, a| b.time_stamp.cmp(&a.time_stamp));
    }

    pub fn as_lrc(&mut self) -> Option<String> {
        let mut result: String = String::new();
        if self.offset != 0 {
            let string_offset = format!("[offset:{}]\n", self.offset);
            result += string_offset.as_ref();
        }

        for line in self.unsynced_captions.iter() {
            result += line.as_lrc().as_str();
        }
        Some(result)
    }

    pub fn merge_adjacent(&mut self) {
        let unsynced_captions = self.unsynced_captions.clone();
        let mut unsynced_captions2 = unsynced_captions.clone();
        let mut offset = 1;
        for (i, v) in unsynced_captions.iter().enumerate() {
            if i < 1 {
                continue;
            }
            if let Some(item) = unsynced_captions2.get(i - offset) {
                if v.time_stamp - item.time_stamp < 2000 {
                    unsynced_captions2[i - offset].text += v.text.as_ref();
                    unsynced_captions2.remove(i - offset + 1);
                    offset += 1;
                }
            }
        }

        self.unsynced_captions = unsynced_captions2;
    }
}

impl UnsyncedCaption {
    fn parse_line(line: &mut String) -> Result<Self, ()> {
        //[00:12.00]Line 1 lyrics
        // !line.starts_with('[') | !line.contains(']')
        // First, parse the time
        let time_stamp = UnsyncedCaption::parse_time(
            line.get(line.find('[').unwrap() + 1..line.find(']').unwrap())
                .unwrap(),
        )?;
        let text = line
            .drain(line.find(']').unwrap() + 1..)
            .collect::<String>();
        Ok(Self { time_stamp, text })
    }

    fn parse_time(string: &str) -> Result<u64, ()> {
        //mm:ss.xx or mm:ss.xxx
        if !(string.contains(':')) | !(string.contains('.')) {
            return Err(());
        }
        let (x, y) = (string.find(':').unwrap(), string.find('.').unwrap());
        let minute = string.get(0..x).ok_or(())?.parse::<u32>().map_err(|_| ())?;
        let second = string
            .get(x + 1..y)
            .ok_or(())?
            .parse::<u32>()
            .map_err(|_| ())?;
        let micros = &format!("0.{}", string.get(y + 1..).ok_or(())?)
            .parse::<f64>()
            .map_err(|_| ())?;
        let sum_milis = minute as u64 * 60 * 1000 + second as u64 * 1000 + (micros * 1000.0) as u64;
        Ok(sum_milis)
    }

    fn as_lrc(&self) -> String {
        let line = format!("[{}]{}", time_lrc(self.time_stamp), self.text);
        line + EOL
    }
}

fn time_lrc(time_stamp: u64) -> String {
    let time_duration = Duration::from_millis(time_stamp);
    let _h = time_duration.as_secs() / 3600;
    let m = (time_duration.as_secs() / 60) % 60;
    let s = time_duration.as_secs() % 60;
    let ms = time_duration.as_millis() % 60;

    let res = format!("{:02}:{:02}.{:02}", m, s, ms);
    res
}

impl FromStr for Lyric {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // s = cleanLRC(s)
        // lines := strings.Split(s, "\n")
        let mut offset: i64 = 0;
        let lang_extension = Some(String::new());
        let mut unsynced_captions = vec![];
        for line in s.split('\n') {
            let mut line = line.to_string();
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            let line = line.trim();
            let mut line = line.to_string();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("[offset") {
                let line = line.trim_start_matches("[offset:");
                let line = line.trim_end_matches(']');
                let line = line.replace(" ", "");
                offset = line.parse().unwrap();
            }

            if !LINE_STARTS_WITH_RE.is_match(line.as_ref()) {
                continue;
            }

            if let Ok(s) = UnsyncedCaption::parse_line(&mut line) {
                unsynced_captions.push(s);
            };
        }

        // we sort the cpations by Timestamp. This is to fix some lyrics downloaded are not sorted
        unsynced_captions.sort_by(|b, a| b.time_stamp.cmp(&a.time_stamp));

        let mut lyric = Lyric {
            offset,
            lang_extension,
            unsynced_captions,
        };

        lyric.merge_adjacent();

        Ok(lyric)
    }
}
