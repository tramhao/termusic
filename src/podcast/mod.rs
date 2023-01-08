// Thanks to the author of shellcaster(https://github.com/jeff-hughes/shellcaster). Most parts of following code are taken from it.

#[allow(unused)]
pub mod db;

use crate::config::Settings;
use crate::ui::{Msg, PCMsg};
use crate::utils::StringUtils;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use db::Database;
use lazy_static::lazy_static;
use opml::{Body, Head, Outline, OPML};
use regex::{Match, Regex};
use rfc822_sanitizer::parse_from_rfc2822_with_fallback;
use rss::{Channel, Item};
use sanitize_filename::{sanitize_with_options, Options};
use std::cmp::Ordering;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    mpsc::{self, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

// How many columns we need, minimum, before we display the
// (unplayed/total) after the podcast title
pub const PODCAST_UNPLAYED_TOTALS_LENGTH: usize = 25;

// How many columns we need, minimum, before we display the duration of
// the episode
pub const EPISODE_DURATION_LENGTH: usize = 45;

// How many columns we need, minimum, before we display the pubdate
// of the episode
pub const EPISODE_PUBDATE_LENGTH: usize = 60;

lazy_static! {
    /// Regex for parsing an episode "duration", which could take the form
    /// of HH:MM:SS, MM:SS, or SS.
    static ref RE_DURATION: Regex = Regex::new(r"(\d+)(?::(\d+))?(?::(\d+))?").expect("Regex error");

    /// Regex for removing "A", "An", and "The" from the beginning of
    /// podcast titles
    static ref RE_ARTICLES: Regex = Regex::new(r"^(a|an|the) ").expect("Regex error");

}

/// Defines interface used for both podcasts and episodes, to be
/// used and displayed in menus.
pub trait Menuable {
    fn get_id(&self) -> i64;
    fn get_title(&self, length: usize) -> String;
    fn is_played(&self) -> bool;
}

/// Struct holding data about an individual podcast feed. This includes a
/// (possibly empty) vector of episodes.
#[derive(Debug, Clone)]
pub struct Podcast {
    pub id: i64,
    pub title: String,
    pub sort_title: String,
    pub url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub explicit: Option<bool>,
    pub last_checked: DateTime<Utc>,
    pub episodes: Vec<Episode>,
}

impl Podcast {
    // Counts and returns the number of unplayed episodes in the podcast.
    pub fn num_unplayed(&self) -> usize {
        self.episodes
            .iter()
            .map(|ep| usize::from(!ep.is_played()))
            .sum()
    }
}

impl Menuable for Podcast {
    /// Returns the database ID for the podcast.
    fn get_id(&self) -> i64 {
        self.id
    }

    /// Returns the title for the podcast, up to length characters.
    fn get_title(&self, length: usize) -> String {
        let mut title_length = length;

        // if the size available is big enough, we add the unplayed data
        // to the end
        if length > PODCAST_UNPLAYED_TOTALS_LENGTH {
            let meta_str = format!("({}/{})", self.num_unplayed(), self.episodes.len());
            title_length = length - meta_str.chars().count() - 3;

            let out = self.title.substr(0, title_length);

            format!(
                " {out} {meta_str:>width$} ",
                width = length - out.grapheme_len() - 3
            ) // this pads spaces between title and totals
        } else {
            format!(" {} ", self.title.substr(0, title_length - 2))
        }
    }

    fn is_played(&self) -> bool {
        self.num_unplayed() == 0
    }
}

impl PartialEq for Podcast {
    fn eq(&self, other: &Self) -> bool {
        self.sort_title == other.sort_title
    }
}
impl Eq for Podcast {}

impl PartialOrd for Podcast {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Podcast {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_title.cmp(&other.sort_title)
    }
}

/// Struct holding data about an individual podcast episode. Most of this
/// is metadata, but if the episode has been downloaded to the local
/// machine, the filepath will be included here as well. `played`
/// indicates whether the podcast has been marked as played or unplayed.
#[derive(Debug, Clone)]
pub struct Episode {
    pub id: i64,
    pub pod_id: i64,
    pub title: String,
    pub url: String,
    pub guid: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
    pub path: Option<PathBuf>,
    pub played: bool,
    pub last_position: Option<i64>,
}

impl Episode {
    /// Formats the duration in seconds into an HH:MM:SS format.
    pub fn format_duration(&self) -> String {
        match self.duration {
            Some(dur) => {
                let mut seconds = dur;
                let hours = seconds / 3600;
                seconds -= hours * 3600;
                let minutes = seconds / 60;
                seconds -= minutes * 60;
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            }
            None => "--:--:--".to_string(),
        }
    }
}

impl Menuable for Episode {
    /// Returns the database ID for the episode.
    fn get_id(&self) -> i64 {
        self.id
    }

    /// Returns the title for the episode, up to length characters.
    fn get_title(&self, length: usize) -> String {
        let out = match self.path {
            Some(_) => {
                let title = self.title.substr(0, length - 4);
                format!("[D] {title}")
            }
            None => self.title.substr(0, length),
        };
        if length > EPISODE_PUBDATE_LENGTH {
            let dur = self.format_duration();
            let meta_dur = format!("[{dur}]");

            if let Some(pubdate) = self.pubdate {
                // print pubdate and duration
                let pd = pubdate.format("%F");
                let meta_str = format!("({pd}) {meta_dur}");
                let added_len = meta_str.chars().count();

                let out_added = out.substr(0, length - added_len - 3);
                format!(
                    " {out_added} {meta_str:>width$} ",
                    width = length - out_added.grapheme_len() - 3
                )
            } else {
                // just print duration
                let out_added = out.substr(0, length - meta_dur.chars().count() - 3);
                format!(
                    " {out_added} {meta_dur:>width$} ",
                    width = length - out_added.grapheme_len() - 3
                )
            }
        } else if length > EPISODE_DURATION_LENGTH {
            let dur = self.format_duration();
            let meta_dur = format!("[{dur}]");
            let out_added = out.substr(0, length - meta_dur.chars().count() - 3);
            format!(
                " {out_added} {meta_dur:>width$} ",
                width = length - out_added.grapheme_len() - 3
            )
        } else {
            format!(" {} ", out.substr(0, length - 2))
        }
    }

    fn is_played(&self) -> bool {
        self.played
    }
}

/// Struct holding data about an individual podcast feed, before it has
/// been inserted into the database. This includes a
/// (possibly empty) vector of episodes.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct PodcastNoId {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub explicit: Option<bool>,
    pub last_checked: DateTime<Utc>,
    pub episodes: Vec<EpisodeNoId>,
}

