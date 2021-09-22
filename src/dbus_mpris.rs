// dbus-send example
// according to https://specifications.freedesktop.org/mpris-spec/latest/Player_Interface.html
// for get Properties
// dbus-send --session --print-reply --dest=org.mpris.MediaPlayer2.ncmt /org/mpris/MediaPlayer2 org.freedesktop.DBus.Properties.Get string:"org.mpris.MediaPlayer2.Player" string:"Rate"
// for method
// dbus-send --session --print-reply --dest=org.mpris.MediaPlayer2.ncmt /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.Next
use crate::ui::activity::main::TermusicActivity;
use dbus::{
    arg::{messageitem::MessageItem, RefArg, Variant},
    blocking::LocalConnection,
    message::SignalArgs,
    strings::Path,
};
use dbus_tree::{Access, Factory};
// use dbus_crossroads::{Context,Crossroads};
use crate::ui::activity::Status;
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use log::info;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::{collections::HashMap, time::Duration};

type Metadata = HashMap<String, Variant<Box<dyn RefArg>>>;
#[allow(unused)]
pub enum PlayerCommand {
    Play,
    Pause,
    Stop,
    PlayPause,
    Seek(i32),
    Next,
    Previous,
    Load(String),
    Position(i32, u64),
    Metadata(MetaInfo, Sender<String>),
}

#[derive(Clone, PartialEq, Debug)]
#[allow(unused)]
pub enum RepeatState {
    Off,
    Track,
    All,
    Shuffle,
}

#[derive(Clone, PartialEq, Debug)]
#[allow(unused)]
pub enum TrackState {
    Forword,
    Backword,
}

#[allow(unused)]
pub enum MetaInfo {
    Volume,
    Shuffle,
    Position,
    LoopStatus,
    Status,
    Info,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongMpris {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

pub struct DbusMpris {
    rx: Receiver<PlayerCommand>,
    tx_update: Sender<MprisState>,
}

struct MprisState(String, SongMpris);

impl DbusMpris {
    pub fn new() -> Self {
        Self::init()
    }

    pub fn init() -> Self {
        let (tx, rx) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel::<MprisState>();
        info!("start mpris thread");
        let _server_handle = {
            thread::spawn(move || {
                match dbus_mpris_server(tx, rx2) {
                    Ok(()) => {}
                    Err(e) => println!("Error in dbus server: {}", e),
                };
            })
        };
        info!("finish mpris thread");
        Self { rx, tx_update: tx2 }
    }

    pub fn next(&self) -> Result<PlayerCommand, mpsc::TryRecvError> {
        self.rx.try_recv()
    }

    pub fn update(&self, song: crate::song::Song, status: crate::ui::activity::Status) {
        // let status = get_playbackstatus(activity);
        let s = SongMpris {
            album: Some(song.album().unwrap_or("").to_string()),
            artist: Some(song.artist().unwrap_or("Unknown Artist").to_string()),
            title: Some(song.title().unwrap_or("Unknown title").to_string()),
        };
        let status = get_playbackstatus(status);
        self.tx_update.send(MprisState(status, s)).unwrap();
    }
}

// #[cfg(not(feature = "dbus_mpris"))]
// #[allow(unused)]
// pub fn dbus_mpris_server(tx: Sender<PlayerCommand>) -> Result<(), Box<dyn Error>> {
//     Ok(())
// }

#[allow(clippy::too_many_lines)]
fn dbus_mpris_server(
    tx: Sender<PlayerCommand>,
    rx: Receiver<MprisState>,
) -> Result<(), Box<dyn Error>> {
    // Let's start by starting up a connection to the session bus and request a name.
    let c = LocalConnection::new_session()?;
    c.request_name("org.mpris.MediaPlayer2.termusic", false, true, false)?;

    // The choice of factory tells us what type of tree we want,
    // and if we want any extra data inside. We pick the simplest variant.
    let f = Factory::new_fnmut::<()>();
    let tx = Arc::new(tx);

    let method_raise = f.method("Raise", (), move |m| {
        let mret = m.msg.method_return();
        Ok(vec![mret])
    });

    let method_quit = {
        // let local_tx = tx.clone();
        f.method("Quit", (), move |m| {
            // local_spirc.shutdown();
            let mret = m.msg.method_return();
            Ok(vec![mret])
        })
    };

    let property_identity = f
        .property::<String, _>("Identity", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append("termusic".to_string());
            Ok(())
        });

    let property_supported_uri_schemes = f
        .property::<Vec<String>, _>("SupportedUriSchemes", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(vec!["http".to_string()]);
            Ok(())
        });

