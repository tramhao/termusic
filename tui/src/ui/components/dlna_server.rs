use std::path::Path;
use termusiclib::config::SharedTuiSettings;
use termusiclib::types::{DSMsg, GSMsg, LIMsg, Msg, PLMsg, RecVec, TEMsg, YSMsg};
use tui_realm_treeview::{Node, Tree, TreeView, TREE_CMD_CLOSE, TREE_CMD_OPEN, TREE_INITIAL_NODE};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, TableBuilder, TextSpan};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};
use termusiclib::config::v2::server::ScanDepth;
use termusiclib::ids::Id;
use crate::ui::components::MusicLibrary;
use crate::ui::model::{DownloadTracker, Model, TxToMain, UserEvent};
use termusiclib::dlna::discovery;
use termusiclib::dlna::media_server::MediaServerController;
use termusiclib::dlna::models::{DlnaDevice, MediaContainer};
use termusiclib::track::MediaTypes::Track;

#[derive(MockComponent)]
pub struct DlnaServer {
    component: TreeView<String>,
    config: SharedTuiSettings,
    pub init: bool,
}

impl DlnaServer {
    pub fn new(
        tree: &Tree<String>,
        initial_node: Option<String>,
        config: SharedTuiSettings,
    ) -> Self {
        let initial_node = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().to_string(),
        };
        let component = {
            let config = config.read();
            TreeView::default()
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .borders(
                    Borders::default()
                        .color(config.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                .indent_size(2)
                .scroll_step(6)
                .title(" DLNA Server ", Alignment::Left)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlight_symbol(&config.settings.theme.style.library.highlight_symbol)
                .preserve_state(true)
                .with_tree(tree.clone())
                .initial_node(initial_node)
        };

        Self {
            component,
            config,
            init: true
        }
    }

    fn handle_left_key(&mut self) -> CmdResult {
        CmdResult::None
    }
    fn handle_right_key(&mut self) -> (CmdResult, Option<Msg>) {
        let current_node = self.component.tree_state().selected().unwrap();
        let path: &Path = Path::new(current_node);
        if path.is_dir() {
            // TODO: try to load the directory if it is not loaded yet.
            (self.perform(Cmd::Custom(TREE_CMD_OPEN)), None)
        } else {
            (
                CmdResult::None,
                Some(Msg::Playlist(PLMsg::Add(path.to_path_buf()))),
            )
        }
    }
}

impl Component<Msg, UserEvent> for DlnaServer {
    #[allow(clippy::too_many_lines)]
    fn on(&mut self, ev: Event<UserEvent>) -> Option<Msg> {
        // When init, open root
        if self.init {
            let root = self.component.tree().root();
            if self.component.tree_state().is_closed(root) {
                self.perform(Cmd::Custom(TREE_CMD_OPEN));
                self.init = false;
            }
        }
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let result = match ev {
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.left.get() => {
                self.handle_left_key()
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE, 
            }) => self.handle_left_key(),
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.right.get() => {
                match self.handle_right_key() {
                    (_, Some(msg)) => return Some(msg),
                    (cmdresult, None) => cmdresult,
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => match self.handle_right_key() {
                (_, Some(msg)) => return Some(msg),
                (cmd_result, None) => cmd_result,
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }

            _ => CmdResult::None,
        };
        match result {
            CmdResult::Submit(State::One(StateValue::String(node))) => {
                Some(Msg::Library(LIMsg::TreeStepInto(node)))
            }
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw)
        }
    }
}

impl Model {
    
    pub async fn discover_servers(tx: TxToMain)
    {
        if let Ok(dlna_devices) = discovery::discover_devices().await {
            println!("Devices discovered, searching renderer...");
            for dlna_device in dlna_devices {
                println!("Device type {} in {}", dlna_device.device_type, dlna_device.location);
                if dlna_device.device_type.contains("MediaServer") {
                    let root_node = Self::mediaserver_dir_tree(&dlna_device).await;
                    let _ = tx.send(Msg::DlnaServer(DSMsg::TreeNodeReady(root_node, None)));
                    return;
                }
            }
        }
        let empty_node = Self::mediaserver_empty_tree();
        let _ = tx.send(Msg::DlnaServer(DSMsg::TreeNodeReady(empty_node, None)));
    }