/// Struct holding data about an individual podcast episode, before it
/// has been inserted into the database.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpisodeNoId {
    pub title: String,
    pub url: String,
    pub guid: String,
    pub description: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

/// Struct holding data about an individual podcast episode, specifically
/// for the popup window that asks users which new episodes they wish to
/// download.
#[derive(Debug, Clone)]
pub struct NewEpisode {
    pub id: i64,
    pub pod_id: i64,
    pub title: String,
    pub pod_title: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct PodcastFeed {
    pub id: Option<i64>,
    pub url: String,
    pub title: Option<String>,
}

impl PodcastFeed {
    pub fn new(id: Option<i64>, url: &str, title: Option<String>) -> Self {
        Self {
            id,
            url: url.to_string(),
            title,
        }
    }
}

/// Spawns a new thread to check a feed and retrieve podcast data.
pub fn check_feed(
    feed: PodcastFeed,
    max_retries: usize,
    threadpool: &Threadpool,
    tx_to_main: Sender<Msg>,
) {
    threadpool.execute(move || {
        tx_to_main
            .send(Msg::Podcast(PCMsg::FetchPodcastStart(feed.url.clone())))
            .expect("thread messaging error in fetch start");
        match get_feed_data(&feed.url, max_retries) {
            Ok(pod) => match feed.id {
                Some(id) => {
                    tx_to_main
                        .send(Msg::Podcast(PCMsg::SyncData((id, pod))))
                        .expect("Thread messaging error when sync old");
                }
                None => tx_to_main
                    .send(Msg::Podcast(PCMsg::NewData(pod)))
                    .expect("Thread messaging error when add new"),
            },
            Err(_err) => tx_to_main
                .send(Msg::Podcast(PCMsg::Error(feed.url.to_string(), feed)))
                .expect("Thread messaging error when get feed"),
        }
    });
}

/// Given a URL, this attempts to pull the data about a podcast and its
/// episodes from an RSS feed.
fn get_feed_data(url: &str, mut max_retries: usize) -> Result<PodcastNoId> {
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(20))
        .build();