    let property_mimetypes = f
        .property::<Vec<String>, _>("SupportedMimeTypes", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(Vec::<String>::new());
            Ok(())
        });

    let property_can_quit = f
        .property::<bool, _>("CanQuit", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(true);
            Ok(())
        });

    let property_can_raise = f
        .property::<bool, _>("CanRaise", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(false);
            Ok(())
        });

    let property_can_fullscreen = f
        .property::<bool, _>("CanSetFullscreen", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(false);
            Ok(())
        });

    let property_has_tracklist = f
        .property::<bool, _>("HasTrackList", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(false);
            Ok(())
        });

    // player method
    let method_next = {
        let local_tx = tx.clone();
        f.method("Next", (), move |m| {
            local_tx.send(PlayerCommand::Next).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_previous = {
        let local_tx = tx.clone();
        f.method("Previous", (), move |m| {
            local_tx.send(PlayerCommand::Previous).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_pause = {
        let local_tx = tx.clone();
        f.method("Pause", (), move |m| {
            local_tx.send(PlayerCommand::Pause).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_play_pause = {
        let local_tx = tx.clone();
        f.method("PlayPause", (), move |m| {
            local_tx.send(PlayerCommand::PlayPause).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_stop = {
        let local_tx = tx.clone();
        f.method("Stop", (), move |m| {
            local_tx.send(PlayerCommand::Stop).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_play = {
        let local_tx = tx.clone();
        f.method("Play", (), move |m| {
            local_tx.send(PlayerCommand::Play).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_seek = {
        let local_tx = tx.clone();
        f.method("Seek", (), move |m| {
            // I change the Time in microseconds to the seconds.
            let offset = m.msg.read1()?;
            local_tx.send(PlayerCommand::Seek(offset)).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_set_position = {
        let local_tx = tx.clone();
        f.method("SetPosition", (), move |m| {
            let (track_id, position) = m.msg.read2()?;
            local_tx
                .send(PlayerCommand::Position(track_id, position))
                .unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let method_open_uri = {
        let local_tx = tx.clone();
        f.method("OpenUri", (), move |m| {
            let uri = m.msg.read1()?;
            local_tx.send(PlayerCommand::Load(uri)).unwrap();
            Ok(vec![m.msg.method_return()])
        })
    };

    let property_rate = f
        .property::<f64, _>("Rate", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(1.0);
            Ok(())
        });

    let property_max_rate = f
        .property::<f64, _>("MaximumRate", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(1.0);
            Ok(())
        });

    let property_min_rate = f
        .property::<f64, _>("MinimumRate", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(1.0);
            Ok(())
        });

    let property_can_play = f
        .property::<bool, _>("CanPlay", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(true);
            Ok(())
        });

    let property_can_pause = f
        .property::<bool, _>("CanPause", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(true);
            Ok(())
        });

    let property_can_seek = f
        .property::<bool, _>("CanSeek", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(true);
            Ok(())
        });

    let property_can_control = f
        .property::<bool, _>("CanControl", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(true);
            Ok(())
        });

    let property_can_go_previous = f
        .property::<bool, _>("CanGoPrevious", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(true);
            Ok(())
        });

    let property_can_go_next = f
        .property::<bool, _>("CanGoNext", ())
        .access(Access::Read)
        .on_get(|iter, _| {
            iter.append(true);
            Ok(())
        });

    let property_loop_status = {
        let local_tx = tx.clone();
        f.property::<String, _>("LoopStatus", ())
            .access(Access::Read)
            .on_get(move |iter, _| {
                // listen channel response
                let (mtx, mrx) = mpsc::channel();
                local_tx
                    .send(PlayerCommand::Metadata(MetaInfo::LoopStatus, mtx))
                    .unwrap();
                let res = mrx.recv();
                match res {
                    Ok(r) => {
                        iter.append(r);
                    }
                    Err(_) => {
                        iter.append("error".to_owned());
                    }
                }
                Ok(())
            })
    };

    let property_playback_status = {
        let local_tx = tx.clone();
        f.property::<String, _>("PlaybackStatus", ())
            .access(Access::Read)
            .on_get(move |iter, _| {
                // listen channel response
                let (mtx, mrx) = mpsc::channel();
                local_tx
                    .send(PlayerCommand::Metadata(MetaInfo::Status, mtx))
                    .unwrap();
                let res = mrx.recv();
                match res {
                    Ok(r) => {
                        iter.append(r);
                    }
                    Err(_) => {
                        iter.append("error".to_owned());
                    }
                }
                Ok(())
            })
    };

    let property_shuffle = {
        let local_tx = tx.clone();
        f.property::<bool, _>("Shuffle", ())
            .access(Access::Read)
            .on_get(move |iter, _| {
                // listen channel response
                let (mtx, mrx) = mpsc::channel();
                local_tx
                    .send(PlayerCommand::Metadata(MetaInfo::Shuffle, mtx))
                    .unwrap();
                let res = mrx.recv();
                match res {
                    Ok(r) => {
                        let rr = match r.as_ref() {
                            "true" => true,
                            &_ => false,
                        };
                        iter.append(rr);
                    }
                    Err(_) => {
                        iter.append("error".to_owned());
                    }
                }
                Ok(())
            })
    };

    let property_position = {
        let local_tx = tx.clone();
        f.property::<i64, _>("Position", ())
            .access(Access::Read)
            .on_get(move |iter, _| {
                // listen channel response
                let (mtx, mrx) = mpsc::channel();
                local_tx
                    .send(PlayerCommand::Metadata(MetaInfo::Position, mtx))
                    .unwrap();
                let res = mrx.recv();
                match res {
                    Ok(r) => {
                        let rr = r.parse::<i64>().unwrap_or(0) * 1000;
                        iter.append(rr);
                    }
                    Err(_) => {
                        iter.append("error".to_owned());
                    }
                }
                Ok(())
            })
    };

    let property_metadata = {
        let local_tx = tx;
        f.property::<HashMap<String, Variant<Box<dyn RefArg>>>, _>("Metadata", ())
            .access(Access::Read)
            .on_get(move |iter, _| {
                // listen channel response
                let (mtx, mrx) = mpsc::channel();
                local_tx
                    .send(PlayerCommand::Metadata(MetaInfo::Info, mtx))
                    .unwrap();
                let res = mrx.recv();
                match res {
                    Ok(r) => {
                        let mut m = HashMap::new();
                        if let Ok(current_playing) = serde_json::from_str::<SongMpris>(&r) {
                            m.insert(
                                "mpris:trackid".to_string(),
                                Variant(
                                    Box::new(MessageItem::Int64(1.to_owned())) as Box<dyn RefArg>
                                ),
                            );
                            m.insert(
                                "mpris:length".to_string(),
                                Variant(Box::new(MessageItem::Int64(i64::from(100) * 1000))
                                    as Box<dyn RefArg>),
                            );
                            m.insert(
                                "xesam:title".to_string(),
                                Variant(Box::new(MessageItem::Str(current_playing.title.unwrap()))
                                    as Box<dyn RefArg>),
                            );
                            m.insert(
                                "xesam:album".to_string(),
                                Variant(Box::new(MessageItem::Str(current_playing.album.unwrap()))
                                    as Box<dyn RefArg>),
                            );
                            m.insert(
                                "xesam:artist".to_string(),
                                Variant(
                                    Box::new(MessageItem::Str(current_playing.artist.unwrap()))
                                        as Box<dyn RefArg>,
                                ),
                            );
                        }
                        iter.append(m);
                    }
                    Err(_) => {
                        iter.append("error".to_owned());
                    }
                }
                Ok(())
            })
    };

    // We create a tree with one object path inside and make that path introspectable.
    let tree = f
        .tree(())
        .add(
            f.object_path("/org/mpris/MediaPlayer2", ())
                .introspectable()
                .add(
                    f.interface("org.mpris.MediaPlayer2", ())
                        .add_m(method_raise)
                        .add_m(method_quit)
                        .add_p(property_can_quit)
                        .add_p(property_can_raise)
                        .add_p(property_can_fullscreen)
                        .add_p(property_has_tracklist)
                        .add_p(property_identity)
                        .add_p(property_supported_uri_schemes)
                        .add_p(property_mimetypes),
                )
                .add(
                    f.interface("org.mpris.MediaPlayer2.Player", ())
                        .add_m(method_next)
                        .add_m(method_previous)
                        .add_m(method_pause)
                        .add_m(method_play_pause)
                        .add_m(method_stop)
                        .add_m(method_play)
                        .add_m(method_seek)
                        .add_m(method_set_position)
                        .add_m(method_open_uri)
                        .add_p(property_rate)
                        .add_p(property_max_rate)
                        .add_p(property_min_rate)
                        .add_p(property_can_play)
                        .add_p(property_can_pause)
                        .add_p(property_can_seek)
                        .add_p(property_can_control)
                        .add_p(property_can_go_next)
                        .add_p(property_can_go_previous)
                        .add_p(property_loop_status)
                        .add_p(property_shuffle)
                        .add_p(property_position)
                        .add_p(property_metadata)
                        .add_p(property_playback_status),
                ),
        )
        .add(f.object_path("/", ()).introspectable());
    // ;

    // We add the tree to the connection so that incoming method calls will be handled.
    tree.start_receive(&c);
    // info!("start");

    // tree.set_registered(&c, true)
    //     .expect("failed to register tree");
    // c.add_handler(tree);

    // Ok(())
    // Serve clients forever.
    loop {
        c.process(Duration::from_millis(200))?;
        // if let Some(m) = c.incoming(200).next() {
        //     warn!("Unhandled dbus message: {:?}", m);
        // }

        // c.process(Duration::from_nanos(1))?;
        if let Ok(state) = rx.try_recv() {
            let mut changed: PropertiesPropertiesChanged = Default::default();
            // debug!(
            //     "mpris PropertiesChanged: status {}, track: {:?}",
            //     state.0, state.1
            // );

            changed.interface_name = "org.mpris.MediaPlayer2.Player".to_string();
            changed.changed_properties.insert(
                "Metadata".to_string(),
                Variant(Box::new(get_metadata(state.1))),
            );

            changed
                .changed_properties
                .insert("PlaybackStatus".to_string(), Variant(Box::new(state.0)));

            c.channel()
                .send(
                    changed.to_emit_message(
                        &Path::new("/org/mpris/MediaPlayer2".to_string()).unwrap(),
                    ),
                )
                .unwrap();
        }

        thread::sleep(Duration::from_millis(250));
    }
}

fn get_playbackstatus(status: crate::ui::activity::Status) -> String {
    match status {
        Status::Running => "Playing".to_owned(),
        Status::Paused => "Paused".to_owned(),
        Status::Stopped => "Stopped".to_owned(),
    }
}

fn get_metadata(song: SongMpris) -> Metadata {
    let mut hm: Metadata = HashMap::new();

    hm.insert(
        "mpris:trackid".to_string(),
        Variant(Box::new(Path::from(
            "/org/termusic/123", // playable
                                 //     .filter(|t| t.id().is_some())
                                 //     .map(|t| t.uri().replace(':', "/"))
                                 //     .unwrap_or_else(|| String::from("0"))
        ))),
    );
    // hm.insert(
    //     "mpris:length".to_string(),
    //     Variant(Box::new(
    //         song.duration().as_secs(),
    //         // .map(|t| t.duration() as i64 * 1_000)
    //         // .unwrap_or(0),
    //     )),
    // );
    // hm.insert(
    //     "mpris:artUrl".to_string(),
    //     Variant(Box::new(
    //         playable
    //             .map(|t| t.cover_url().unwrap_or_default())
    //             .unwrap_or_default(),
    //     )),
    // );

    hm.insert(
        "xesam:album".to_string(),
        Variant(Box::new(
                song.album.unwrap_or_else(|| "".to_string())
            // playable
            //     .and_then(|p| p.track())
            //     .map(|t| t.album.unwrap_or_default())
            //     .unwrap_or_default(),
        )),
    );
    // hm.insert(
    //     "xesam:albumArtist".to_string(),
    //     Variant(Box::new(
    //             song.
    //         // playable
    //         //     .and_then(|p| p.track())
    //         //     .map(|t| t.album_artists)
    //         //     .unwrap_or_default(),
    //     )),
    // );
    hm.insert(
        "xesam:artist".to_string(),
        Variant(Box::new(
            song.artist.unwrap_or_else(|| "Unknown Artist".to_string())
            // playable
            //     .and_then(|p| p.track())
            //     .map(|t| t.artists)
            //     .unwrap_or_default(),
        )),
    );
    // hm.insert(
    //     "xesam:discNumber".to_string(),
    //     Variant(Box::new(
    //         playable
    //             .and_then(|p| p.track())
    //             .map(|t| t.disc_number)
    //             .unwrap_or(0),
    //     )),
    // );
    hm.insert(
        "xesam:title".to_string(),
        Variant(Box::new(
                song.title.unwrap_or_else(||"Unknown Title".to_string())
            // playable
            //     .map(|t| match t {
            //         Playable::Track(t) => t.title.clone(),
            //         Playable::Episode(ep) => ep.name.clone(),
            //     })
            //     .unwrap_or_default(),
        )),
    );
    // hm.insert(
    //     "xesam:trackNumber".to_string(),
    //     Variant(Box::new(
    //         playable
    //             .and_then(|p| p.track())
    //             .map(|t| t.track_number)
    //             .unwrap_or(0) as i32,
    //     )),
    // );
    // hm.insert(
    //     "xesam:url".to_string(),
    //     Variant(Box::new(
    //         playable
    //             .map(|t| t.share_url().unwrap_or_default())
    //             .unwrap_or_default(),
    //     )),
    // );

    hm
}

pub fn mpris_handler(r: PlayerCommand, activity: &mut TermusicActivity) {
    match r {
        PlayerCommand::Next | PlayerCommand::Previous => {
            // app.skip_track(TrackState::Forword);
            activity.status = Some(Status::Stopped);
        }
        // PlayerCommand::Previous => {
        //     // app.skip_track(TrackState::Backword);
        //     activity.status = Some(Status::Stopped);
        // }
        PlayerCommand::Pause => {
            activity.player.pause();
        }
        PlayerCommand::PlayPause => {
            // app.player.pause();
            if activity.player.is_paused() {
                activity.status = Some(Status::Running);
                activity.player.resume();
            } else {
                activity.status = Some(Status::Paused);
                activity.player.pause();
            }
        }
        PlayerCommand::Stop => {
            // app.player.stop();
        }
        PlayerCommand::Play => {
            activity.player.resume();
        }
        PlayerCommand::Seek(x) => {
            activity.player.seek(x.into()).ok();
            // app.player.seek(x);
        }
        PlayerCommand::Position(_track_id, position) => {
            let _position = position / 1000;
            // app.player.position(position);
        }
        PlayerCommand::Load(uri) => {
            // app.player.play_url(&uri);
            activity.player.queue_and_play(&uri);
        }
        PlayerCommand::Metadata(info, tx) => {
            let msg = match info {
                // MetaInfo::LoopStatus => match app.repeat_state {
                //     RepeatState::Off => "None".to_owned(),
                //     RepeatState::All => "Playlist".to_owned(),
                //     RepeatState::Track => "Track".to_owned(),
                //     _ => "None".to_owned(),
                // },
                MetaInfo::LoopStatus => "None".to_owned(),
                MetaInfo::Status => match &activity.current_song {
                    Some(_) => {
                        if activity.player.is_paused() {
                            "Paused".to_owned()
                        } else {
                            "Playing".to_owned()
                        }
                    }
                    None => "Stopped".to_owned(),
                },
                // MetaInfo::Shuffle => match app.repeat_state {
                //     RepeatState::Shuffle => "true".to_owned(),
                //     _ => "false".to_owned(),
                // },
                MetaInfo::Shuffle => "false".to_owned(),
                // MetaInfo::Position => app.player.get_position().unwrap_or(0).to_string(),
                MetaInfo::Position => {
                    let (_, pos, _) = activity.player.get_progress();
                    pos.to_string()
                }
                MetaInfo::Info => {
                    let s = activity.current_song.as_ref().map_or_else(
                        || SongMpris {
                            title: Some("No current song".to_string()),
                            artist: Some("".to_string()),
                            album: Some("".to_string()),
                        },
                        |song| SongMpris {
                            title: Some(song.title().unwrap_or("Unknown Title").to_string()),
                            artist: Some(song.artist().unwrap_or("Unknown Artist").to_string()),
                            album: Some(song.album().unwrap_or("").to_string()),
                        },
                    );
                    serde_json::to_string(&s).unwrap()
                }
                MetaInfo::Volume => "75".to_string(),
            };
            info!("send msg {:#?}", msg);
            tx.send(msg).expect("send error");
        }
    }
}
