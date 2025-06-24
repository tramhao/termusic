use termusiclib::config::SharedTuiSettings;
use termusiclib::types::{GSMsg, LIMsg, Msg, PLMsg, RecVec, TEMsg, YSMsg};
use tui_realm_treeview::{Node, Tree, TreeView, TREE_CMD_CLOSE, TREE_CMD_OPEN, TREE_INITIAL_NODE};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders, TableBuilder, TextSpan};
use tuirealm::{AttrValue, Attribute, Component, Event, MockComponent, State, StateValue};

use crate::ui::model::{DownloadTracker, Model, TxToMain, UserEvent};

#[derive(MockComponent)]
pub  struct DlnaServer {
    component: TreeView<String>,
    config: SharedTuiSettings,
    pub  init: bool,
}

impl DlnaServer {
    pub  fn new(
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
                .title("DLNA Server", Alignment::Left)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlight_symbol(&config.settings.theme.style.library.highlight_symbol)
                .preserve_state(true)
                .with_tree(tree.clone())
                .initial_node(initial_node)
        };
        
        Self { 
            component, 
            config, 
            init:  true 
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
                //self.handle_left_key()
                CmdResult::None
            }
            //Event::Keyboard(KeyEvent {
            //    code: Key::Left,
            //    modifiers: KeyModifiers::NONE, 
            //                }) => self.handle_left_key(),
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