    let request: Result<ureq::Response> = loop {
        let response = agent.get(url).call();
        if let Ok(resp) = response {
            break Ok(resp);
        }
        max_retries -= 1;
        if max_retries == 0 {
            break Err(anyhow!("No response from feed"));
        }
    };

    match request {
        Ok(resp) => {
            let mut reader = resp.into_reader();
            let mut resp_data = Vec::new();
            reader.read_to_end(&mut resp_data)?;

            let channel = Channel::read_from(&resp_data[..])?;
            Ok(parse_feed_data(channel, url))
        }
        Err(err) => Err(err),
    }
}

/// Given a Channel with the RSS feed data, this parses the data about a
/// podcast and its episodes and returns a Podcast. There are existing
/// specifications for podcast RSS feeds that a feed should adhere to, but
/// this does try to make some attempt to account for the possibility that
/// a feed might not be valid according to the spec.
fn parse_feed_data(channel: Channel, url: &str) -> PodcastNoId {
    let title = channel.title().to_string();
    let url = url.to_string();
    let description = Some(channel.description().to_string());
    let last_checked = Utc::now();

    let mut author = None;
    let mut explicit = None;
    if let Some(itunes) = channel.itunes_ext() {
        author = itunes.author().map(std::string::ToString::to_string);
        explicit = match itunes.explicit() {
            None => None,
            Some(s) => {
                let ss = s.to_lowercase();
                match &ss[..] {
                    "yes" | "explicit" | "true" => Some(true),
                    "no" | "clean" | "false" => Some(false),
                    _ => None,
                }
            }
        };
    }

    let mut episodes = Vec::new();
    let items = channel.into_items();
    if !items.is_empty() {
        for item in &items {
            episodes.push(parse_episode_data(item));
        }
    }

    PodcastNoId {
        title,
        url,
        description,
        author,
        explicit,
        last_checked,
        episodes,
    }
}

/// For an item (episode) in an RSS feed, this pulls data about the item
/// and converts it to an Episode. There are existing specifications for
/// podcast RSS feeds that a feed should adhere to, but this does try to
/// make some attempt to account for the possibility that a feed might
/// not be valid according to the spec.
fn parse_episode_data(item: &Item) -> EpisodeNoId {
    let title = match item.title() {
        Some(s) => s.to_string(),
        None => String::new(),
    };
    let url = match item.enclosure() {
        Some(enc) => enc.url().to_string(),
        None => String::new(),
    };
    let guid = match item.guid() {
        Some(guid) => guid.value().to_string(),
        None => String::new(),
    };
    let description = match item.description() {
        Some(dsc) => dsc.to_string(),
        None => String::new(),
    };
    let pubdate = match item.pub_date() {
        Some(pd) => match parse_from_rfc2822_with_fallback(pd) {
            Ok(date) => {
                // this is a bit ridiculous, but it seems like
                // you have to convert from a DateTime<FixedOffset>
                // to a NaiveDateTime, and then from there create
                // a DateTime<Utc>; see
                // https://github.com/chronotope/chrono/issues/169#issue-239433186
                Some(DateTime::from_utc(date.naive_utc(), Utc))
            }
            Err(_) => None,
        },
        None => None,
    };

    let mut duration = None;
    if let Some(itunes) = item.itunes_ext() {
        duration = duration_to_int(itunes.duration()).map(i64::from);
    }

    EpisodeNoId {
        title,
        url,
        guid,
        description,
        pubdate,
        duration,
    }
}

