use termusiclib::config::{SharedTuiSettings, TuiOverlay};
use termusiclib::types::{Id, Msg, PCMsg};
use tui_realm_stdlib::{Input, Table};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, InputType, TableBuilder, TextSpan},
    Component, Event, MockComponent, NoUserEvent, State, StateValue,
};

use crate::ui::components::popups::DeleteConfirmInputPopup;
use crate::ui::model::Model;

use super::{YNConfirm, YNConfirmStyle};

#[derive(MockComponent)]
pub struct PodcastAddPopup {
    component: Input,
}

impl PodcastAddPopup {
    pub fn new(config: &TuiOverlay) -> Self {
        let config = &config.settings;
        Self {
            component: Input::default()
                .foreground(config.theme.library_foreground())
                .background(config.theme.library_background())
                .borders(
                    Borders::default()
                        .color(config.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                // .invalid_style(Style::default().fg(Color::Red))
                .input_type(InputType::Text)
                .title(
                    " Add or search podcast feed : (Enter to confirm) ",
                    Alignment::Left,
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for PodcastAddPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::SHIFT | KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::Podcast(PCMsg::PodcastAddPopupCloseCancel));
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => match self.component.state() {
                State::One(StateValue::String(input_string)) => {
                    return Some(Msg::Podcast(PCMsg::PodcastAddPopupCloseOk(input_string)));
                }
                _ => CmdResult::None,
            },
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

#[derive(MockComponent)]
pub struct FeedDeleteConfirmRadioPopup {
    component: YNConfirm,
}

impl FeedDeleteConfirmRadioPopup {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component =
            YNConfirm::new_with_cb(config, " Are sure you to delete the feed? ", |config| {
                YNConfirmStyle {
                    foreground_color: config.settings.theme.library_foreground(),
                    background_color: config.settings.theme.library_background(),
                    border_color: config.settings.theme.library_border(),
                    title_alignment: Alignment::Left,
                }
            });

        Self { component }
    }
}

impl Component<Msg, NoUserEvent> for FeedDeleteConfirmRadioPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        self.component.on(
            ev,
            Msg::Podcast(PCMsg::FeedDeleteCloseOk),
            Msg::Podcast(PCMsg::FeedDeleteCloseCancel),
        )
    }
}

#[derive(MockComponent)]
pub struct PodcastSearchTablePopup {
    component: Table,
    config: SharedTuiSettings,
}

impl PodcastSearchTablePopup {
    pub fn new(config: SharedTuiSettings) -> Self {
        let component = {
            let config = config.read();
            Table::default()
                .background(config.settings.theme.library_background())
                .foreground(config.settings.theme.library_foreground())
                .borders(
                    Borders::default()
                        .color(config.settings.theme.library_border())
                        .modifiers(BorderType::Rounded),
                )
                // .foreground(Color::Yellow)
                .title(" Enter to add feed: ", Alignment::Left)
                .scroll(true)
                .highlighted_color(config.settings.theme.library_highlight())
                .highlighted_str(&config.settings.theme.style.library.highlight_symbol)
                // .highlighted_str("🚀")
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&[" Name ", " url "])
                .column_spacing(3)
                .widths(&[40, 60])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::from("Empty result."))
                        .add_col(TextSpan::from("Loading..."))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for PodcastSearchTablePopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        let cmd_result = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                return Some(Msg::Podcast(PCMsg::SearchItunesCloseCancel))
            }
            Event::Keyboard(keyevent) if keyevent == keys.quit.get() => {
                return Some(Msg::Podcast(PCMsg::SearchItunesCloseCancel))
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => self.perform(Cmd::Move(Direction::Down)),

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.down.get() => {
                self.perform(Cmd::Move(Direction::Down))
            }

            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.up.get() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                ..
            }) => self.perform(Cmd::Scroll(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::PageUp, ..
            }) => self.perform(Cmd::Scroll(Direction::Up)),
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_top.get() => {
                self.perform(Cmd::GoTo(Position::Begin))
            }
            Event::Keyboard(keyevent) if keyevent == keys.navigation_keys.goto_bottom.get() => {
                self.perform(Cmd::GoTo(Position::End))
            }
            // Event::Keyboard(KeyEvent {
            //     code: Key::Tab,
            //     modifiers: KeyModifiers::NONE,
            // }) => return Some(Msg::YoutubeSearch(YSMsg::TablePopupNext)),
            // Event::Keyboard(KeyEvent {
            //     code: Key::BackTab,
            //     modifiers: KeyModifiers::SHIFT,
            // }) => return Some(Msg::YoutubeSearch(YSMsg::TablePopupPrevious)),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                if let State::One(StateValue::Usize(index)) = self.state() {
                    return Some(Msg::Podcast(PCMsg::SearchItunesCloseOk(index)));
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        };
        match cmd_result {
            CmdResult::None => None,
            _ => Some(Msg::ForceRedraw),
        }
    }
}