    /// Convert a [`RecVec`] to a [`Node`].
    fn mediaserver_recvec_to_node(vec: RecVec<String, String>) -> Node<String> {
        let mut node = Node::new(vec.id, vec.value);

        for val in vec.children {
            node.add_child(Self::mediaserver_recvec_to_node(val));
        }

        node
    }

    pub fn mediaserver_apply_as_tree(
        &mut self,
        msg: RecVec<String, String>,
        focus_node: Option<String>,
    ) {
        let root_path = msg.id.clone();
        let root_node = Self::mediaserver_recvec_to_node(msg);

        let old_current_node = match self.app.state(&Id::DlnaServer).ok().unwrap() {
            State::One(StateValue::String(id)) => Some(id),
            _ => None,
        };

        self.media_server.tree_path = root_path;
        self.media_server.tree = Tree::new(root_node);

        // remount preserves focus
        let _ = self.app.remount(
            Id::DlnaServer,
            Box::new(DlnaServer::new(
                &self.media_server.tree,
                old_current_node,
                self.config_tui.clone(),
            )),
            Vec::new(),
        );

        // focus the specified node
        if let Some(id) = focus_node {
            let _ = self.app.attr(
                &Id::DlnaServer,
                Attribute::Custom(TREE_INITIAL_NODE),
                AttrValue::String(id),
            );
        }
    }

    fn mediaserver_empty_tree() -> RecVec<String, String> {
        let mut node = RecVec {
            id: "0".to_string(),
            value: "No servers found".to_string(), //device.id.to_string(),
            children: Vec::new(),
        };
        node
    }

    fn mediaserver_item(id: String, value: String) -> RecVec<String, String> {
        let mut node = RecVec {
            id,
            value,
            children: Vec::new(),
        };
        node
    }

    async fn mediaserver_dir_tree(device: &DlnaDevice) -> RecVec<String, String> {
    // fn mediaserver_dir_tree() -> RecVec<String, String> {
        let mut node = RecVec {
            // id: "Hello".to_string(),
            id: device.id.to_string(),
            value: device.name.to_string(),
            // value: "World".to_string(), //device.id.to_string(),
            children: Vec::new(),
        };
        let mut ms = MediaServerController::new(device.clone());
        if let Ok(result) = ms.browse_directory(device.name.clone()).await {
            node = Self::mediaserver_add_container(result, node);
        }

        node
    }

    fn mediaserver_add_container(container: MediaContainer, mut root: RecVec<String, String>) -> RecVec<String, String> {
        let mut node = Self::mediaserver_item(container.id, container.name);
        for child in container.childs {
            node = Self::mediaserver_add_container(child, node);
        }
        /*let artists = container.items.iter().map(|artist_item| {
            let artist_name = artist_item.artist.clone().unwrap();
            let mut artist_node = Self::mediaserver_item(artist_item.id.clone(), artist_name.clone());
            let albums: Vec<RecVec<String, String>> = container.items.iter()
                .filter(|x| x.artist==Some(artist_name.clone()))
                .map(|album_item| {
                    let album_name = album_item.album.clone().unwrap();
                    let mut album_node = Self::mediaserver_item(album_item.id.clone(), album_name.clone());
                    let songs: Vec<RecVec<String, String>> = container.items.iter()
                        .filter(|x| x.album==Some(album_name.clone()) && x.artist==Some(artist_name.clone()))
                        .map(|track_item| {
                            let element = format!("{} {}", track_item.track.clone(), track_item.title.clone());
                            let song = Self::mediaserver_item(track_item.url.clone(), element);
                            song
                        }).collect();
                    for song in songs {
                        album_node.children.push(song);
                    }
                    album_node
            }).collect();
            for album in albums {
                artist_node.children.push(album);
            }
            artist_node
        });
        for artist in artists {
            node.children.push(artist);
        } */
        let mut songs = 0;
        for item in container.items {
            let album = item.album.unwrap_or("Unknown album".to_string());
            let artist = item.artist.unwrap_or("Unknown artist".to_string());
            let element = format!("[{}] {} {} - {}", album, item.track, artist, item.title);
            let song = Self::mediaserver_item(item.url, element);
            node.children.push(song);
            songs += 1;
            if songs > 1000 {
                break;
            }
        }
        // root.children.push(node);
        node
    }
}