/// Given a string representing an episode duration, this attempts to
/// convert to an integer representing the duration in seconds. Covers
/// formats HH:MM:SS, MM:SS, and SS. If the duration cannot be converted
/// (covering numerous reasons), it will return None.
fn duration_to_int(duration: Option<&str>) -> Option<i32> {
    match duration {
        Some(dur) => {
            match RE_DURATION.captures(dur) {
                Some(cap) => {
                    /*
                     * Provided that the regex succeeds, we should have
                     * 4 capture groups (with 0th being the full match).
                     * Depending on the string format, however, some of
                     * these may return None. We first loop through the
                     * capture groups and push Some results to an array.
                     * This will fail on the first non-numeric value,
                     * so the duration is parsed only if all components
                     * of it were successfully converted to integers.
                     * Finally, we convert hours, minutes, and seconds
                     * into a total duration in seconds and return.
                     */

                    let mut times = [None; 3];
                    let mut counter = 0;
                    // cap[0] is always full match
                    for c in cap.iter().skip(1).flatten() {
                        if let Ok(intval) = regex_to_int(c) {
                            times[counter] = Some(intval);
                            counter += 1;
                        } else {
                            return None;
                        }
                    }

                    match counter {
                        // HH:MM:SS
                        3 => Some(
                            times[0].unwrap() * 60 * 60
                                + times[1].unwrap() * 60
                                + times[2].unwrap(),
                        ),
                        // MM:SS
                        2 => Some(times[0].unwrap() * 60 + times[1].unwrap()),
                        // SS
                        1 => times[0],
                        _ => None,
                    }
                }
                None => None,
            }
        }
        None => None,
    }
}

/// Helper function converting a match from a regex capture group into an
/// integer.
fn regex_to_int(re_match: Match<'_>) -> Result<i32, std::num::ParseIntError> {
    let mstr = re_match.as_str();
    mstr.parse::<i32>()
}

// Much of the threadpool implementation here was taken directly from
// the Rust Book: https://doc.rust-lang.org/book/ch20-02-multithreaded.html
// and https://doc.rust-lang.org/book/ch20-03-graceful-shutdown-and-cleanup.html

/// Manages a threadpool of a given size, sending jobs to workers as
/// necessary. Implements Drop trait to allow threads to complete
/// their current jobs before being stopped.
pub struct Threadpool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<JobMessage>,
}

impl Threadpool {
    /// Creates a new Threadpool of a given size.
    pub fn new(n_threads: usize) -> Threadpool {
        let (sender, receiver) = mpsc::channel();
        let receiver_lock = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(n_threads);

        for _ in 0..n_threads {
            workers.push(Worker::new(Arc::clone(&receiver_lock)));
        }

        Threadpool { workers, sender }
    }

    /// Adds a new job to the threadpool, passing closure to first
    /// available worker.
    pub fn execute<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(func);
        self.sender
            .send(JobMessage::NewJob(job))
            .expect("Thread messaging error");
    }
}

impl Drop for Threadpool {
    /// Upon going out of scope, Threadpool sends terminate message to
    /// all workers but allows them to complete current jobs.
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender
                .send(JobMessage::Terminate)
                .expect("Thread messaging error");
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                // joins to ensure threads finish job before stopping
                thread.join().expect("Error dropping threads");
            }
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Messages used by Threadpool to communicate with Workers.
enum JobMessage {
    NewJob(Job),
    Terminate,
}

