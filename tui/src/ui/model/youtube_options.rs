use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use id3::TagLike;
use id3::Version::Id3v24;
use regex::Regex;
use shell_words;
use termusiclib::invidious::{Instance, YoutubeVideo};
use termusiclib::track::DurationFmtShort;
use termusiclib::utils::get_parent_folder;
use tuirealm::props::{Alignment, AttrValue, Attribute, TableBuilder, TextSpan};
use tuirealm::{State, StateValue};
use ytd_rs::{Arg, YoutubeDL};

use super::Model;
use crate::ui::ids::Id;
use crate::ui::msg::{Msg, YSMsg};

#[expect(dead_code)]
static RE_FILENAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[ffmpeg\] Destination: (?P<name>.*)\.mp3").unwrap());

static RE_FILENAME_YTDLP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[ExtractAudio\] Destination: (?P<name>.*)\.mp3").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YoutubeData {
    pub items: Vec<YoutubeVideo>,
    pub page: u32,
}

impl Default for YoutubeData {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            page: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct YoutubeOptions {
    pub data: YoutubeData,
    pub invidious_instance: Instance,
}

impl YoutubeOptions {
    pub fn get_by_index(&self, index: usize) -> Result<&YoutubeVideo> {
        if let Some(item) = self.data.items.get(index) {
            return Ok(item);
        }
        Err(anyhow!("index not found"))
    }

    /// Fetch the previous page's content if there is a previous page.
    ///
    /// The returned Future does not need the lifetime of `self` for the fetch and is safe to [`Send`].
    pub fn get_prev_page(&self) -> Option<impl Future<Output = Result<YoutubeData>> + use<>> {
        if self.data.page > 1 {
            let mut res = YoutubeData {
                page: self.data.page - 1,
                ..Default::default()
            };
            let instance = self.invidious_instance.clone();

            return Some(async move {
                res.items = instance.get_search_query(res.page).await?;
                Ok(res)
            });
        }

        None
    }

    /// Fetch the next page's content.
    ///
    /// The returned Future does not need the lifetime of `self` for the fetch and is safe to [`Send`].
    pub fn get_next_page(&self) -> impl Future<Output = Result<YoutubeData>> + use<> {
        let mut res = YoutubeData {
            page: self.data.page + 1,
            ..Default::default()
        };
        let instance = self.invidious_instance.clone();

        async move {
            res.items = instance.get_search_query(res.page).await?;
            Ok(res)
        }
    }

    #[must_use]
    pub const fn page(&self) -> u32 {
        self.data.page
    }

    pub fn is_empty(&self) -> bool {
        self.data.items.is_empty()
    }

    pub fn items(&self) -> &[YoutubeVideo] {
        &self.data.items
    }
}

impl Model {
    pub fn youtube_options_download(&mut self, index: usize) -> Result<()> {
        // download from search result here
        if let Ok(item) = self.youtube_options.get_by_index(index) {
            let url = format!("https://www.youtube.com/watch?v={}", item.video_id);
            if let Err(e) = self.youtube_dl(url.as_ref()) {
                bail!("Download error: {e}");
            }
        }
        Ok(())
    }

    /// This function requires to be run in a tokio Runtime context
    pub fn youtube_options_search(&mut self, keyword: String) {
        let tx = self.tx_to_main.clone();
        tokio::spawn(async move {
            match Instance::new(&keyword).await {
                Ok((instance, result)) => {
                    let youtube_options = YoutubeOptions {
                        data: YoutubeData {
                            items: result,
                            page: 1,
                        },
                        invidious_instance: instance,
                    };
                    tx.send(Msg::YoutubeSearch(YSMsg::YoutubeSearchSuccess(
                        youtube_options,
                    )))
                    .ok();
                }
                Err(e) => {
                    tx.send(Msg::YoutubeSearch(YSMsg::YoutubeSearchFail(e.to_string())))
                        .ok();
                }
            }
        });
    }

    /// This function requires to be run in a tokio Runtime context
    pub fn youtube_options_prev_page(&self) {
        let tx_to_main = self.tx_to_main.clone();

        let Some(fut) = self.youtube_options.get_prev_page() else {
            return;
        };

        tokio::task::spawn(async move {
            match fut.await {
                Ok(data) => {
                    let _ = tx_to_main.send(Msg::YoutubeSearch(YSMsg::PageLoaded(data)));
                }
                Err(err) => {
                    let _ =
                        tx_to_main.send(Msg::YoutubeSearch(YSMsg::PageLoadError(err.to_string())));
                }
            }
        });
    }

