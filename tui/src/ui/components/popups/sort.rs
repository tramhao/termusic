use termusiclib::config::SharedTuiSettings;
use termusiclib::player::{SortCriterion, SortDirection};
use tui_realm_stdlib::components::Table;
use tui_realm_stdlib::prop_ext::CommonHighlight;
use tuirealm::command::{Cmd, Direction};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use tuirealm::props::{
    AttrValue, Attribute, BorderType, Borders, HorizontalAlignment, LineStatic, PropPayload,
    PropValue, Style, TableBuilder, Title,
};
use tuirealm::state::{State, StateValue};

use crate::ui::ids::Id;
use crate::ui::model::{Model, UserEvent};
use crate::ui::msg::{Msg, SortPopupMsg};

/// Describes one sort criterion with its key bindings and display labels.
struct SortHint {
    ascending_key: char,
    descending_key: char,
    criterion: SortCriterion,
    label: &'static str,
    asc_description: &'static str,
    desc_description: &'static str,
}

const SORT_HINTS: &[SortHint] = &[
    SortHint {
        ascending_key: 'a',
        descending_key: 'A',
        criterion: SortCriterion::Alphanumeric,
        label: "Alphanumeric",
        asc_description: "Filename A\u{2192}Z",
        desc_description: "Filename Z\u{2192}A",
    },
    SortHint {
        ascending_key: 't',
        descending_key: 'T',
        criterion: SortCriterion::FirstAdded,
        label: "First Added",
        asc_description: "Date added oldest\u{2192}newest",
        desc_description: "Date added newest\u{2192}oldest",
    },
    SortHint {
        ascending_key: 'd',
        descending_key: 'D',
        criterion: SortCriterion::Duration,
        label: "Duration",
        asc_description: "Length shortest\u{2192}longest",
        desc_description: "Length longest\u{2192}shortest",
    },
];

const TITLE_ASC: &str = " Sort (Ascending) \u{2014} Tab: toggle, Enter: select, Esc: cancel ";
const TITLE_DESC: &str = " Sort (Descending) \u{2014} Tab: toggle, Enter: select, Esc: cancel ";

/// Build the table content (rows) for the given sort direction.
fn table_data(direction: SortDirection) -> tuirealm::props::Table {
    let mut builder = TableBuilder::default();
    for (idx, hint) in SORT_HINTS.iter().enumerate() {
        let desc = match direction {
            SortDirection::Asc => hint.asc_description,
            SortDirection::Desc => hint.desc_description,
        };
        builder
            .add_col(LineStatic::from(format!(
                "{} / {}",
                hint.ascending_key, hint.descending_key
            )))
            .add_col(LineStatic::from(hint.label))
            .add_col(LineStatic::from(desc));
        if idx < SORT_HINTS.len() - 1 {
            builder.add_row();
        }
    }
    builder.build()
}

/// Build the full `Table` widget with borders, styling, and headers.
fn build_table(config: &SharedTuiSettings, direction: SortDirection) -> Table {
    let config = config.read();
    let table = table_data(direction);
    let title = match direction {
        SortDirection::Asc => TITLE_ASC,
        SortDirection::Desc => TITLE_DESC,
    };

    Table::default()
        .borders(
            Borders::default()
                .modifiers(BorderType::Rounded)
                .color(config.settings.theme.fallback_border()),
        )
        .inactive(Style::new().bg(config.settings.theme.fallback_background()))
        .foreground(config.settings.theme.fallback_foreground())
        .background(config.settings.theme.fallback_background())
        .highlight_style(
            CommonHighlight::default()
                .style
                .fg(config.settings.theme.fallback_highlight()),
        )
        .highlight_str(config.settings.theme.style.library.highlight_symbol.clone())
        .scroll(true)
        .title(Title::from(title).alignment(HorizontalAlignment::Center))
        .rewind(false)
        .step(1)
        .row_height(1)
        .headers(["Key", "Name", "Description"])
        .column_spacing(3)
        .widths(&[12, 20, 46])
        .table(table)
}

