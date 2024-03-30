use termusiclib::{
    config::BindingForEvent,
    types::{Id, Msg},
};
use termusicplayback::SharedSettings;
use tui_realm_stdlib::Table;
use tuirealm::{
    command::{Cmd, CmdResult, Direction},
    event::{Key, KeyEvent, KeyModifiers},
    props::{Alignment, BorderType, Borders, Color, TableBuilder, TextSpan},
    Component, Event, MockComponent, NoUserEvent,
};

use crate::ui::model::Model;

#[derive(MockComponent)]
pub struct HelpPopup {
    component: Table,
    config: SharedSettings,
}

impl HelpPopup {
    fn key(keys: &[BindingForEvent]) -> TextSpan {
        let mut text = String::new();
        for (idx, key) in keys.iter().enumerate() {
            if idx > 0 {
                text.push_str(", ");
            }
            text.push_str(&format!("<{key}>"));
        }
        TextSpan::from(text).bold().fg(Color::Cyan)
    }
    fn comment(text: &str) -> TextSpan {
        TextSpan::new(text)
    }
    #[allow(clippy::too_many_lines)]
    pub fn new(config: SharedSettings) -> Self {
        let component = {
            let config = config.read();
            let keys = &config.keys;
            Table::default()
                .borders(
                    Borders::default().modifiers(BorderType::Rounded).color(
                        config
                            .style_color_symbol
                            .library_border()
                            .unwrap_or(Color::Green),
                    ),
                )
                .foreground(
                    config
                        .style_color_symbol
                        .library_foreground()
                        .unwrap_or(Color::Yellow),
                )
                .background(
                    config
                        .style_color_symbol
                        .library_background()
                        .unwrap_or(Color::Black),
                )
                .highlighted_color(
                    config
                        .style_color_symbol
                        .library_highlight()
                        .unwrap_or(Color::LightBlue),
                )
                .highlighted_str(&config.style_color_symbol.library_highlight_symbol)
                .scroll(true)
                .title("Help: Esc or Enter to exit.", Alignment::Center)
                .rewind(false)
                .step(4)
                .row_height(1)
                .headers(&["Key", "Function"])
                .column_spacing(3)
                .widths(&[40, 60])
                .table(
                    TableBuilder::default()
                        .add_col(TextSpan::new("Global").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.global_esc, keys.global_quit]))
                        .add_col(Self::comment("Exit"))
                        .add_row()
                        .add_col(TextSpan::new("<TAB>, <SHIFT+TAB>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_left,
                            keys.global_right,
                            keys.global_up,
                            keys.global_down,
                            keys.global_goto_top,
                            keys.global_goto_bottom,
                        ]))
                        .add_col(Self::comment("Move cursor(vim style by default)"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_seek_forward,
                            keys.global_player_seek_backward,
                        ]))
                        .add_col(Self::comment("Seek forward/backward 5 seconds"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_lyric_adjust_forward,
                            keys.global_lyric_adjust_backward,
                        ]))
                        .add_col(Self::comment("Seek forward/backward 1 second for lyrics"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_speed_up,
                            keys.global_player_speed_down,
                        ]))
                        .add_col(Self::comment("Playback speed up/down 10 percent"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_player_toggle_gapless]))
                        .add_col(Self::comment("Toggle gapless playback"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_lyric_adjust_forward,
                            keys.global_lyric_adjust_backward,
                        ]))
                        .add_col(Self::comment("Before 10 seconds,adjust offset of lyrics"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_lyric_cycle]))
                        .add_col(Self::comment("Switch lyrics if more than 1 available"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_next,
                            keys.global_player_previous,
                            keys.global_player_toggle_pause,
                        ]))
                        .add_col(Self::comment("Next/Previous/Pause current track"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_player_volume_plus_1,
                            keys.global_player_volume_plus_2,
                            keys.global_player_volume_minus_1,
                            keys.global_player_volume_minus_2,
                        ]))
                        .add_col(Self::comment("Increase/Decrease volume"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_config_open]))
                        .add_col(Self::comment("Open Config Editor(all configuration)"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_save_playlist]))
                        .add_col(Self::comment("Save Playlist to m3u"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_layout_treeview]))
                        .add_col(Self::comment("Switch layout to treeview"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_layout_database]))
                        .add_col(Self::comment("Switch layout to database"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_layout_podcast]))
                        .add_col(Self::comment("Switch layout to podcast"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_xywh_move_left,
                            keys.global_xywh_move_right,
                        ]))
                        .add_col(Self::comment("Move album cover left/right"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_xywh_move_up,
                            keys.global_xywh_move_down,
                        ]))
                        .add_col(Self::comment("Move album cover up/down"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.global_xywh_zoom_in,
                            keys.global_xywh_zoom_out,
                        ]))
                        .add_col(Self::comment("Zoom in/out album cover"))
                        .add_row()
                        .add_col(Self::key(&[keys.global_xywh_hide]))
                        .add_col(Self::comment("Hide/Show album cover"))
                        .add_row()
                        .add_col(TextSpan::new("Library").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.global_right, keys.library_load_dir]))
                        .add_col(Self::comment("Add one/all tracks to playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_delete]))
                        .add_col(Self::comment("Delete track or folder"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_search_youtube]))
                        .add_col(Self::comment("Search or download track from youtube"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_tag_editor_open]))
                        .add_col(Self::comment("Open tag editor for tag and lyric download"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_yank, keys.library_paste]))
                        .add_col(Self::comment("Yank and Paste files"))
                        .add_row()
                        .add_col(TextSpan::new("<Enter>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Open sub directory as root"))
                        .add_row()
                        .add_col(TextSpan::new("<Backspace>").bold().fg(Color::Cyan))
                        .add_col(TextSpan::from("Go back to parent directory"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_search]))
                        .add_col(Self::comment("Search in library"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_switch_root]))
                        .add_col(Self::comment("Switch among several root folders"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_add_root]))
                        .add_col(Self::comment("Add new root folder"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_remove_root]))
                        .add_col(Self::comment("Remove current root from root folder list"))
                        .add_row()
                        .add_col(TextSpan::new("Playlist").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_delete, keys.playlist_delete_all]))
                        .add_col(Self::comment("Delete one/all tracks from playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_play_selected]))
                        .add_col(Self::comment("Play selected"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_shuffle]))
                        .add_col(Self::comment("Randomize playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_mode_cycle]))
                        .add_col(Self::comment("Loop mode cycle"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_search]))
                        .add_col(Self::comment("Search in playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.playlist_swap_down, keys.playlist_swap_up]))
                        .add_col(Self::comment("Swap track down/up in playlist"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.playlist_cmus_tqueue,
                            keys.playlist_cmus_lqueue,
                        ]))
                        .add_col(Self::comment("Select random tracks/albums to playlist"))
                        .add_row()
                        .add_col(TextSpan::new("Database").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.global_right, keys.database_add_all]))
                        .add_col(Self::comment("Add one/all track(s) to playlist"))
                        .add_row()
                        .add_col(Self::key(&[keys.library_search]))
                        .add_col(Self::comment("Search in database"))
                        .add_row()
                        .add_col(TextSpan::new("Podcast").bold().fg(Color::LightYellow))
                        .add_row()
                        .add_col(Self::key(&[keys.podcast_search_add_feed]))
                        .add_col(Self::comment("Feeds: search or add feed"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.podcast_delete_feed,
                            keys.podcast_delete_all_feeds,
                        ]))
                        .add_col(Self::comment("Feeds : delete one/all feeds"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.podcast_refresh_feed,
                            keys.podcast_refresh_all_feeds,
                        ]))
                        .add_col(Self::comment("Feeds : refresh one/all feeds"))
                        .add_row()
                        .add_col(Self::key(&[
                            keys.podcast_mark_played,
                            keys.podcast_mark_all_played,
                        ]))
                        .add_col(Self::comment("Episode: Mark one/all episodes played"))
                        .add_row()
                        .add_col(Self::key(&[keys.podcast_episode_download]))
                        .add_col(Self::comment("Episode: Download episode"))
                        .add_row()
                        .add_col(Self::key(&[keys.podcast_episode_delete_file]))
                        .add_col(Self::comment("Episode: delete episode local file"))
                        .build(),
                )
        };

        Self { component, config }
    }
}

impl Component<Msg, NoUserEvent> for HelpPopup {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().keys;
        let _cmd_result = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::HelpPopupClose),

            Event::Keyboard(key) if key == keys.global_quit.key_event() => {
                return Some(Msg::HelpPopupClose)
            }
            Event::Keyboard(key) if key == keys.global_esc.key_event() => {
                return Some(Msg::HelpPopupClose)
            }

            Event::Keyboard(key) if key == keys.global_down.key_event() => {
                self.perform(Cmd::Move(Direction::Down))
            }
            Event::Keyboard(key) if key == keys.global_up.key_event() => {
                self.perform(Cmd::Move(Direction::Up))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),
            _ => CmdResult::None,
        };

        Some(Msg::None)
    }
}

impl Model {
    /// Mount help popup
    pub fn mount_help_popup(&mut self) {
        assert!(self
            .app
            .remount(
                Id::HelpPopup,
                Box::new(HelpPopup::new(self.config.clone())),
                vec![]
            )
            .is_ok());
        self.update_photo().ok();
        assert!(self.app.active(&Id::HelpPopup).is_ok());
    }
}