/// Used by Threadpool to complete jobs. Each Worker manages a single
/// thread.
struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new Worker, which waits for Jobs to be passed by the
    /// Threadpool.
    fn new(receiver: Arc<Mutex<mpsc::Receiver<JobMessage>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .expect("Threadpool error")
                .recv()
                .expect("Thread messaging error");

            match message {
                JobMessage::NewJob(job) => job(),
                JobMessage::Terminate => break,
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}

/// Imports a list of podcasts from OPML format, either reading from a
/// file or from stdin. If the `replace` flag is set, this replaces all
/// existing data in the database.
pub fn import_from_opml(db_path: &Path, config: &Settings, filepath: &str) -> Result<()> {
    // read from file or from stdin
    let mut f =
        File::open(filepath).with_context(|| format!("Could not open OPML file: {filepath}"))?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read from OPML file: {filepath}"))?;
    let xml = contents;

    let mut podcast_list = import_opml_feeds(&xml).with_context(|| {
        "Could not properly parse OPML file -- file may be formatted improperly or corrupted."
    })?;

    if podcast_list.is_empty() {
        println!("No podcasts to import.");
        return Ok(());
    }

    let db_inst = db::Database::connect(db_path)?;

    // delete database if we are replacing the data
    // if args.is_present("replace") {
    //     db_inst
    //         .clear_db()
    //         .with_context(|| "Error clearing database")?;
    // } else {
    let old_podcasts = db_inst.get_podcasts()?;

    // if URL is already in database, remove it from import
    podcast_list.retain(|pod| {
        for op in &old_podcasts {
            if pod.url == op.url {
                return false;
            }
        }
        true
    });
    // }

    // check again, now that we may have removed feeds after looking at
    // the database
    if podcast_list.is_empty() {
        println!("No podcasts to import.");
        return Ok(());
    }

    println!("Importing {} podcasts...", podcast_list.len());

    let threadpool = Threadpool::new(config.podcast_simultanious_download);
    let (tx_to_main, rx_to_main) = mpsc::channel();

    for pod in &podcast_list {
        check_feed(
            pod.clone(),
            config.podcast_max_retries,
            &threadpool,
            tx_to_main.clone(),
        );
    }

    let mut msg_counter: usize = 0;
    let mut failure = false;
    while let Some(message) = rx_to_main.iter().next() {
        match message {
            Msg::Podcast(PCMsg::NewData(pod)) => {
                msg_counter += 1;
                let title = pod.title.clone();
                let db_result = db_inst.insert_podcast(&pod);
                match db_result {
                    Ok(_) => {
                        println!("Added {title}");
                    }
                    Err(_err) => {
                        failure = true;
                        eprintln!("Error adding {title}");
                    }
                }
            }

            Msg::Podcast(PCMsg::Error(_, feed)) => {
                msg_counter += 1;
                failure = true;
                if let Some(t) = feed.title {
                    eprintln!("Error retrieving RSS feed: {t}");
                } else {
                    eprintln!("Error retrieving RSS feed");
                }
            }

            Msg::Podcast((PCMsg::SyncData((_id, _pod)))) => {
                msg_counter += 1;
            }
            _ => {}
        }

        if msg_counter >= podcast_list.len() {
            break;
        }
    }

    if failure {
        return Err(anyhow!("Process finished with errors."));
    }
    println!("Import successful.");

    Ok(())
}

/// Exports all podcasts to OPML format, either printing to stdout or
/// exporting to a file.
pub fn export_to_opml(db_path: &Path, file: &str) -> Result<()> {
    let db_inst = Database::connect(db_path)?;
    let podcast_list = db_inst.get_podcasts()?;
    let opml = export_opml_feeds(&podcast_list);

    let xml = opml
        .to_string()
        .map_err(|err| anyhow!(err))
        .with_context(|| "Could not create OPML format")?;

    let mut dst =
        File::create(file).with_context(|| format!("Could not create output file: {file}"))?;
    dst.write_all(xml.as_bytes())
        .with_context(|| format!("Could not copy OPML data to output file: {file}"))?;
    Ok(())
}

/// Import a list of podcast feeds from an OPML file. Supports
/// v1.0, v1.1, and v2.0 OPML files.
fn import_opml_feeds(xml: &str) -> Result<Vec<PodcastFeed>> {
    match OPML::from_str(xml) {
        Err(err) => Err(anyhow!(err)),
        Ok(opml) => {
            let mut feeds = Vec::new();
            for pod in opml.body.outlines {
                if pod.xml_url.is_some() {
                    // match against title attribute first -- if this is
                    // not set or empty, then match against the text
                    // attribute; this must be set, but can be empty
                    let temp_title = pod.title.filter(|t| !t.is_empty());
                    let title = match temp_title {
                        Some(t) => Some(t),
                        None => {
                            if pod.text.is_empty() {
                                None
                            } else {
                                Some(pod.text)
                            }
                        }
                    };
                    feeds.push(PodcastFeed::new(None, &pod.xml_url.unwrap(), title));
                }
            }
            Ok(feeds)
        }
    }
}

/// Converts the current set of podcast feeds to the OPML format
fn export_opml_feeds(podcasts: &[Podcast]) -> OPML {
    let date = Utc::now();
    let mut opml = OPML {
        head: Some(Head {
            title: Some("Shellcaster Podcast Feeds".to_string()),
            date_created: Some(date.to_rfc2822()),
            ..Head::default()
        }),
        ..Default::default()
    };

    let mut outlines = Vec::new();

    for pod in podcasts {
        // opml.add_feed(&pod.title, &pod.url);
        outlines.push(Outline {
            text: pod.title.clone(),
            r#type: Some("rss".to_string()),
            xml_url: Some(pod.url.clone()),
            title: Some(pod.title.clone()),
            ..Outline::default()
        });
    }

    opml.body = Body { outlines };
    opml
}

/// Enum used to communicate relevant data to the threadpool.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpData {
    pub id: i64,
    pub pod_id: i64,
    pub title: String,
    pub url: String,
    pub pubdate: Option<DateTime<Utc>>,
    pub file_path: Option<PathBuf>,
}

