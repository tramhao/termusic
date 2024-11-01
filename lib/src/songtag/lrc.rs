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
use std::cmp::Ordering;
use std::fmt::{Error as FmtError, Write};
use std::str::FromStr;
use std::time::Duration;

use crate::utils::display_with;

/// The struct to hold all the metadata and the lyric frames
#[derive(Clone, Debug, PartialEq)]
pub struct Lyric {
    /// Offset in milliseconds
    ///
    /// positive means delay lyric
    pub offset: i64,
    /// Text frames
    pub captions: Vec<Caption>,
}

/// A caption for a specific time
#[derive(Clone, Debug, PartialEq)]
pub struct Caption {
    /// Timestamp in milliseconds
    timestamp: i64,
    /// The text of the current caption, trimmed
    text: String,
}

impl Lyric {
    // GetText will fetch lyric by time in seconds
    pub fn get_text(&self, time: Duration) -> Option<String> {
        if self.captions.is_empty() {
            return None;
        };

        #[allow(clippy::cast_possible_wrap)]
        let mut time = time.as_secs() as i64;

        // here we want to show lyric 2 second earlier
        let mut adjusted_time = time * 1000 + 2000;
        adjusted_time += self.offset;
        if adjusted_time < 0 {
            adjusted_time = 0;
        }

        time = adjusted_time;

        let mut text = self.captions.first()?.text.clone();
        for caption in &self.captions {
            if time >= caption.timestamp {
                text.clone_from(&caption.text);
            } else {
                break;
            }
        }
        Some(text)
    }

