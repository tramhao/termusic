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
use regex::Regex;
use std::str::FromStr;

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

// const EOL: &str = "\n";

pub fn looks_like_lrc(s: String) -> bool {
    if s != "" {
        if s.chars().nth(0).unwrap() == '[' {
            return true;
        }
    }
    false
}

impl Lyric {
    // NewFromLRC parses a .lrc text into Subtitle, assumes s is a clean utf8 string
    // GetText will fetch lyric by time in seconds
    pub fn get_text(&self, time: i64) -> Option<String> {
        if self.unsynced_captions.len() < 1 {
            return None;
        };

        // here we want to show lyric 1 second earlier
        let mut time = time * 1000 + 1000;
        time += self.offset;

        let mut text: String = self.unsynced_captions[0].text.clone();
        for v in self.unsynced_captions.iter() {
            if time >= v.time_stamp as i64 {
                text = v.text.clone();
            } else {
                break;
            }
        }
        Some(text)
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
            let mut line = String::from(line);
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            let line = line.trim();
            let mut line = String::from(line);
            if line.is_empty() {
                continue;
            }

            if line.starts_with("[offset") {
                let line = line.trim_start_matches("[offset:");
                let line = line.trim_end_matches("]");
                let line = line.replace(" ", "");
                offset = line.parse().unwrap();
            }
            let time_stamp_re = Regex::new(
                r"(?x)
                                          [\d{2}
                                           :
                                           ]
                                           ",
            )
            .unwrap();
            let caps = time_stamp_re.captures(line.as_ref()).unwrap();

            if caps.len() < 1 {
                continue;
            }

            match UnsyncedCaption::parse_line(&mut line) {
                Ok(s) => unsynced_captions.push(s),
                Err(_) => {}
            };
        }

        // we sort the cpations by Timestamp. This is to fix some lyrics downloaded are not sorted
        unsynced_captions.sort_by(|b, a| b.time_stamp.cmp(&a.time_stamp));
        // lyric.mergeLRC()

        // return
        Ok(Lyric {
            lang_extension,
            offset,
            unsynced_captions,
        })
    }
}