impl Model {
    pub fn mount_feed_delete_confirm_radio(&mut self) {
        assert!(self
            .app
            .remount(
                Id::FeedDeleteConfirmRadioPopup,
                Box::new(FeedDeleteConfirmRadioPopup::new(self.config_tui.clone())),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::FeedDeleteConfirmRadioPopup).is_ok());
    }

    pub fn umount_feed_delete_confirm_radio(&mut self) {
        if self.app.mounted(&Id::FeedDeleteConfirmRadioPopup) {
            assert!(self.app.umount(&Id::FeedDeleteConfirmRadioPopup).is_ok());
        }
    }
    pub fn mount_feed_delete_confirm_input(&mut self) {
        assert!(self
            .app
            .remount(
                Id::FeedDeleteConfirmInputPopup,
                Box::new(DeleteConfirmInputPopup::new(
                    &self.config_tui.read(),
                    "You're about the erase all feeds.",
                    Msg::Podcast(PCMsg::FeedsDeleteCloseOk),
                    Msg::Podcast(PCMsg::FeedsDeleteCloseCancel)
                )),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::FeedDeleteConfirmInputPopup).is_ok());
    }
    pub fn umount_feed_delete_confirm_input(&mut self) {
        if self.app.mounted(&Id::FeedDeleteConfirmInputPopup) {
            assert!(self.app.umount(&Id::FeedDeleteConfirmInputPopup).is_ok());
        }
    }

    pub fn mount_podcast_search_table(&mut self) {
        assert!(self
            .app
            .remount(
                Id::PodcastSearchTablePopup,
                Box::new(PodcastSearchTablePopup::new(self.config_tui.clone())),
                vec![]
            )
            .is_ok());
        assert!(self.app.active(&Id::PodcastSearchTablePopup).is_ok());
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }
    }

    pub fn update_podcast_search_table(&mut self) {
        let mut table: TableBuilder = TableBuilder::default();
        let mut idx = 0;
        if let Some(vec) = &self.podcast.search_results {
            for record in vec {
                if idx > 0 {
                    table.add_row();
                }

                let title = record
                    .title
                    .clone()
                    .unwrap_or_else(|| "no title found".to_string());

                table
                    .add_col(TextSpan::new(title).bold())
                    .add_col(TextSpan::new(record.url.clone()));
                // .add_col(TextSpan::new(record.album().unwrap_or("Unknown Album")));
                idx += 1;
            }
            // if self.player.playlist.is_empty() {
            //     table.add_col(TextSpan::from("0"));
            //     table.add_col(TextSpan::from("empty playlist"));
            //     table.add_col(TextSpan::from(""));
            // }
        }
        let table = table.build();

        self.app
            .attr(
                &Id::PodcastSearchTablePopup,
                tuirealm::Attribute::Content,
                tuirealm::AttrValue::Table(table),
            )
            .ok();
    }
    pub fn umount_podcast_search_table(&mut self) {
        if self.app.mounted(&Id::PodcastSearchTablePopup) {
            assert!(self.app.umount(&Id::PodcastSearchTablePopup).is_ok());
        }
        if let Err(e) = self.update_photo() {
            self.mount_error_popup(e.context("update_photo"));
        }
    }

    pub fn mount_podcast_add_popup(&mut self) {
        assert!(self
            .app
            .remount(
                Id::PodcastAddPopup,
                Box::new(PodcastAddPopup::new(&self.config_tui.read())),
                vec![]
            )
            .is_ok());

        assert!(self.app.active(&Id::PodcastAddPopup).is_ok());
    }

    pub fn umount_podcast_add_popup(&mut self) {
        if self.app.mounted(&Id::PodcastAddPopup) {
            assert!(self.app.umount(&Id::PodcastAddPopup).is_ok());
        }
    }
}