    /// Get a index into the captions list for a specific time
    pub fn get_index(&self, mut time: i64) -> Option<usize> {
        if self.captions.is_empty() {
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
        for (i, caption) in self.captions.iter().enumerate() {
            if time >= caption.timestamp {
                index = i;
            } else {
                break;
            }
        }
        Some(index)
    }

    /// Adjust all captions in `time` by `offset`(milliseconds) and sort captions based on adjusted time
    pub fn adjust_offset(&mut self, time: Duration, offset: i64) {
        #[allow(clippy::cast_possible_wrap)]
        let time = time.as_secs() as i64;
        if let Some(index) = self.get_index(time) {
            // when time stamp is less than 10 seconds or index is before the first line, we adjust
            // the offset.
            if (index == 0) || (time < 11) {
                self.offset -= offset;
            } else {
                // fine tuning each line after 10 seconds
                let caption = &mut self.captions[index];
                let adjusted_time_stamp = caption.timestamp + offset;
                caption.timestamp = match adjusted_time_stamp.cmp(&0) {
                    Ordering::Greater | Ordering::Equal => adjusted_time_stamp,
                    Ordering::Less => 0,
                };
            }
        };
        // we sort the captions by time_stamp. This is to fix some lyrics downloaded are not sorted
        self.captions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    /// Format current [`Lyric`] as a LRC file
    pub fn as_lrc_text(&self) -> String {
        let mut result: String = String::new();
        if self.offset != 0 {
            // No known ways this could fail, ignore the result
            let _ = writeln!(&mut result, "[offset:{}]", self.offset);
        }

        for line in &self.captions {
            // No known ways this could fail, ignore the result
            let _ = line.as_lrc(&mut result);
        }
        result
    }

    /// Merge captions that are less than 2 seconds apart
    pub fn merge_adjacent(&mut self) {
        let mut merged_captions = self.captions.clone();
        let mut offset = 1;
        for (i, old_caption) in self.captions.iter().enumerate().skip(1) {
            if let Some(item) = merged_captions.get_mut(i - offset) {
                if old_caption.timestamp - item.timestamp < 2000 {
                    item.text += "  ";
                    item.text += old_caption.text.as_ref();
                    merged_captions.remove(i - offset + 1);
                    offset += 1;
                }
            }
        }

        self.captions = merged_captions;
    }
}

impl Caption {
    /// Try to parse a single [`Caption`]
    fn parse_line(line: &str) -> Option<Self> {
        //[00:12.00]Line 1 lyrics

        // plus 1 can always be done because "find" has found a instance (and returns before), and the character is ASCII
        // start index after the character
        let timestamp_start = line.find('[')? + 1;
        // theoretically, a LRC timestamp is always 8 characters long, but we do this to support longer possible values
        // end index before the character
        let timestamp_end = (line[timestamp_start..]).find(']')? + timestamp_start;
        // exclude the end character
        let text_start = timestamp_end + 1;

        let time_stamp = Self::parse_time(&line[timestamp_start..timestamp_end])?;
        let text = line[text_start..].to_string();

        Some(Self {
            timestamp: time_stamp.try_into().unwrap_or(0),
            text,
        })
    }

    /// Parse the time from a caption, the input needs to have the "[]" already removed
    ///
    /// LRC time is `mm:ss.xx` where `m` is minutes, `s` is seconds and `x` hundreths of a second (centis)
    /// or non-standard `mm:ss.xxx` where `x` is milliseconds
    fn parse_time(string: &str) -> Option<u64> {
        let double_idx = string.find(':')?;
        let dot_idx = string[double_idx..].find('.')? + double_idx;

        let minutes: u32 = string[..double_idx].parse().ok()?;
        let seconds: u32 = string[double_idx + 1..dot_idx].parse().ok()?;
        let centis_or_millis: u32 = string[dot_idx + 1..].parse().ok()?;

        // support non-standard ".xxx" (milliseconds)
        // will still have to below 1 second (999 milliseconds max)
        let millis = if centis_or_millis < 99 {
            centis_or_millis * 10
        } else {
            centis_or_millis
        };
        let sum_millis = (u64::from(minutes) * 60 + u64::from(seconds)) * 1000 + u64::from(millis);

        Some(sum_millis)
    }

    /// Format the current [`Caption`] as a LRC line
    fn as_lrc(&self, w: &mut impl Write) -> Result<(), FmtError> {
        writeln!(
            w,
            "[{}]{}",
            time_lrc(self.timestamp.try_into().unwrap_or(0)),
            self.text
        )
    }
}

/// Format the given timestamp as a LRC time: `mm:ss.ms`
fn time_lrc(time_stamp: u64) -> impl std::fmt::Display {
    let time_duration = Duration::from_millis(time_stamp);
    // LRC format does not handle hours, so this formatting assumes it is below 1 hour
    // let _h = time_duration.as_secs() / 3600;
    // modulate by 60 to keep it only to the current hour, instead of all the duration as minutes
    let m = (time_duration.as_secs() / 60) % 60;
    // modulate by 60 to keep it only to the current minute, instead of all the duration as seconds
    let s = time_duration.as_secs() % 60;
    // subsec is always guranteed to be less than a second; dividing by 10 to only have the 2 most significant numbers
    let ms = time_duration.subsec_millis() / 10;

    display_with(move |f| write!(f, "{m:02}:{s:02}.{ms:02}"))
}

impl FromStr for Lyric {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut offset: i64 = 0;
        let mut captions = Vec::new();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(remainder) = line.strip_prefix("[offset:") {
                let Some(end_idx) = remainder.find(']') else {
                    continue;
                };
                let offset_str = remainder[..end_idx].trim();
                if let Ok(o) = offset_str.parse() {
                    offset = o;
                    // no need to check the line again, there *should* be only one tag per line
                    continue;
                }
            }

            // skip all lines that do not start with "["
            if !line.as_bytes().iter().next().is_some_and(|v| *v == b'[') {
                continue;
            }

            if let Some(caption) = Caption::parse_line(line) {
                captions.push(caption);
            };
        }

        // we sort the captions by Timestamp. This is to fix some lyrics downloaded are not sorted
        captions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let mut lyric = Self { offset, captions };

        lyric.merge_adjacent();

        Ok(lyric)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn should_parse_simple() {
        let txt = r"[al:Album Title]
[ar:Performing Artist]
[by:Lyric creator]
[offset:+10]
[re:Lyric creator App]
[ve:Lyric creator version]
[ti:Song Title]
[au:Song Author]
[00:12.00]Lyrics beginning ...
[00:15.30]Some more lyrics ...
[10:11.12]Extra Lyrics";

        let lyrics = Lyric::from_str(txt).unwrap();

        assert_eq!(lyrics.offset, 10);

        assert_eq!(
            lyrics.captions.as_slice(),
            &[
                Caption {
                    timestamp: 12 * 1000,
                    text: "Lyrics beginning ...".into()
                },
                Caption {
                    timestamp: (15 * 1000) + 300,
                    text: "Some more lyrics ...".into()
                },
                Caption {
                    timestamp: (10 * 60 * 1000) + (11 * 1000) + 120,
                    text: "Extra Lyrics".into()
                },
            ]
        );
    }

    #[test]
    fn should_parse_minimal() {
        let txt = r"[00:12.00]Lyrics beginning ...";

        let lyrics = Lyric::from_str(txt).unwrap();

        assert_eq!(lyrics.offset, 0);

        assert_eq!(
            lyrics.captions.as_slice(),
            &[Caption {
                timestamp: 12 * 1000,
                text: "Lyrics beginning ...".into()
            },]
        );
    }

    #[test]
    fn should_parse_milliseconds() {
        let txt = r"[00:12.305]Lyrics beginning ...";

        let lyrics = Lyric::from_str(txt).unwrap();

        assert_eq!(lyrics.offset, 0);

        assert_eq!(
            lyrics.captions.as_slice(),
            &[Caption {
                timestamp: 12 * 1000 + 305,
                text: "Lyrics beginning ...".into()
            },]
        );
    }

    #[test]
    fn should_handle_empty() {
        let txt = "";

        let lyrics = Lyric::from_str(txt).unwrap();

        assert_eq!(lyrics.captions.len(), 0);
    }

    #[test]
    fn should_format_as_lrc() {
        let lyrics = Lyric {
            offset: 10,
            captions: vec![
                Caption {
                    timestamp: 12 * 1000,
                    text: "Lyrics beginning ...".into(),
                },
                Caption {
                    timestamp: (15 * 1000) + 300,
                    text: "Some more lyrics ...".into(),
                },
                Caption {
                    timestamp: (10 * 60 * 1000) + (11 * 1000) + 120,
                    text: "Extra Lyrics".into(),
                },
            ],
        };

        assert_eq!(
            lyrics.as_lrc_text(),
            r"[offset:10]
[00:12.00]Lyrics beginning ...
[00:15.30]Some more lyrics ...
[10:11.12]Extra Lyrics
"
        );
    }
}