    /// This function requires to be run in a tokio Runtime context
    pub fn youtube_options_next_page(&mut self) {
        let tx_to_main = self.tx_to_main.clone();

        let fut = self.youtube_options.get_next_page();

        tokio::task::spawn(async move {
            match fut.await {
                Ok(data) => {
                    let _ = tx_to_main.send(Msg::YoutubeSearch(YSMsg::PageLoaded(data)));
                }
                Err(err) => {
                    let _ =
                        tx_to_main.send(Msg::YoutubeSearch(YSMsg::PageLoadError(err.to_string())));
                }
            }
        });
    }

    pub fn sync_youtube_options(&mut self) {
        if self.youtube_options.is_empty() {
            let table = TableBuilder::default()
                .add_col(TextSpan::from("No results."))
                .add_col(TextSpan::from(
                    "Nothing was found in 10 seconds, connection issue encountered.",
                ))
                .build();
            self.app
                .attr(
                    &Id::YoutubeSearchTablePopup,
                    Attribute::Content,
                    AttrValue::Table(table),
                )
                .ok();
            return;
        }

        let mut table: TableBuilder = TableBuilder::default();
        for (idx, record) in self.youtube_options.items().iter().enumerate() {
            if idx > 0 {
                table.add_row();
            }
            let duration = DurationFmtShort(Duration::from_secs(record.length_seconds));
            let duration_string = format!("[{duration:^10.10}]");

            let title = record.title.as_str();

            table
                .add_col(TextSpan::new(duration_string))
                .add_col(TextSpan::new(title).bold());
        }
        let table = table.build();
        self.app
            .attr(
                &Id::YoutubeSearchTablePopup,
                Attribute::Content,
                AttrValue::Table(table),
            )
            .ok();

        if let Some(domain) = &self.youtube_options.invidious_instance.domain {
            let title = format!(
                "\u{2500}\u{2500}\u{2500} Page {} \u{2500}\u{2500}\u{2500}\u{2524} {} \u{251c}\u{2500}\u{2500} {} \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                self.youtube_options.page(),
                "Tab/Shift+Tab switch pages",
                domain,
            );
            self.app
                .attr(
                    &Id::YoutubeSearchTablePopup,
                    Attribute::Title,
                    AttrValue::Title((title, Alignment::Left)),
                )
                .ok();
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn youtube_dl(&mut self, url: &str) -> Result<()> {
        let mut path: PathBuf = std::env::temp_dir();
        if let Ok(State::One(StateValue::String(node_id))) = self.app.state(&Id::Library) {
            path = get_parent_folder(Path::new(&node_id)).to_path_buf();
        }
        let config_tui = self.config_tui.read();
        let mut args = vec![
            Arg::new("--no-playlist"),
            Arg::new("--extract-audio"),
            // Arg::new_with_arg("--audio-format", "vorbis"),
            Arg::new_with_arg("--audio-format", "mp3"),
            Arg::new("--add-metadata"),
            Arg::new("--embed-thumbnail"),
            Arg::new_with_arg("--metadata-from-title", "%(artist) - %(title)s"),
            #[cfg(target_os = "windows")]
            Arg::new("--restrict-filenames"),
            Arg::new("--write-sub"),
            Arg::new("--all-subs"),
            Arg::new_with_arg("--convert-subs", "lrc"),
            Arg::new_with_arg("--output", "%(title).90s.%(ext)s"),
        ];
        let extra_args = parse_args(&config_tui.settings.ytdlp.extra_args)
            .context("Parsing config `extra_ytdlp_args`")?;
        let mut extra_args_parsed = convert_to_args(extra_args);
        if !extra_args_parsed.is_empty() {
            args.append(&mut extra_args_parsed);
        }

        let ytd = YoutubeDL::new(&path, args, url)?;
        let tx = self.tx_to_main.clone();

        // avoid full string clones when sending via a channel
        let url: Arc<str> = Arc::from(url);

        thread::spawn(move || -> Result<()> {
            tx.send(Msg::YoutubeSearch(YSMsg::Download(YTDLMsg::Start(
                url.clone(),
                "youtube music".to_string(),
            ))))
            .ok();
            // start download
            let download = ytd.download();

            // check what the result is and print out the path to the download or the error
            match download {
                Ok(result) => {
                    tx.send(Msg::YoutubeSearch(YSMsg::Download(YTDLMsg::Success(
                        url.clone(),
                    ))))
                    .ok();
                    // here we extract the full file name from download output
                    if let Some(file_fullname) =
                        extract_filepath(result.output(), &path.to_string_lossy())
                    {
                        tx.send(Msg::YoutubeSearch(YSMsg::Download(YTDLMsg::Completed(
                            url,
                            Some(file_fullname.clone()),
                        ))))
                        .ok();

                        // here we remove downloaded live_chat.json file
                        remove_downloaded_json(&path, &file_fullname);

                        embed_downloaded_lrc(&path, &file_fullname);
                    } else {
                        tx.send(Msg::YoutubeSearch(YSMsg::Download(YTDLMsg::Completed(
                            url, None,
                        ))))
                        .ok();
                    }
                }
                Err(e) => {
                    tx.send(Msg::YoutubeSearch(YSMsg::Download(YTDLMsg::Err(
                        url.clone(),
                        "youtube music".to_string(),
                        e.to_string(),
                    ))))
                    .ok();
                    tx.send(Msg::YoutubeSearch(YSMsg::Download(YTDLMsg::Completed(
                        url, None,
                    ))))
                    .ok();
                }
            }
            Ok(())
        });
        Ok(())
    }
}

pub type YTDLMsgURL = Arc<str>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum YTDLMsg {
    /// Indicates a Start of a download.
    ///
    /// `(Url, Title)`
    Start(YTDLMsgURL, String),
    /// Indicates the Download was a Success, though termusic post-processing is not done yet.
    ///
    /// `(Url)`
    Success(YTDLMsgURL),
    /// Indicates the Download thread finished in both Success or Error.
    ///
    /// `(Url, Filename)`
    Completed(YTDLMsgURL, Option<String>),
    /// Indicates that the Download has Errored and has been aborted.
    ///
    /// `(Url, Title, ErrorAsString)`
    Err(YTDLMsgURL, String, String),
}

// This just parsing the output from youtubedl to get the audio path
// This is used because we need to get the song name
// example ~/path/to/song/song.mp3
fn extract_filepath(output: &str, dir: &str) -> Option<String> {
    // #[cfg(not(feature = "yt-dlp"))]
    // if let Some(cap) = RE_FILENAME.captures(output) {
    //     if let Some(c) = cap.name("name") {
    //         let filename = format!("{}/{}.mp3", dir, c.as_str());
    //         return Ok(filename);
    //     }
    // }
    // #[cfg(feature = "yt-dlp")]
    if let Some(cap) = RE_FILENAME_YTDLP.captures(output) {
        if let Some(c) = cap.name("name") {
            let filename = format!("{dir}/{}.mp3", c.as_str());
            return Some(filename);
        }
    }
    None
}

fn remove_downloaded_json(path: &Path, file_fullname: &str) {
    let files = walkdir::WalkDir::new(path).follow_links(true);
    for f in files
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| {
            let p = Path::new(f.file_name());
            p.extension().is_some_and(|ext| ext == "json")
        })
        .filter(|f| {
            let path_json = Path::new(f.file_name());
            let p1: &Path = Path::new(file_fullname);
            path_json.file_stem().is_some_and(|stem_lrc| {
                p1.file_stem().is_some_and(|p_base| {
                    stem_lrc
                        .to_string_lossy()
                        .contains(p_base.to_string_lossy().as_ref())
                })
            })
        })
    {
        std::fs::remove_file(f.path()).ok();
    }
}

fn embed_downloaded_lrc(path: &Path, file_fullname: &str) {
    let mut id3_tag = if let Ok(tag) = id3::Tag::read_from_path(file_fullname) {
        tag
    } else {
        let mut tags = id3::Tag::new();
        let file_path = Path::new(file_fullname);
        if let Some(p_base) = file_path.file_stem() {
            tags.set_title(p_base.to_string_lossy());
        }
        tags.write_to_path(file_path, Id3v24).ok();
        tags
    };

    // here we add all downloaded lrc file
    let files = walkdir::WalkDir::new(path).follow_links(true);

    for entry in files
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .filter(|f| {
            let name = f.file_name();
            let p = Path::new(&name);
            p.extension().is_some_and(|ext| ext == "lrc")
        })
        .filter(|f| {
            let path_lrc = Path::new(f.file_name());
            let p1: &Path = Path::new(file_fullname);
            path_lrc.file_stem().is_some_and(|stem_lrc| {
                p1.file_stem().is_some_and(|p_base| {
                    stem_lrc
                        .to_string_lossy()
                        .contains(p_base.to_string_lossy().as_ref())
                })
            })
        })
    {
        let path_lrc = Path::new(entry.file_name());
        let mut lang_ext = "eng".to_string();
        if let Some(p_short) = path_lrc.file_stem() {
            let p2 = Path::new(p_short);
            if let Some(ext2) = p2.extension() {
                lang_ext = ext2.to_string_lossy().to_string();
            }
        }
        let lyric_string = std::fs::read_to_string(entry.path());
        id3_tag.add_frame(id3::frame::Lyrics {
            lang: "eng".to_string(),
            description: lang_ext,
            text: lyric_string.unwrap_or_else(|_| String::from("[00:00:01] No lyric")),
        });
        std::fs::remove_file(entry.path()).ok();
    }

    id3_tag.write_to_path(file_fullname, Id3v24).ok();
}

#[derive(Debug, Clone, PartialEq)]
enum ArgOrVal {
    ArgumentWithVal(String),
    Flag(String),
    Argument(String),
    Positional(String),
}

/// Parse the input shell-like string into a Vector of `argument` and `maybe argument value`.
fn parse_args(input: &str) -> Result<Vec<ArgOrVal>, shell_words::ParseError> {
    let result = shell_words::split(input)?
        .into_iter()
        .map(|token| {
            if token.starts_with("--") {
                if token.contains('=') {
                    ArgOrVal::ArgumentWithVal(token)
                } else {
                    ArgOrVal::Argument(token)
                }
            } else if token.starts_with('-') {
                ArgOrVal::Flag(token)
            } else {
                ArgOrVal::Positional(token)
            }
        })
        .collect();
    Ok(result)
}

/// Convert the `argument, maybe value` vector to [ytdrs Arguments](Arg).
fn convert_to_args(extra_args: Vec<ArgOrVal>) -> Vec<Arg> {
    // This capacity *may* be a little inaccurate, but should broadly reflect what we need
    let mut extra_args_parsed = Vec::with_capacity(extra_args.len());

    // store the last "maybe incomplete" argument here
    // this has to be done because ytdrs `Arg` are non-modifiable after creation.
    let mut last_arg: Option<String> = None;

    for val in extra_args {
        // push last arg to the array, before processing a new one
        match &val {
            ArgOrVal::ArgumentWithVal(_) | ArgOrVal::Argument(_) | ArgOrVal::Flag(_) => {
                if let Some(v) = last_arg.take() {
                    extra_args_parsed.push(Arg::new(&v));
                }
            }
            ArgOrVal::Positional(_) => (),
        }

        match val {
            ArgOrVal::ArgumentWithVal(v) | ArgOrVal::Flag(v) => {
                extra_args_parsed.push(Arg::new(&v));
            }
            ArgOrVal::Argument(v) => {
                last_arg = Some(v);
            }
            ArgOrVal::Positional(v) => {
                let Some(last_arg) = last_arg.take() else {
                    // in case there is a positional but no previous argument to combine with, skip the positional with a error
                    // maybe we should error instead?
                    error!("Positional without previous argument! {v:#?}");
                    continue;
                };
                extra_args_parsed.push(Arg::new_with_arg(&last_arg, &v));
            }
        }
    }

    if let Some(remainder) = last_arg {
        extra_args_parsed.push(Arg::new(&remainder));
    }

    extra_args_parsed
}

#[cfg(test)]
mod tests {

    use crate::ui::model::youtube_options::extract_filepath;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_youtube_output_parsing() {
        // #[cfg(not(feature = "yt-dlp"))]
        // assert_eq!(
        //     extract_filepath(
        //         r"sdflsdf [ffmpeg] Destination: 观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3 sldflsdfj",
        //         "/tmp"
        //     )
        //     .unwrap(),
        //     "/tmp/观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3".to_string()
        // );
        assert_eq!(
            extract_filepath(
                r"sdflsdf [ExtractAudio] Destination: 观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3 sldflsdfj",
                "/tmp"
            )
            .unwrap(),
            "/tmp/观众说“小哥哥，到饭点了”《干饭人之歌》走，端起饭盆干饭去.mp3".to_string()
        );
    }
}