/// This is the function the main controller uses to indicate new
/// files to download. It uses the threadpool to start jobs
/// for every episode to be downloaded. New jobs can be requested
/// by the user while there are still ongoing jobs.
pub fn download_list(
    episodes: Vec<EpData>,
    dest: &Path,
    max_retries: usize,
    threadpool: &Threadpool,
    tx_to_main: &Sender<Msg>,
) {
    // parse episode details and push to queue
    for ep in episodes {
        let tx = tx_to_main.clone();
        let dest2 = dest.to_path_buf();
        threadpool.execute(move || {
            tx.send(Msg::Podcast(PCMsg::DLStart(ep.clone())))
                .expect("Thread messaging error when start download");
            let result = download_file(ep, dest2, max_retries);
            tx.send(Msg::Podcast(result))
                .expect("Thread messaging error");
        });
    }
}

/// Downloads a file to a local filepath, returning `DownloadMsg` variant
/// indicating success or failure.
#[allow(clippy::single_match_else)]
fn download_file(mut ep_data: EpData, destination_path: PathBuf, mut max_retries: usize) -> PCMsg {
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(120))
        .build();

    let request: Result<ureq::Response, ()> = loop {
        let response = agent.get(&ep_data.url).call();
        match response {
            Ok(resp) => break Ok(resp),
            Err(_) => {
                max_retries -= 1;
                if max_retries == 0 {
                    break Err(());
                }
            }
        }
    };

    if request.is_err() {
        return PCMsg::DLResponseError(ep_data);
    };

    let response = request.unwrap();

    // figure out the file type
    let ext = match response.header("content-type") {
        Some("audio/x-m4a") => "m4a",
        // Some("audio/mpeg") => "mp3",
        Some("video/quicktime") => "mov",
        Some("video/mp4") => "mp4",
        Some("video/x-m4v") => "m4v",
        _ => "mp3", // assume .mp3 unless we figure out otherwise
    };

    let mut file_name = sanitize_with_options(
        &ep_data.title,
        Options {
            truncate: true,
            windows: true, // for simplicity, we'll just use Windows-friendly paths for everyone
            replacement: "",
        },
    );

    if let Some(pubdate) = ep_data.pubdate {
        file_name = format!("{file_name}_{}", pubdate.format("%Y%m%d_%H%M%S"));
    }

    let mut file_path = destination_path;
    file_path.push(format!("{file_name}.{ext}"));

    let dst = File::create(&file_path);
    if dst.is_err() {
        return PCMsg::DLFileCreateError(ep_data);
    };

    ep_data.file_path = Some(file_path);

    let mut reader = response.into_reader();
    match std::io::copy(&mut reader, &mut dst.unwrap()) {
        Ok(_) => PCMsg::DLComplete(ep_data),
        Err(_) => PCMsg::DLFileWriteError(ep_data),
    }
}
