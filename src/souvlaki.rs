use crate::ui::activity::main::{Status, TermusicActivity};
use dbus::arg::{RefArg, Variant};
use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged as Ppc;
use dbus::message::SignalArgs;
use dbus::strings::Path as DbusPath;
use dbus::Error as DbusError;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use std::collections::HashMap;
use std::convert::From;
use std::convert::TryInto;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// A platform-specific error.
#[derive(Debug)]
pub struct Error;

#[derive(Clone, Copy, Debug)]
pub struct PlatformConfig<'a> {
    /// The name to be displayed to the user. (*Required on Linux*)
    pub display_name: &'a str,
    /// Should follow [the D-Bus spec](https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-names-bus). (*Required on Linux*)
    pub dbus_name: &'a str,
}
/// A handle to OS media controls.
pub struct MediaControls {
    shared_data: Arc<Mutex<MprisData>>,
    thread: Option<DbusThread>,
}

struct DbusThread {
    kill_signal: mpsc::Sender<()>,
    thread: JoinHandle<()>,
    update_signal: mpsc::Sender<()>,
}
/// The status of media playback.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaPlayback {
    Stopped,
    Paused { progress: Option<MediaPosition> },
    Playing { progress: Option<MediaPosition> },
}

/// The metadata of a media item.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MediaMetadata<'a> {
    pub title: Option<&'a str>,
    pub album: Option<&'a str>,
    pub artist: Option<&'a str>,
    pub cover_url: Option<&'a str>,
    pub duration: Option<Duration>,
}

/// Events sent by the OS media controls.
#[derive(Clone, PartialEq, Eq, Debug)]
#[allow(unused)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Stop,

    /// Seek forward or backward by an undetermined amount.
    Seek(SeekDirection),
    /// Seek forward or backward by a certain amount.
    SeekBy(SeekDirection, Duration),
    /// Set the position/progress of the currently playing media item.
    SetPosition(MediaPosition),
    /// Open the URI in the media player.
    OpenUri(String),

    /// Bring the media player's user interface to the front using any appropriate mechanism available.
    Raise,
    /// Shut down the media player.
    Quit,
}

/// An instant in a media item.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MediaPosition(pub Duration);

/// The direction to seek in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SeekDirection {
    Forward,
    Backward,
}

impl Drop for MediaControls {
    fn drop(&mut self) {
        // Ignores errors if there are any.
        self.detach().ok();
    }
}
struct MprisData {
    dbus_name: String,
    friendly_name: String,
    metadata: OwnedMetadata,
    playback_status: MediaPlayback,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct OwnedMetadata {
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover_url: Option<String>,
    pub duration: Option<i64>,
}

impl From<MediaMetadata<'_>> for OwnedMetadata {
    fn from(other: MediaMetadata) -> Self {
        Self {
            title: other.title.map(std::string::ToString::to_string),
            artist: other.artist.map(std::string::ToString::to_string),
            album: other.album.map(std::string::ToString::to_string),
            cover_url: other.cover_url.map(std::string::ToString::to_string),
            duration: other.duration.map(|d| d.as_micros().try_into().unwrap()),
        }
    }
}

impl MediaControls {
    /// Create media controls with the specified config.
    pub fn new(config: PlatformConfig) -> Self {
        let PlatformConfig {
            dbus_name,
            display_name,
            ..
        } = config;

        let shared_data = Arc::new(Mutex::new(MprisData {
            dbus_name: dbus_name.to_string(),
            friendly_name: display_name.to_string(),
            metadata: OwnedMetadata::default(),
            playback_status: MediaPlayback::Stopped,
        }));

        Self {
            shared_data,
            thread: None,
        }
    }

    /// Attach the media control events to a handler.
    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.detach()?;

