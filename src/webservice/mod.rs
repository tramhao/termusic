// TODO: enable logging
use axum::body::Bytes;
use axum::extract::{BodyStream, Query};
use axum::http::StatusCode;
use axum::middleware;

use axum::routing::post;
use axum::{BoxError, Json};
use directories::UserDirs;
use futures::{Stream, TryStreamExt};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::str::FromStr;
use std::{
    net::SocketAddr,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

use self::middle_ware::auth;
use self::model::{
    AddTrackPayload, GetStatusPayload, PlayerActionPayload, QueryParamUploadTrack, WebApiError,
    WebApiResponse,
};
use crate::config::Settings;
use crate::ui::{self, Msg, PLMsg};
use crate::webservice::model::PlayerInfo;
use axum::{extract::State, Router};
use filenamify::filenamify;
use lazy_static::lazy_static;

pub mod middle_ware;
pub mod model;

lazy_static! {
    pub static ref PLAYER_INFO_HASHMAP: Arc<Mutex<HashMap<String, PlayerInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub fn set_player_info_hashmap(key: String, val: PlayerInfo) -> Result<(), WebApiError> {
    if let Ok(mut map) = PLAYER_INFO_HASHMAP.lock() {
        map.insert(key, val);
        Ok(())
    } else {
        Err(WebApiError::Internal(
            "failed to lock PLAYER_INFO_HASHMAP to set a value".to_string(),
        ))
    }
}

pub struct AppState {
    // send to main ui process
    sender: Mutex<std::sync::mpsc::Sender<ui::Msg>>,
    // pub msg_queue: Arc<Mutex<VecDeque<bool>>>,
    music_dirs: Vec<String>,
    web_service_token: String,
}

// NOTE: handle player action such as pause, unpause, toggle pause, next, prev player
#[allow(clippy::unused_async, clippy::too_many_lines)]
async fn action_handler(
    Json(payload): Json<PlayerActionPayload>,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<WebApiResponse>) {
    let rand_string = generate_random_string(32);

    let message;
    let mut key = None;
    if let Ok(sender) = state.sender.lock() {
        match payload.op {
            model::Op::PlayerStart => {
                message = Msg::PlayerStart;
            }
            model::Op::PlayerStop => {
                message = Msg::PlayerStop;
            }
            model::Op::PlayerNext => {
                message = Msg::Playlist(PLMsg::NextSong);
            }
            model::Op::PlayerPrev => {
                message = Msg::Playlist(PLMsg::PrevSong);
            }
            model::Op::PlayerTogglePause => {
                message = Msg::PlayerTogglePause;
            }
            model::Op::PlayerEnableGapless => {
                message = Msg::PlayerEnableGapless;
            }
            model::Op::PlayerDisableGapless => {
                message = Msg::PlayerDisableGapless;
            }
            model::Op::PlayerToggleGapless => {
                message = Msg::PlayerToggleGapless;
            }
            model::Op::PlayerVolumeUp => {
                message = Msg::PlayerVolumeUp;
            }
            model::Op::PlayerVolumeDown => {
                message = Msg::PlayerVolumeDown;
            }
            model::Op::PlayerSpeedUp => {
                message = Msg::PlayerSpeedUp;
            }
            model::Op::PlayerSpeedDown => {
                message = Msg::PlayerSpeedDown;
            }
            model::Op::ChangeLoopModeQueue => {
                message = Msg::Playlist(PLMsg::LoopModeQueue);
            }
            model::Op::ChangeLoopModePlaylist => {
                message = Msg::Playlist(PLMsg::LoopModePlaylist);
            }
            model::Op::ChangeLoopModeSingle => {
                message = Msg::Playlist(PLMsg::LoopModeSingle);
            }
            model::Op::EnablePlaylistAddFront => {
                message = Msg::PlaylistEnableAddFront(true);
            }
            model::Op::EnablePlaylistAddTail => {
                message = Msg::PlaylistEnableAddFront(false);
            }
            // below part are for getting player information
            model::Op::GetPlayerStatus => {
                message = Msg::PlayerStatus(rand_string.clone());
                key = Some(rand_string);
            }
            model::Op::GetCurrentTrack => {
                message = Msg::PlayerCurrentTrack(rand_string.clone());
                key = Some(rand_string);
            }
            model::Op::GetNextTrack => {
                message = Msg::NextTrack(rand_string.clone());
                key = Some(rand_string);
            }
            // model::Op::GetPrevTrack => {
            //     // message = Msg::PlayerCurrentTrack(rand_string.clone());
            //     // key = Some(rand_string);
            //     // TODO:
            //     todo!()
            // }
            model::Op::GetPlayerGapless => {
                message = Msg::PlayerGapless(rand_string.clone());
                key = Some(rand_string);
            }
            model::Op::GetCurrentLoopMode => {
                message = Msg::LoopMode(rand_string.clone());
                key = Some(rand_string);
            }
            model::Op::GetPlaylistAddFront => {
                message = Msg::PlaylistCurrnetAddFront(rand_string.clone());
                key = Some(rand_string);
            }
        }
        if sender.send(message).is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(WebApiResponse::GeneralResult {
                    result: false,
                    message: "Failed to send message to ui main loop".to_owned(),
                    key,
                }),
            );
        }
        (
            StatusCode::OK,
            axum::Json(WebApiResponse::GeneralResult {
                result: true,
                message: String::new(),
                key,
            }),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(WebApiResponse::GeneralResult {
                result: false,
                message: "Failed to lock state.locker to send message to ui main loop".to_owned(),
                key: None,
            }),
        )
    }
}

