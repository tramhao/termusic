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
use super::{
    Model,
    UpdateComponents::{
        DownloadCompleted, DownloadErrDownload, DownloadRunning, DownloadSuccess,
        YoutubeSearchFail, YoutubeSearchSuccess,
    },
};
use crate::invidious::{Instance, YoutubeVideo};
use anyhow::{anyhow, bail, Result};
use humantime::format_duration;
use id3::frame::Lyrics;
use id3::Version::Id3v24;
use if_chain::if_chain;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, sleep};
use std::time::Duration;
use tuirealm::props::{TableBuilder, TextSpan};
use ytd_rs::{Arg, YoutubeDL, YoutubeDLResult};

lazy_static! {
    static ref RE_FILENAME: Regex =
        Regex::new(r"\[ffmpeg\] Destination: (?P<name>.*)\.mp3").unwrap();
}

pub struct YoutubeOptions {
    items: Vec<YoutubeVideo>,
    page: u32,
    invidious_instance: Instance,
}

impl YoutubeOptions {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            page: 1,
            invidious_instance: crate::invidious::Instance::default(),
        }
    }
    pub fn get_by_index(&self, index: usize) -> Result<&YoutubeVideo> {
        if let Some(item) = self.items.get(index) {
            return Ok(item);
        }
        Err(anyhow!("index not found"))
    }

    pub fn prev_page(&mut self) -> Result<()> {
        if self.page > 1 {
            self.page -= 1;
            self.items = self.invidious_instance.get_search_query(self.page)?;
        }
        Ok(())
    }

    pub fn next_page(&mut self) -> Result<()> {
        self.page += 1;
        self.items = self.invidious_instance.get_search_query(self.page)?;
        Ok(())
    }

    pub const fn page(&self) -> u32 {
        self.page
    }
}

impl Model {
    pub fn youtube_options_download(&mut self, index: usize) -> Result<()> {
        // download from search result here
        let mut url = "https://www.youtube.com/watch?v=".to_string();
        if let Ok(item) = self.youtube_options.get_by_index(index) {
            url.push_str(&item.video_id);
            if let Err(e) = self.youtube_dl(url.as_ref()) {
                bail!("Error download: {}", e);
            }
        }
        Ok(())
    }

    pub fn youtube_options_search(&mut self, keyword: &str) {
        let search_word = keyword.to_string();
        let tx = self.sender.clone();
        thread::spawn(
            move || match crate::invidious::Instance::new(&search_word) {
                Ok((instance, result)) => {
                    let youtube_options = YoutubeOptions {
                        items: result,
                        page: 1,
                        invidious_instance: instance,
                    };
                    tx.send(YoutubeSearchSuccess(youtube_options)).ok();
                }
                Err(e) => {
                    tx.send(YoutubeSearchFail(e.to_string())).ok();
                }
            },
        );
    }

    pub fn youtube_options_prev_page(&mut self) {
        match self.youtube_options.prev_page() {
            Ok(_) => self.sync_youtube_options(),
            Err(e) => self.mount_error_popup(format!("search error: {}", e).as_str()),
        }
    }
    pub fn youtube_options_next_page(&mut self) {
        match self.youtube_options.next_page() {
            Ok(_) => self.sync_youtube_options(),
            Err(e) => self.mount_error_popup(format!("search error: {}", e).as_str()),
        }
    }
    pub fn sync_youtube_options(&mut self) {
        if self.youtube_options.items.is_empty() {
            // if let Some(props) = self.view.get_props(COMPONENT_TABLE_YOUTUBE) {
            //     let props = TablePropsBuilder::from(props)
            //         .with_table(
            //             TableBuilder::default()
            //                 .add_col(TextSpan::from("Empty result."))
            //                 .add_col(TextSpan::from(
            //                     "Wait 10 seconds but no results, means all servers are down.",
            //                 ))
            //                 .build(),
            //         )
            //         .build();
            //     let msg = self.view.update(COMPONENT_TABLE_YOUTUBE, props);
            //     self.update(&msg);
            // }
            return;
        }

        let mut table: TableBuilder = TableBuilder::default();
        for (idx, record) in self.youtube_options.items.iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let duration = format_duration(Duration::from_secs(record.length_seconds)).to_string();
            let duration_string = format!("[{:^10.10}]", duration);

            let title = record.title.as_str();

            table
                .add_col(TextSpan::new(duration_string))
                .add_col(TextSpan::new(title).bold());
        }
        let table = table.build();