        let shared_data = self.shared_data.clone();
        let event_handler = Arc::new(Mutex::new(event_handler));
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        self.thread = Some(DbusThread {
            kill_signal: tx,
            thread: thread::spawn(move || {
                mpris_run(event_handler, &shared_data, &rx, &rx2).unwrap();
            }),
            update_signal: tx2,
        });
        Ok(())
    }

    /// Detach the event handler.
    pub fn detach(&mut self) -> Result<(), Error> {
        if let Some(DbusThread {
            kill_signal,
            thread,
            update_signal: _,
        }) = self.thread.take()
        {
            kill_signal.send(()).map_err(|_| Error)?;
            thread.join().unwrap();
        }
        Ok(())
    }

    /// Set the current playback status.
    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        let mut data = self.shared_data.lock().map_err(|_e| Error)?;
        data.playback_status = playback;
        self.update();
        Ok(())
    }

    /// Set the metadata of the currently playing media item.
    pub fn set_metadata(&mut self, metadata: MediaMetadata) {
        if let Ok(mut data) = self.shared_data.lock() {
            data.metadata = metadata.into();
            // self.update();
        }
    }
    pub fn update(&self) {
        if let Some(DbusThread {
            kill_signal: _,
            thread: _,
            update_signal,
        }) = &self.thread
        {
            update_signal.send(()).ok();
        }
    }
}

