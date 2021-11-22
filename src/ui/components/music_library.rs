use crate::ui::model::MAX_DEPTH;
use crate::ui::{Id, Model, Msg};
use std::path::{Path, PathBuf};
use tui_realm_treeview::{Node, Tree, TreeView, TREE_CMD_CLOSE, TREE_CMD_OPEN};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::props::{Alignment, BorderType, Borders};
use tuirealm::tui::style::{Color, Style};
use tuirealm::{Component, Event, MockComponent, NoUserEvent, State, StateValue, View};

#[derive(MockComponent)]
pub struct MusicLibrary {
    component: TreeView,
}

impl MusicLibrary {
    pub fn new(tree: Tree, initial_node: Option<String>) -> Self {
        // Preserve initial node if exists
        let initial_node = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().to_string(),
        };
        Self {
            component: TreeView::default()
                .foreground(Color::Reset)
                .borders(
                    Borders::default()
                        .color(Color::LightYellow)
                        .modifiers(BorderType::Rounded),
                )
                .inactive(Style::default().fg(Color::Gray))
                .indent_size(1)
                .scroll_step(6)
                .title(tree.root().id(), Alignment::Left)
                .highlighted_color(Color::LightYellow)
                .highlight_symbol("\u{1f984}")
                // .highlight_symbol("🦄")
                .with_tree(tree)
                .initial_node(initial_node),
        }
    }
}

impl Component<Msg, NoUserEvent> for MusicLibrary {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left | Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Custom(TREE_CMD_CLOSE)),
            Event::Keyboard(KeyEvent {
                code: Key::Right | Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            }) => {
                let current_node = self.component.tree_state().selected().unwrap();
                let p: &Path = Path::new(current_node);
                if p.is_dir() {
                    self.perform(Cmd::Custom(TREE_CMD_OPEN))
                } else {
                    return Some(Msg::PlaylistAdd(current_node.to_string()));
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::End)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Submit),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::LibraryTreeGoToUpperDir),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::LibraryTreeBlur),
            // Event::Keyboard(
            //     KeyEvent {
            //         code: Key::Esc,
            //         modifiers: KeyModifiers::NONE,
            //     }
            //     | KeyEvent {
            //         code: Key::Char('Q'),
            //         modifiers: KeyModifiers::SHIFT,
            //     },
            // ) => return Some(Msg::AppClose),
            _ => return None,
        };
        match result {
            CmdResult::Submit(State::One(StateValue::String(node))) => {
                Some(Msg::LibraryTreeExtendDir(node))
            }
            _ => Some(Msg::None),
        }
    }
}

impl Model {
    pub fn scan_dir(&mut self, p: &Path) {
        self.path = p.to_path_buf();
        self.tree = Tree::new(Self::dir_tree(p, MAX_DEPTH));
    }

    pub fn upper_dir(&self) -> Option<PathBuf> {
        self.path.parent().map(std::path::Path::to_path_buf)
    }

    pub fn extend_dir(&mut self, id: &str, p: &Path, depth: usize) {
        if let Some(node) = self.tree.root_mut().query_mut(&String::from(id)) {
            if depth > 0 && p.is_dir() {
                // Clear node
                node.clear();
                // Scan dir
                if let Ok(e) = std::fs::read_dir(p) {
                    e.flatten().for_each(|x| {
                        node.add_child(Self::dir_tree(x.path().as_path(), depth - 1));
                    });
                }
            }
        }
    }

    pub fn dir_tree(p: &Path, depth: usize) -> Node {
        let name: String = match p.file_name() {
            None => "/".to_string(),
            Some(n) => n.to_string_lossy().into_owned(),
        };
        let mut node: Node = Node::new(p.to_string_lossy().into_owned(), name);
        if depth > 0 && p.is_dir() {
            if let Ok(e) = std::fs::read_dir(p) {
                e.flatten()
                    .for_each(|x| node.add_child(Self::dir_tree(x.path().as_path(), depth - 1)));
            }
        }
        node
    }
    pub fn reload_tree(&mut self, view: &mut View<Id, Msg, NoUserEvent>) {
        let current_node = match view.state(&Id::Library).ok().unwrap() {
            State::One(StateValue::String(id)) => Some(id),
            _ => None,
        };
        // Remount tree
        assert!(view.umount(&Id::Library).is_ok());
        assert!(view
            .mount(
                Id::Library,
                Box::new(MusicLibrary::new(self.tree.clone(), current_node))
            )
            .is_ok());
        assert!(view.active(&Id::Library).is_ok());
    }
}
