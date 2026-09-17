use std::collections::HashSet;

use anyhow::Result;
use termusiclib::config::SharedTuiSettings;
use tokio_util::sync::{CancellationToken, DropGuard};
use tui_realm_stdlib::components::Paragraph;
use tuirealm::{
    component::{AppComponent, Component},
    event::{Event, Key, KeyEvent},
    props::{
        BorderType, Borders, HorizontalAlignment, LineStatic, TextModifiers, TextStatic, Title,
    },
};

use crate::ui::model::{Model, TMPTrackLoadMsg, UserEvent};
use crate::ui::msg::Msg;
use crate::ui::{ids::Id, msg::GSMsg};

#[derive(Component)]
pub struct SearchDataLoading {
    component: Paragraph,
    config: SharedTuiSettings,

    /// Automatically cancel the load request if this component gets dropped (unmounted).
    #[expect(unused)]
    drop_guard: DropGuard,
}

impl SearchDataLoading {
    pub fn new(config: SharedTuiSettings, drop_guard: DropGuard) -> Self {
        let component = {
            let config = config.read();
            Paragraph::default()
                .borders(
                    Borders::default()
                        .color(config.settings.theme.fallback_border())
                        .modifiers(BorderType::Rounded),
                )
                .title(Title::from(" Loading Data ").alignment(HorizontalAlignment::Center))
                .foreground(config.settings.theme.fallback_foreground())
                .background(config.settings.theme.fallback_background())
                .modifiers(TextModifiers::BOLD)
                .alignment_horizontal(HorizontalAlignment::Center)
                .text(TextStatic::from_iter([
                    LineStatic::from("Loading Search data, please wait..."),
                    LineStatic::from(format!(
                        "Use <ESC>, <{}> or <{}> to close the request",
                        config.settings.keys.quit, config.settings.keys.escape
                    )),
                ]))
        };

        Self {
            component,
            config,
            drop_guard,
        }
    }
}

impl AppComponent<Msg, UserEvent> for SearchDataLoading {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => {
                Some(Msg::GeneralSearch(GSMsg::CloseLoading))
            }
            Event::Keyboard(key) if key == keys.quit.get() => {
                Some(Msg::GeneralSearch(GSMsg::CloseLoading))
            }
            Event::Keyboard(key) if key == keys.escape.get() => {
                Some(Msg::GeneralSearch(GSMsg::CloseLoading))
            }
            _ => None,
        }
    }
}

impl Model {
    /// Mount the "Playlist Data loading..." popup for search.
    pub fn mount_search_loading_playlist_data(&mut self) -> Result<()> {
        let token = CancellationToken::new();

        self.app.mount(
            Id::GeneralSearchDataLoading,
            Box::new(SearchDataLoading::new(
                self.config_tui.clone(),
                token.clone().drop_guard(),
            )),
            Vec::new(),
        )?;
        self.app.active(&Id::GeneralSearchDataLoading)?;

        let playlist = self.playback.playlist.read();
        let tracks = HashSet::from_iter(playlist.tracks().iter().map(Clone::clone));
        let _ = playlist.request_data(TMPTrackLoadMsg::LoadUncached(tracks, token));

        Ok(())
    }

    /// Unmount the "Playlist Data loading..." popup for search.
    pub fn umount_search_loading_playlist_data(&mut self) -> Result<()> {
        self.app.umount(&Id::GeneralSearchDataLoading)?;

        Ok(())
    }
}