#[allow(clippy::unused_async)]
async fn add_track_handler(
    Json(payload): Json<AddTrackPayload>,
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<WebApiResponse>) {
    let p: &Path = Path::new(&payload.path);
    if !p.exists() {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(WebApiResponse::GeneralResult {
                result: false,
                message: "path doesn't exist!".to_string(),
                key: None,
            }),
        );
    }

    let (status_code, result, message, key) = add_track(&state, payload.path, payload.play_now);
    (
        status_code,
        axum::Json(WebApiResponse::GeneralResult {
            result,
            message,
            key,
        }),
    )
}

fn add_track(
    state: &Arc<AppState>,
    file_path: String,
    play_now: bool,
) -> (StatusCode, bool, String, Option<String>) {
    let message = Msg::AddTrack(file_path, play_now);
    if let Ok(sender) = state.sender.lock() {
        if sender.send(message).is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                false,
                "Failed to send message to ui main loop".to_owned(),
                None,
            );
        }
        return (StatusCode::OK, true, String::new(), None);
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        false,
        "Failed to lock state.locker to send message to ui main loop".to_owned(),
        None,
    )
}

#[allow(clippy::unused_async, clippy::too_many_lines)]
async fn upload_track_handler<S, E>(
    Query(params): Query<QueryParamUploadTrack>,
    State(state): State<Arc<AppState>>,
    stream: S,
) -> Result<Json<WebApiResponse>, (StatusCode, Json<WebApiResponse>)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let save_to_music_folder: bool = params.save_to_music_folder.map_or(false, |v| v);
    let play_now: bool = params.play_now.map_or(false, |v| v);

    // NOTE: file_name can't be empty
    if params.file_name.is_none() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(WebApiResponse::UploadTrack {
                result: false,
                message: "query param 'file_name' can't be empty".to_owned(),
                file_name: None,
                play_now: None,
            }),
        ));
    }

    // This unwrap() is safe since we already handle params.file_name=None above
    let old_file_name = params.file_name.unwrap();
    // NOTE: only support file name without path
    if old_file_name.find(['/', '\\']).is_some() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(WebApiResponse::UploadTrack {
                result: false,
                message: "'file_name' should not contain the path. (can't contains '/' or '\')"
                    .to_string(),
                file_name: None,
                play_now: None,
            }),
        ));
    }

    let file_ext: String;
    let new_file_name;
    let mut folder_path: String = String::new();
    let full_file_path;

    // NOTE: check if file extension is supported
    if let Some(ext) = Path::new(&old_file_name).extension() {
        file_ext = ext.to_string_lossy().to_string();
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(WebApiResponse::UploadTrack {
                result: false,
                message: "Failed to get extension of {old_file_name}".to_string(),
                file_name: None,
                play_now: None,
            }),
        ));
    }

    // NOTE: save to user's <music> folder
    if save_to_music_folder {
        if let Some(dir) = state.music_dirs.iter().find(|v| v.ends_with("Music")) {
            folder_path = dir.to_string();
        } else if let Some(dir) = state.music_dirs.first() {
            folder_path = dir.to_string();
        } else if let Some(user_dirs) = UserDirs::new() {
            if let Some(dir) = user_dirs.audio_dir() {
                folder_path = dir.to_string_lossy().into_owned();
            }
        }

        if folder_path.is_empty() {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(WebApiResponse::UploadTrack {
                    result: false,
                    message: "Failed to get music folder.".to_string(),
                    file_name: None,
                    play_now: None,
                }),
            ));
        }
        // println!("file_name={file_name}, folder_path={folder_path}");
        new_file_name = filenamify(old_file_name);
        full_file_path = Path::new(&folder_path).join(&new_file_name);
    }
    // NOTE: save to tmp folder
    else {
        folder_path = std::env::temp_dir().to_string_lossy().into_owned();
        new_file_name = generate_random_string(32) + "." + &file_ext;
        full_file_path = Path::new(&folder_path).join(&new_file_name);
    }

    let result = async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let mut file = BufWriter::new(File::create(&full_file_path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await;

    match result {
        Ok(_) => {
            // NOTE: add to playlist and play it immediately if params.play_now=true
            let (status_code, _result, message, _key) = add_track(
                &state,
                full_file_path.to_string_lossy().to_string(),
                play_now,
            );
            if status_code != StatusCode::OK {
                return Err((
                    status_code,
                    axum::Json(WebApiResponse::UploadTrack {
                        result: false,
                        message,
                        file_name: None,
                        play_now: None,
                    }),
                ));
            }
            Ok(axum::Json(WebApiResponse::UploadTrack {
                result: true,
                message: "Succesfully uploading track.".to_string(),
                file_name: Some(new_file_name),
                play_now: Some(play_now),
            }))
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(WebApiResponse::UploadTrack {
                result: false,
                message: err.to_string(),
                file_name: None,
                play_now: None,
            }),
        )),
    }
}