// TODO: better errors
#[allow(clippy::too_many_lines)]
fn mpris_run(
    event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
    shared_data: &Arc<Mutex<MprisData>>,
    kill_signal: &mpsc::Receiver<()>,
    update_signal: &mpsc::Receiver<()>,
) -> Result<(), DbusError> {
    let (dbus_name, friendly_name) = {
        let data = shared_data.lock().unwrap();
        (
            format!("org.mpris.MediaPlayer2.{}", data.dbus_name),
            data.friendly_name.clone(),
        )
    };

    let c = Connection::new_session()?;
    c.request_name(dbus_name, false, true, false)?;

    let mut cr = Crossroads::new();

    let media_player_2 = cr.register("org.mpris.MediaPlayer2", {
        let event_handler = event_handler.clone();

        move |b| {
            b.property("Identity")
                .get(move |_, _| Ok(friendly_name.clone()));

            register_method(b, &event_handler, "Raise", MediaControlEvent::Raise);
            register_method(b, &event_handler, "Quit", MediaControlEvent::Quit);

            // TODO: allow user to set these properties
            b.property("CanQuit").get(|_, _| Ok(true));
            b.property("CanRaise").get(|_, _| Ok(true));
            b.property("HasTracklist").get(|_, _| Ok(false));
            b.property("SupportedUriSchemes")
                .get(move |_, _| Ok(&[] as &[String]));
            b.property("SupportedMimeTypes")
                .get(move |_, _| Ok(&[] as &[String]));
        }
    });

    let shared_data1 = shared_data.clone();
    let player = cr.register("org.mpris.MediaPlayer2.Player", move |b| {
        // TODO: allow user to set these properties
        b.property("CanControl").get(|_, _| Ok(true));
        b.property("CanPlay").get(|_, _| Ok(true));
        b.property("CanPause").get(|_, _| Ok(true));
        b.property("CanGoNext").get(|_, _| Ok(true));
        b.property("CanGoPrevious").get(|_, _| Ok(true));
        b.property("CanSeek").get(|_, _| Ok(true));

        b.property("PlaybackStatus").get({
            let shared_data = shared_data1.clone();
            move |_, _| {
                let data = shared_data.lock().unwrap();
                let status = match data.playback_status {
                    MediaPlayback::Playing { .. } => "Playing",
                    MediaPlayback::Paused { .. } => "Paused",
                    MediaPlayback::Stopped => "Stopped",
                };
                Ok(status.to_string())
            }
        });

        b.property("Position").get({
            let shared_data = shared_data1.clone();
            move |_, _| {
                let data = shared_data.lock().unwrap();
                let progress: i64 = match data.playback_status {
                    MediaPlayback::Playing {
                        progress: Some(progress),
                    }
                    | MediaPlayback::Paused {
                        progress: Some(progress),
                    } => progress.0.as_micros(),
                    _ => 0,
                }
                .try_into()
                .unwrap();
                Ok(progress)
            }
        });

        b.property("Metadata").get({
            let shared_data = shared_data1.clone();

            move |_, _| {
                // TODO: this could be stored in a cache in `shared_data`.
                let mut dict = HashMap::<String, Variant<Box<dyn RefArg>>>::new();

                let data = shared_data.lock().unwrap();
                let mut insert = |k: &str, v| dict.insert(k.to_string(), Variant(v));

                let OwnedMetadata {
                    ref title,
                    ref album,
                    ref artist,
                    ref cover_url,
                    ref duration,
                } = data.metadata;

                // TODO: For some reason the properties don't follow the order when
                // queried from the D-Bus. Probably because of the use of HashMap.
                // Can't use `dbus::arg::Dict` though, because it isn't Send.

                // MPRIS

                // TODO: this is just a workaround to enable SetPosition.
                insert("mpris:trackid", Box::new(DbusPath::new("/").unwrap()));

                if let Some(length) = duration {
                    insert("mpris:length", Box::new(*length));
                }

                if let Some(cover_url) = cover_url {
                    insert("mpris:artUrl", Box::new(cover_url.clone()));
                }

                // Xesam
                if let Some(title) = title {
                    insert("xesam:title", Box::new(title.clone()));
                }
                if let Some(artist) = artist {
                    insert("xesam:albumArtist", Box::new(artist.clone()));
                }
                if let Some(album) = album {
                    insert("xesam:album", Box::new(album.clone()));
                }

                Ok(dict)
            }
        });

        b.signal::<(String, dbus::arg::PropMap, Vec<String>), _>(
            "PropertiesChanged",
            ("org.freedesktop.DBus.Properties", "Metadata", "Metadata"),
        );
        // b.signal::<(String, dbus::arg::PropMap, Vec<String>), _>(
        //     "PropertiesChanged",
        //     (
        //         "org.freedesktop.DBus.Properties",
        //         "PlaybackStatus",
        //         "PlaybackStatus",
        //     ),
        // );

        register_method(b, &event_handler, "Play", MediaControlEvent::Play);
        register_method(b, &event_handler, "Pause", MediaControlEvent::Pause);
        register_method(b, &event_handler, "PlayPause", MediaControlEvent::Toggle);
        register_method(b, &event_handler, "Next", MediaControlEvent::Next);
        register_method(b, &event_handler, "Previous", MediaControlEvent::Previous);
        register_method(b, &event_handler, "Stop", MediaControlEvent::Stop);

        b.method("Seek", ("Offset",), (), {
            let event_handler = event_handler.clone();

            move |_, _, (offset,): (i64,)| {
                let abs_offset = offset.abs() as u64;
                let direction = if offset > 0 {
                    SeekDirection::Forward
                } else {
                    SeekDirection::Backward
                };

                (event_handler.lock().unwrap())(MediaControlEvent::SeekBy(
                    direction,
                    Duration::from_micros(abs_offset),
                ));
                Ok(())
            }
        });

        b.method("SetPosition", ("TrackId", "Position"), (), {
            let event_handler = event_handler.clone();
            let shared_data = shared_data1.clone();

            move |_, _, (_trackid, position): (DbusPath, i64)| {
                let data = shared_data.lock().unwrap();
                // According to the MPRIS specification:

                // 1.
                // If the TrackId argument is not the same as the current
                // trackid, the call is ignored as stale. So here we check that.
                // (Maybe it should be optional?)

                // TODO: the check. (We first need to store the TrackId somewhere)

                // 2.
                // If the Position argument is less than 0, do nothing.
                // If the Position argument is greater than the track length, do nothing.

                if position < 0 {
                    return Ok(());
                }

                if let Some(duration) = data.metadata.duration {
                    if position > duration {
                        return Ok(());
                    }
                }

                let position: u64 = position.try_into().unwrap();

                (event_handler.lock().unwrap())(MediaControlEvent::SetPosition(MediaPosition(
                    Duration::from_micros(position),
                )));
                Ok(())
            }
        });

        b.method("OpenUri", ("Uri",), (), {
            move |_, _, (uri,): (String,)| {
                (event_handler.lock().unwrap())(MediaControlEvent::OpenUri(uri));
                Ok(())
            }
        });
    });

    cr.insert("/org/mpris/MediaPlayer2", &[media_player_2, player], ());

    c.start_receive(
        dbus::message::MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn).unwrap();
            true
        }),
    );

    // Below lines are left for debug. Listen to propertieschanged signal
    // let mr = Ppc::match_rule(Some(&"org.mpris.MediaPlayer2.Player".into()), None).static_clone();
    // c.add_match(mr, |ppc: Ppc, _, _msg| {
    //     println!("{:?}", ppc);
    //     true
    // })?;

    // Start the server loop.
    loop {
        // If the kill signal was sent, then break the loop.
        if kill_signal.recv_timeout(Duration::from_millis(10)).is_ok() {
            break;
        }

        // Do the event processing.
        c.process(Duration::from_millis(1000))?;

        // send propertieschanged signal when received update signal
        if let Ok(()) = update_signal.try_recv() {
            let mut changed = Ppc {
                interface_name: "org.mpris.MediaPlayer2.Player".to_string(),
                ..Ppc::default()
            };

            let data = shared_data.lock().unwrap();
            let metadata = data.metadata.clone();
            changed.changed_properties.insert(
                "Metadata".to_string(),
                Variant(Box::new(get_metadata(metadata))),
            );
            let status = match data.playback_status {
                MediaPlayback::Playing { .. } => "Playing",
                MediaPlayback::Paused { .. } => "Paused",
                MediaPlayback::Stopped => "Stopped",
            };
            changed.changed_properties.insert(
                "PlaybackStatus".to_string(),
                Variant(Box::new(status.to_string())),
            );

            c.channel()
                .send(changed.to_emit_message(
                    &DbusPath::new("/org/mpris/MediaPlayer2".to_string()).unwrap(),
                ))
                .unwrap();
        }
        // thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

fn register_method(
    b: &mut IfaceBuilder<()>,
    event_handler: &Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
    name: &'static str,
    event: MediaControlEvent,
) {
    let event_handler = event_handler.clone();

    b.method(name, (), (), move |_, _, _: ()| {
        (event_handler.lock().unwrap())(event.clone());
        Ok(())
    });
}

type Metadata = HashMap<String, Variant<Box<dyn RefArg>>>;
fn get_metadata(song: OwnedMetadata) -> Metadata {
    let mut hm: Metadata = HashMap::new();

    hm.insert(
        "mpris:trackid".to_string(),
        Variant(Box::new(DbusPath::from("/org/termusic/123"))),
    );

    hm.insert(
        "xesam:album".to_string(),
        Variant(Box::new(song.album.unwrap_or_else(|| "".to_string()))),
    );
    hm.insert(
        "xesam:artist".to_string(),
        Variant(Box::new(
            song.artist.unwrap_or_else(|| "Unknown Artist".to_string()),
        )),
    );
    hm.insert(
        "xesam:title".to_string(),
        Variant(Box::new(
            song.title.unwrap_or_else(|| "Unknown Title".to_string()),
        )),
    );
    hm
}

pub fn mpris_handler(e: MediaControlEvent, activity: &mut TermusicActivity) {
    match e {
        MediaControlEvent::Next => {
            activity.next_song();
        }
        MediaControlEvent::Previous => {
            activity.previous_song();
        }
        MediaControlEvent::Pause => {
            activity.player.pause();
        }
        MediaControlEvent::Toggle => {
            if activity.player.is_paused() {
                activity.status = Some(Status::Running);
                activity.player.resume();
            } else {
                activity.status = Some(Status::Paused);
                activity.player.pause();
            }
        }
        MediaControlEvent::Play => {
            activity.player.resume();
        }
        // MediaControlEvent::Seek(x) => match x {
        //     SeekDirection::Forward => activity.player.seek(5).ok(),
        //     SeekDirection::Backward => activity.player.seek(-5).ok(),
        // },
        // MediaControlEvent::SetPosition(position) => {
        //     let _position = position. / 1000;
        // }
        MediaControlEvent::OpenUri(uri) => {
            activity.player.add_and_play(&uri);
        }
        _ => {}
    }
}
