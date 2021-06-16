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
pub struct Lyric {
    album: Option<String>,
    artist: Option<String>,
    by_creator: Option<String>,       // Creator of LRC file
    offset: i32,                      // positive means delay lyric
    re_player_editor: Option<String>, // Player or Editor to create this LRC file
    title: Option<String>,
    version_player_editor: Option<String>, // Version of player or editor
    lang_extension: Option<String>,
    unsynced_captions: Vec<UnsyncedCaption>, // USLT captions
                                             // SyncedCaptions      []id3v2.SyncedText // SYLT captions
}

pub struct UnsyncedCaption {
    time_stamp: u32,
    text: Option<String>,
}

const EOL: &str = "\n";

pub fn looks_like_lrc(s: String) -> bool {
    if s != "" {
        if s.chars().nth(0).unwrap() == '[' {
            return true;
        }
    }
    false
}