// NOTE: to get the result for the request of fetching player information such as "paused or not
// paused", current playing song
#[allow(clippy::unused_async)]
async fn get_result_handler(
    Json(payload): Json<GetStatusPayload>,
    State(_state): State<Arc<AppState>>,
) -> (StatusCode, Json<WebApiResponse>) {
    if let Some(key) = payload.key {
        // println!("get_result_handler: key={key}");
        match PLAYER_INFO_HASHMAP.lock() {
            Ok(mut hm) => {
                if let Some(value) = hm.remove(&key) {
                    // format!("<h1>Found value={value:?}</h1>")
                    match value {
                        PlayerInfo::CurrentStatus(is_playing) => (
                            StatusCode::OK,
                            axum::Json(WebApiResponse::GetPlayerStatus {
                                result: true,
                                message: String::new(),
                                is_playing,
                            }),
                        ),
                        PlayerInfo::CurrentTrack(track) | PlayerInfo::NextTrack(track) => (
                            StatusCode::OK,
                            axum::Json(WebApiResponse::GetTrack {
                                result: true,
                                message: String::new(),
                                artist: track.artist().map(std::borrow::ToOwned::to_owned),
                                album: track.album().map(std::borrow::ToOwned::to_owned),
                                title: track.title().map(std::borrow::ToOwned::to_owned),
                                duration: track.duration().as_secs(),
                                file_path: track.file().map(std::borrow::ToOwned::to_owned),
                            }),
                        ),
                        PlayerInfo::CurrentGapless(is_gapless) => (
                            StatusCode::OK,
                            axum::Json(WebApiResponse::GetPlayerGapless {
                                result: true,
                                message: String::new(),
                                is_gapless,
                            }),
                        ),
                        PlayerInfo::NotFoundTrack => (
                            StatusCode::OK,
                            axum::Json(WebApiResponse::GetTrack {
                                result: true,
                                message: "Not found track".to_string(),
                                artist: None,
                                album: None,
                                title: None,
                                // NOTE: duration=0 means we don't find track info
                                duration: 0,
                                file_path: None,
                            }),
                        ),
                        PlayerInfo::CurrentLoopMode(loop_mode) => (
                            StatusCode::OK,
                            axum::Json(WebApiResponse::GetLoopMode {
                                result: true,
                                message: String::new(),
                                loop_mode,
                            }),
                        ),
                        PlayerInfo::CurrentPlaylistAddFront(add_front) => (
                            StatusCode::OK,
                            axum::Json(WebApiResponse::GetAddFront {
                                result: true,
                                message: String::new(),
                                add_front,
                            }),
                        ),
                    }
                } else {
                    (
                        StatusCode::NOT_FOUND,
                        axum::Json(WebApiResponse::GeneralResult {
                            result: false,
                            message: "Not found this key, did you call /action to get player's info or did you provide correct 'key'?".to_string(),
                            key: None,
                        })
                    )
                }
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(WebApiResponse::GeneralResult {
                    result: false,
                    message: "Failed to lock state.locker to send message to ui main loop"
                        .to_owned(),
                    key: None,
                }),
            ),
        }
    } else {
        (
            StatusCode::NOT_ACCEPTABLE,
            axum::Json(WebApiResponse::GeneralResult {
                result: false,
                message: "Not found key in json payload".to_string(),
                key: None,
            }),
        )
    }
}