        // if let Some(props) = self.view.get_props(COMPONENT_TABLE_YOUTUBE) {
        //     if let Some(domain) = &self.youtube_options.invidious_instance.domain {
        //         let title = format!(
        //             "\u{2500}\u{2500}\u{2500} Page {} \u{2500}\u{2500}\u{2500}\u{2524} {} \u{251c}\u{2500}\u{2500} {} \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        //             self.youtube_options.page(),
        //             "Tab/Shift+Tab switch pages",
        //             domain,
        //         );
        //         let props = TablePropsBuilder::from(props)
        //             .with_title(title, tuirealm::tui::layout::Alignment::Left)
        //             .with_header(&["Duration", "Name"])
        //             .with_table(table)
        //             .build();
        //         self.view.update(COMPONENT_TABLE_YOUTUBE, props);
        //     }
        // }
    }

    pub fn youtube_dl(&mut self, link: &str) -> Result<()> {
        let mut path: PathBuf = PathBuf::new();
        // if let Some(Payload::One(Value::Str(node_id))) =
        //     self.view.get_state(COMPONENT_TREEVIEW_LIBRARY)
        // {
        //     let p: &Path = Path::new(node_id.as_str());
        //     if p.is_dir() {
        //         path = PathBuf::from(p);
        //     } else if let Some(p) = p.parent() {
        //         path = p.to_path_buf();
        //     }
        // }

        let args = vec![
            Arg::new("--extract-audio"),
            Arg::new_with_arg("--audio-format", "mp3"),
            Arg::new("--add-metadata"),
            Arg::new("--embed-thumbnail"),
            Arg::new_with_arg("--metadata-from-title", "%(artist) - %(title)s"),
            Arg::new("--write-sub"),
            Arg::new("--all-subs"),
            Arg::new_with_arg("--convert-subs", "lrc"),
            Arg::new_with_arg("--output", "%(title).90s.%(ext)s"),
        ];

        let ytd = YoutubeDL::new(&path, args, link)?;
        let tx = self.sender.clone();

        thread::spawn(move || -> Result<()> {
            tx.send(DownloadRunning).ok();
            // start download
            let download = ytd.download()?;

            // check what the result is and print out the path to the download or the error
            // match download. result_type() {
            //     YoutubeDLResult::SUCCESS => {
            //         // here we extract the full file name from download output
            //         if let Ok(file_fullname) =
            //             extract_filepath(download.output(), &path.to_string_lossy())
            //         {
            //             let mut id3_tag = if let Ok(tag) = id3::Tag::read_from_path(&file_fullname)
            //             {
            //                 tag
            //             } else {
            //                 let mut t = id3::Tag::new();
            //                 let p: &Path = Path::new(&file_fullname);
            //                 if let Some(p_base) = p.file_stem() {
            //                     t.set_title(p_base.to_string_lossy());
            //                 }
            //                 t.write_to_path(p, Id3v24).ok();
            //                 t
            //             };

            //             // here we add all downloaded lrc file
            //             if let Ok(files) = std::fs::read_dir(&path) {
            //                 for f in files.flatten() {
            //                     let name = f.file_name().clone();
            //                     let p = Path::new(&name);
            //                     if_chain! {
            //                         if let Some(ext) = p.extension();
            //                         if ext == "lrc";
            //                         if let Some(stem_lrc) = p.file_stem();
            //                         let p1: &Path = Path::new(&file_fullname);
            //                         if let Some(p_base) = p1.file_stem();
            //                         if stem_lrc
            //                             .to_string_lossy()
            //                             .to_string()
            //                             .contains(p_base.to_string_lossy().as_ref());

            //                         then {
            //                             let mut lang_ext = "eng".to_string();
            //                             if let Some(p_short) = p.file_stem() {
            //                                 let p2 = Path::new(p_short);
            //                                 if let Some(ext2) = p2.extension() {
            //                                     lang_ext =
            //                                         ext2.to_string_lossy().to_string();
            //                                 }
            //                             }
            //                             let lyric_string =
            //                                 std::fs::read_to_string(f.path());
            //                             id3_tag.add_lyrics(Lyrics {
            //                                 lang: "eng".to_string(),
            //                                 description: lang_ext,
            //                                 text: lyric_string.unwrap_or_else(|_| {
            //                                     String::from("[00:00:01] No lyric")
            //                                 }),
            //                             });
            //                             std::fs::remove_file(f.path()).ok();
            //                         }
            //                     }
            //                 }
            //             }

            //             id3_tag.write_to_path(&file_fullname, Id3v24).ok();
            //             tx.send(DownloadSuccess).ok();
            //             sleep(Duration::from_secs(5));
            //             tx.send(DownloadCompleted(Some(file_fullname))).ok();
            //         } else {
            //             // This shoudn't happen unless the output format of youtubedl changed
            //             tx.send(DownloadSuccess).ok();
            //             sleep(Duration::from_secs(5));
            //             tx.send(DownloadCompleted(None)).ok();
            //         }
            //     }
            //     ResultType::IOERROR | ResultType::FAILURE => {
            //         tx.send(DownloadErrDownload).ok();
            //         sleep(Duration::from_secs(5));
            //         tx.send(DownloadCompleted(None)).ok();
            //     }
            // }
            Ok(())
        });
        Ok(())
    }
}
// This just parsing the output from youtubedl to get the audio path
// This is used because we need to get the song name
// example ~/path/to/song/song.mp3
pub fn extract_filepath(output: &str, dir: &str) -> Result<String> {
    let mut filename = String::new();
    filename.push_str(dir);
    filename.push('/');

    if let Some(cap) = RE_FILENAME.captures(output) {
        match cap.name("name") {
            Some(c) => filename.push_str(c.as_str()),
            None => bail!("parsing output error"),
        }
    }

    filename.push_str(".mp3");

    Ok(filename)
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {

    use crate::ui::model::youtube_options::extract_filepath;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_youtube_output_parsing() {
        assert_eq!(
            extract_filepath(
                r"sdflsdf [ffmpeg] Destination: 观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3 sldflsdfj",
                "/tmp"
            )
            .unwrap(),
            "/tmp/观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3".to_string()
        );
    }
}
