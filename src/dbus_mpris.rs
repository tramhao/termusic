// dbus-send example
// according to https://specifications.freedesktop.org/mpris-spec/latest/Player_Interface.html
// for get Properties
// dbus-send --session --print-reply --dest=org.mpris.MediaPlayer2.ncmt /org/mpris/MediaPlayer2 org.freedesktop.DBus.Properties.Get string:"org.mpris.MediaPlayer2.Player" string:"Rate"
// for method
// dbus-send --session --print-reply --dest=org.mpris.MediaPlayer2.ncmt /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.Next
#[cfg(feature = "dbus_mpris")]
extern crate dbus;
use super::app::App;
#[cfg(feature = "dbus_mpris")]
use super::app::RepeatState;
#[cfg(feature = "dbus_mpris")]
use super::handlers::TrackState;
#[cfg(feature = "dbus_mpris")]
use super::model::playlist::Track;
#[cfg(feature = "dbus_mpris")]
use super::player::MetaInfo;
use super::player::PlayerCommand;
#[cfg(feature = "dbus_mpris")]
use dbus::tree::{Access, Factory};
#[cfg(feature = "dbus_mpris")]
use dbus::{
    arg::{messageitem::MessageItem, RefArg, Variant},
    blocking::Connection,
};
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
#[cfg(feature = "dbus_mpris")]
use std::sync::Arc;
use std::thread;
#[cfg(feature = "dbus_mpris")]
use std::{collections::HashMap, time::Duration};

pub struct DbusMpris {
    rx: mpsc::Receiver<PlayerCommand>,
}

impl DbusMpris {
    pub fn new() -> DbusMpris {
        DbusMpris::init()
    }

    pub fn init() -> DbusMpris {
        let (tx, rx) = mpsc::channel();
        info!("start mpris thread");
        let _server_handle = {
            thread::spawn(move || {
                dbus_mpris_server(tx).unwrap();
            })
        };
        info!("finish mpris thread");
        DbusMpris { rx }
    }

    pub fn next(&self) -> Result<PlayerCommand, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
}

#[cfg(not(feature = "dbus_mpris"))]
#[allow(unused)]
pub fn dbus_mpris_server(tx: Sender<PlayerCommand>) -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[cfg(feature = "dbus_mpris")]
pub fn dbus_mpris_server(tx: Sender<PlayerCommand>) -> Result<(), Box<dyn Error>> {
    // Let's start by starting up a connection to the session bus and request a name.
    let mut c = Connection::new_session()?;
    c.request_name("org.mpris.MediaPlayer2.ncmt", false, true, false)?;

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
            iter.append("ncmt".to_string());
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
        let local_tx = tx.clone();
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
                        match serde_json::from_str::<Track>(&r) {
                            Ok(current_playing) => {
                                m.insert(
                                    "mpris:trackid".to_string(),
                                    Variant(Box::new(MessageItem::Int64(
                                        current_playing.id.unwrap().to_owned(),
                                    ))
                                        as Box<dyn RefArg>),
                                );
                                m.insert(
                                    "mpris:length".to_string(),
                                    Variant(Box::new(MessageItem::Int64(i64::from(100) * 1000))
                                        as Box<dyn RefArg>),
                                );
                                m.insert(
                                    "xesam:title".to_string(),
                                    Variant(Box::new(MessageItem::Str(
                                        current_playing.name.unwrap().to_owned(),
                                    ))
                                        as Box<dyn RefArg>),
                                );
                                m.insert(
                                    "xesam:album".to_string(),
                                    Variant(Box::new(MessageItem::Str(
                                        current_playing.album.unwrap().name.unwrap().to_owned(),
                                    ))
                                        as Box<dyn RefArg>),
                                );
                                m.insert(
                                    "xesam:artist".to_string(),
                                    Variant(Box::new(MessageItem::Str(
                                        current_playing.artists.unwrap()[0].name.to_owned(),
                                    ))
                                        as Box<dyn RefArg>),
                                );
                            }
                            Err(_) => {}
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
    let tree = f.tree(()).add(
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
                    .add_p(property_playback_status)
                    .add_p(property_shuffle)
                    .add_p(property_position)
                    .add_p(property_metadata),
            ),
    );
    // .add(f.object_path("/", ()).introspectable());

    // We add the tree to the connection so that incoming method calls will be handled.
    tree.start_receive(&c);
    info!("start");

    // Ok(())
    // Serve clients forever.
    loop {
        c.process(Duration::from_nanos(1))?;
        thread::sleep(Duration::from_millis(250));
    }
}

#[cfg(not(feature = "dbus_mpris"))]
#[allow(unused)]
pub fn dbus_mpris_handler(r: PlayerCommand, app: &mut App) {}

#[cfg(feature = "dbus_mpris")]
pub fn dbus_mpris_handler(r: PlayerCommand, app: &mut App) {
    match r {
        PlayerCommand::Next => {
            app.skip_track(TrackState::Forword);
        }
        PlayerCommand::Previous => {
            app.skip_track(TrackState::Backword);
        }
        PlayerCommand::Pause => {
            app.player.pause();
        }
        PlayerCommand::PlayPause => {
            app.player.pause();
        }
        PlayerCommand::Stop => {
            app.player.stop();
        }
        PlayerCommand::Play => {
            app.player.play();
        }
        PlayerCommand::Seek(x) => {
            app.player.seek(x);
        }
        PlayerCommand::Position(_track_id, position) => {
            let position = position / 1000;
            app.player.position(position);
        }
        PlayerCommand::Load(uri) => {
            app.player.play_url(&uri);
        }
        PlayerCommand::Metadata(info, tx) => {
            let msg = match info {
                MetaInfo::LoopStatus => match app.repeat_state {
                    RepeatState::Off => "None".to_owned(),
                    RepeatState::All => "Playlist".to_owned(),
                    RepeatState::Track => "Track".to_owned(),
                    _ => "None".to_owned(),
                },
                MetaInfo::Status => match &app.current_playing {
                    Some(_) => {
                        if app.player.is_playing() {
                            "Playing".to_owned()
                        } else {
                            "Paused".to_owned()
                        }
                    }
                    None => "Stopped".to_owned(),
                },
                MetaInfo::Shuffle => match app.repeat_state {
                    RepeatState::Shuffle => "true".to_owned(),
                    _ => "false".to_owned(),
                },
                MetaInfo::Position => app.player.get_position().unwrap_or(0).to_string(),
                MetaInfo::Info => serde_json::to_string(&app.current_playing.to_owned()).unwrap(),
                _ => return,
            };
            info!("send msg {:#?}", msg);
            tx.send(msg).expect("send error");
        }
    }
}
