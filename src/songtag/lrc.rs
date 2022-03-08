/**
 * MIT License
 *
 * termusic - Copyright (c) 2021 Larry Hao
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// Package downloaded lyrics from different websites and embed them into an MP3 file.
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
}

#[derive(Clone)]
pub struct UnsyncedCaption {
    time_stamp: i64,
    text: String,
}

const EOL: &str = "\n";

impl Lyric {
    // GetText will fetch lyric by time in seconds
    pub fn get_text(&self, mut time: i64) -> Option<String> {
        if self.unsynced_captions.is_empty() {
            return None;
        };

        // here we want to show lyric 2 second earlier
        let mut adjusted_time = time * 1000 + 2000;
        adjusted_time += self.offset;
        if adjusted_time < 0 {
            adjusted_time = 0;
        }

        time = adjusted_time;

        let mut text = self.unsynced_captions.get(0)?.text.clone();
        for v in &self.unsynced_captions {
            if time >= v.time_stamp {
                text = v.text.clone();
            } else {
                break;
            }
        }
        Some(text)
    }

    pub fn get_index(&self, mut time: i64) -> Option<usize> {
        if self.unsynced_captions.is_empty() {
            return None;
        };

        // here we want to show lyric 2 second earlier
        let mut adjusted_time = time * 1000 + 2000;
        adjusted_time += self.offset;
        if adjusted_time < 0 {
            adjusted_time = 0;
        }

        time = adjusted_time.abs();

        let mut index: usize = 0;
        for (i, v) in self.unsynced_captions.iter().enumerate() {
            if time >= v.time_stamp {
                index = i;
            } else {
                break;
            }
        }
        Some(index)
    }

    pub fn adjust_offset(&mut self, time: i64, offset: i64) {
        if let Some(index) = self.get_index(time) {
            // when time stamp is less than 10 seconds or index is before the first line, we adjust
            // the offset.
            if (index == 0) | (time < 11) {
                self.offset -= offset;
            } else {
                // fine tuning each line after 10 seconds
                let mut v = &mut self.unsynced_captions[index];
                let adjusted_time_stamp = v.time_stamp + offset;
                v.time_stamp = match adjusted_time_stamp.cmp(&0) {
                    Ordering::Greater | Ordering::Equal => adjusted_time_stamp,
                    Ordering::Less => 0,
                };
            }
        };
        // we sort the captions by time_stamp. This is to fix some lyrics downloaded are not sorted
        self.unsynced_captions
            .sort_by(|b, a| b.time_stamp.cmp(&a.time_stamp));
    }

    pub fn as_lrc_text(&self) -> String {
        let mut result: String = String::new();
        if self.offset != 0 {
            let string_offset = format!("[offset:{}]\n", self.offset);
            result += string_offset.as_ref();
        }

        for line in &self.unsynced_captions {
            result += line.as_lrc().as_str();
        }
        result
    }

    pub fn merge_adjacent(&mut self) {
        let mut unsynced_captions = self.unsynced_captions.clone();
        let mut offset = 1;
        for (i, v) in self.unsynced_captions.iter().enumerate() {
            if i < 1 {
                continue;
            }
            if let Some(item) = unsynced_captions.get(i - offset) {
                if v.time_stamp - item.time_stamp < 2000 {
                    unsynced_captions[i - offset].text += "  ";
                    unsynced_captions[i - offset].text += v.text.as_ref();
                    unsynced_captions.remove(i - offset + 1);
                    offset += 1;
                }
            }
        }

        self.unsynced_captions = unsynced_captions;
    }
}

impl UnsyncedCaption {
    fn parse_line(line: &mut String) -> Result<Self, ()> {
        //[00:12.00]Line 1 lyrics
        // !line.starts_with('[') | !line.contains(']')
        // First, parse the time
        let time_stamp = Self::parse_time(
            line.get(line.find('[').ok_or(())? + 1..line.find(']').ok_or(())?)
                .ok_or(())?,
        )?;
        let text = line
            .drain(line.find(']').ok_or(())? + 1..)
            .collect::<String>();
        Ok(Self {
            time_stamp: time_stamp.try_into().unwrap_or(0),
            text,
        })
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn parse_time(string: &str) -> Result<u64, ()> {
        //mm:ss.xx or mm:ss.xxx
        if !(string.contains(':')) | !(string.contains('.')) {
            return Err(());
        }
        let (x, y) = (string.find(':').ok_or(())?, string.find('.').ok_or(())?);
        let minute = string.get(0..x).ok_or(())?.parse::<u32>().map_err(|_| ())?;
        let second = string
            .get(x + 1..y)
            .ok_or(())?
            .parse::<u32>()
            .map_err(|_| ())?;
        let micros = &format!("0.{}", string.get(y + 1..).ok_or(())?)
            .parse::<f64>()
            .map_err(|_| ())?;
        // let secs_u64 = u64::try_from(micros * 1000.0).ok();
        // let secs = u64::from((micros * 1000.0).round());
        let sum_milis = u64::from(minute) * 60 * 1000
            + u64::from(second) * 1000
            + (micros * 1000.0).abs() as u64;
        Ok(sum_milis)
    }

    fn as_lrc(&self) -> String {
        let line = format!(
            "[{}]{}",
            time_lrc(self.time_stamp.try_into().unwrap_or(0)),
            self.text
        );
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
    // type Err = std::string::ParseError;
    type Err = anyhow::Error;

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
                let line = line.replace(' ', "");
                if let Ok(o) = line.parse() {
                    offset = o;
                }
            }

            if !LINE_STARTS_WITH_RE.is_match(line.as_ref()) {
                continue;
            }

            if let Ok(s) = UnsyncedCaption::parse_line(&mut line) {
                unsynced_captions.push(s);
            };
        }

        // we sort the captions by Timestamp. This is to fix some lyrics downloaded are not sorted
        unsynced_captions.sort_by(|b, a| b.time_stamp.cmp(&a.time_stamp));

        let mut lyric = Self {
            offset,
            lang_extension,
            unsynced_captions,
        };

        lyric.merge_adjacent();

        Ok(lyric)
    }
}