/// Sort popup component — displays available sort criteria with key hints.
#[derive(Component)]
pub struct SortPopup {
    component: Table,
    direction: SortDirection,
    config: SharedTuiSettings,
}

impl SortPopup {
    /// Create a new sort popup with ascending direction selected.
    pub fn new(config: &SharedTuiSettings) -> Self {
        let component = build_table(config, SortDirection::Asc);
        Self {
            component,
            direction: SortDirection::Asc,
            config: config.clone(),
        }
    }

    /// Match a character against the sort hint key bindings.
    fn match_key(ch: char) -> Option<(SortCriterion, SortDirection)> {
        for hint in SORT_HINTS {
            if ch == hint.ascending_key {
                return Some((hint.criterion, SortDirection::Asc));
            }
            if ch == hint.descending_key {
                return Some((hint.criterion, SortDirection::Desc));
            }
        }
        None
    }

    /// Update the table content in place, preserving the active row highlight.
    ///
    /// Replacing the whole component would drop the `is_active` flag,
    /// making the selection highlight disappear.
    fn rebuild(&mut self, direction: SortDirection) {
        let idx = match self.component.state() {
            State::Single(StateValue::Usize(i)) => Some(i),
            _ => None,
        };
        let title = match direction {
            SortDirection::Asc => TITLE_ASC,
            SortDirection::Desc => TITLE_DESC,
        };
        self.component
            .attr(Attribute::Content, AttrValue::Table(table_data(direction)));
        self.component.attr(
            Attribute::Title,
            AttrValue::Title(Title::from(title).alignment(HorizontalAlignment::Center)),
        );
        if let Some(i) = idx {
            self.component.attr(
                Attribute::Value,
                AttrValue::Payload(PropPayload::Single(PropValue::Usize(i))),
            );
        }
    }

    /// Return the currently highlighted sort criterion and direction, if any.
    fn selected_criterion(&self) -> Option<(SortCriterion, SortDirection)> {
        let State::Single(StateValue::Usize(idx)) = self.component.state() else {
            return None;
        };
        SORT_HINTS.get(idx).map(|h| (h.criterion, self.direction))
    }
}

impl AppComponent<Msg, UserEvent> for SortPopup {
    fn on(&mut self, ev: &Event<UserEvent>) -> Option<Msg> {
        let config = self.config.clone();
        let keys = &config.read().settings.keys;
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                ..
            }) => {
                if let Some((criterion, direction)) = Self::match_key(*ch) {
                    return Some(Msg::SortPopup(SortPopupMsg::Selected(criterion, direction)));
                }
                None
            }
            Event::Keyboard(key) if key == keys.quit.get() => {
                Some(Msg::SortPopup(SortPopupMsg::Close))
            }
            Event::Keyboard(key) if key == keys.escape.get() => {
                Some(Msg::SortPopup(SortPopupMsg::Close))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self
                .selected_criterion()
                .map(|(c, d)| Msg::SortPopup(SortPopupMsg::Selected(c, d))),
            Event::Keyboard(KeyEvent {
                code: Key::Tab,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.direction = match self.direction {
                    SortDirection::Asc => SortDirection::Desc,
                    SortDirection::Desc => SortDirection::Asc,
                };
                self.rebuild(self.direction);
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent { code: Key::Up, .. }) => {
                self.perform(Cmd::Move(Direction::Up));
                Some(Msg::ForceRedraw)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down, ..
            }) => {
                self.perform(Cmd::Move(Direction::Down));
                Some(Msg::ForceRedraw)
            }
            _ => None,
        }
    }
}

impl Model {
    /// Mount the sort popup, hiding the album cover behind it.
    pub fn mount_sort_popup(&mut self) {
        assert!(
            self.app
                .remount(
                    Id::SortPopup,
                    Box::new(SortPopup::new(&self.config_tui)),
                    vec![]
                )
                .is_ok()
        );
        self.update_photo().ok();
        assert!(self.app.active(&Id::SortPopup).is_ok());
    }

    /// Unmount the sort popup.
    pub fn umount_sort_popup(&mut self) {
        self.app.umount(&Id::SortPopup).ok();
    }
}