pub fn run_music_web_cmd_process(tx_to_main: Sender<Msg>, settings: &Settings) {
    let music_folders = settings.music_dir.clone();

    if let Some(addr) = settings.web_service_addr.clone() {
        let shared_state = Arc::new(AppState {
            sender: Mutex::new(tx_to_main),
            music_dirs: music_folders,
            web_service_token: settings
                .web_service_token
                .clone()
                .expect("web_service_token can't be empty"),
        });

        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let app = Router::new()
                        .route(
                            "/action",
                            post({
                                let shared_state = Arc::clone(&shared_state);
                                move |body| action_handler(body, axum::extract::State(shared_state))
                            }),
                        )
                        .route(
                            "/get_status",
                            post({
                                let shared_state = Arc::clone(&shared_state);
                                move |body| {
                                    get_result_handler(body, axum::extract::State(shared_state))
                                }
                            }),
                        )
                        .route(
                            "/add_track",
                            post({
                                let shared_state = Arc::clone(&shared_state);
                                move |body| {
                                    add_track_handler(body, axum::extract::State(shared_state))
                                }
                            }),
                        )
                        .route(
                            "/upload_track",
                            post({
                                let shared_state = Arc::clone(&shared_state);
                                move |query, request: BodyStream| {
                                    upload_track_handler(
                                        query,
                                        axum::extract::State(shared_state),
                                        request,
                                    )
                                }
                            }),
                        )
                        .route_layer(middleware::from_fn_with_state(
                            Arc::clone(&shared_state),
                            auth,
                        ))
                        .with_state(shared_state);

                    // run it
                    if let Ok(socket_addr) = SocketAddr::from_str(&addr) {
                        // let addr = SocketAddr::from_str(config.web);
                        // println!("listening on {addr}");
                        axum::Server::bind(&socket_addr)
                            .serve(app.into_make_service())
                            .await
                            .unwrap();
                    }
                });
        });
    }
}

pub fn generate_random_string(len: u8) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len.into())
        .map(char::from)
        .collect()
}
