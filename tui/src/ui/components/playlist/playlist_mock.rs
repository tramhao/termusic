use std::{fmt::Debug, num::NonZeroUsize};

use tui_realm_stdlib::prop_ext::{CommonHighlight, CommonProps};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    component::Component,
    props::{
        AttrValue, AttrValueRef, Attribute, Borders, Color, LineStatic, Props, Style,
        TextModifiers, Title,
    },
    ratatui::{buffer::Buffer, layout::Rect, text::Span, widgets::Widget},
    state::{State, StateValue},
};

use crate::ui::components::playlist::playlist_widget::PlaylistTableWidget;

/// Custom attributes for [`Attribute::Custom`].
pub mod attr {
    /// Attribute to control the message to display on a empty Table
    pub const EMPTY_TABLE: &str = "empty-table-text";
    // /// Attribute to control the horizontal scroll stepping
    // pub const HORIZ_SCROLL_STEP: &str = "horiz-scroll-step";
    /// Attribute to control the vertical scroll stepping
    pub const VERT_SCROLL_STEP: &str = "vert-scroll-step";
}

/// Custom commands for [`Cmd::Custom`] (/ [`PlaylistTable::perform`]).
pub mod cmd {
    /// Command to run a "Page Down" selection
    pub const PG_UP: &str = "pg-down";
    /// Command to run a "Page Up" selection
    pub const PG_DOWN: &str = "pg-up";
}

/// The distance to preview instead of always having the last displayed element be the selected one.
///
/// Note that this distance is reduced when there is not enough area to display.
pub const PREVIEW_DISTANCE_DEFAULT: usize = 2;

#[derive(Debug, Clone)]
pub struct PlaylistTableContext<'a> {
    /// Areas as defined by the headers.
    ///
    /// There are always the same amount of areas as set in headers, but some [`Rect`]s may be 0-size.
    pub areas: &'a [Rect],
    /// There are always `columns.len() - 1` spacer areas.
    pub areas_spacer: &'a [Rect],
    /// Item offset, most commonly used as the index into a list.
    ///
    /// This is NOT the line offset (if lines can be more than 1 line).
    pub item_offset: usize,
    /// Determines whether the current item is selected or not.
    pub is_selected: bool,
}

#[derive(Debug, Clone)]
pub struct PlaylistTableHeaderContext<'a> {
    /// Areas as defined by the headers.
    ///
    /// There are always the same amount of areas as set in headers, but some [`Rect`]s may be 0-size.
    pub areas: &'a [Rect],
    /// There are always `columns.len() - 1` spacer areas.
    #[expect(unused)]
    pub areas_spacer: &'a [Rect],
    /// All the columns that are defined.
    pub columns: &'a [Column],
}

#[derive(Debug, Clone)]
pub struct ListValueRenderReturn {
    /// Indicate how much vertical space was used to draw this item.
    ///
    /// If `0` is set, this will indicate it is done.
    pub consumed_vertical_size: u16,
    /// Indicate that there is no more data following this item (even if space was used).
    pub done: bool,
}

impl ListValueRenderReturn {
    /// Indicate that no space was used and no further data can be displayed.
    pub const EMPTY: Self = Self {
        consumed_vertical_size: 0,
        done: true,
    };
}

// TODO: allow reducing spacers for specific columns?
#[derive(Debug, Clone)]
pub struct Column {
    /// The text to display in the header.
    pub header_text: String,
    /// Minimal size this column should have.
    pub min_size: u16,
    /// Maximal size this column should have.
    ///
    /// This can be used to allow more growth in other columns while this one only needs at most X amount (ex. Flag column).
    /// Set to `0` to not have a max size.
    pub max_size: u16,
}

impl Column {
    pub fn new<S: Into<String>>(title: S, min_size: u16, max_size: u16) -> Self {
        Self {
            header_text: title.into(),
            min_size,
            max_size,
        }
    }
}

/// Run prep work once before rendering each value.
/// For example to acquire a lock once and releasing it once, not each iteration.
/// If no prep work is required, see the [`ListAcquire::acquire`] example
/// for implementing [`ListValue`] and [`ListAcquire`] on the same struct.
///
/// Release (eg. Lock release) should happen in [`Self::Value`]'s [`Drop`] impl.
pub trait ListAcquire<'a> {
    type Value: ListValue + 'a;

    /// Acquire the value to render, for example to acquire locks.
    ///
    /// Note that this function may also be called outside of the rendering to get access to other functions on [`ListValue`],
    /// for example to [`ListValue::is_empty`].
    ///
    /// To impl [`ListAcquire`] and [`ListValue`] on the same value, follow:
    ///
    /// ```
    /// # struct Type(Vec<String>);
    /// #
    /// impl<'a> ListAcquire<'a> for ListVecString {
    ///     type Value = &'a Self;
    ///
    ///     fn acquire(&'a mut self) -> Self::Value {
    ///         self
    ///     }
    /// }
    /// ```
    fn acquire(&'a mut self) -> Self::Value;
}

pub trait ListValue {
    /// Render a individual value from the list.
    ///
    /// Will be called over and over until `consumed_vertical_size` is `0` or `done` is `true`. (or there is no more remaining area)
    fn render(
        &self,
        buf: &mut Buffer,
        ctx: &PlaylistTableContext<'_>,
        style: Style,
    ) -> ListValueRenderReturn;
    /// Render the headers.
    ///
    /// Will be called over and over until `consumed_vertical_size` is `0` or `done` is `true`. (or there is no more remaining area)
    fn render_header(
        &self,
        buf: &mut Buffer,
        ctx: &PlaylistTableHeaderContext<'_>,
        style: Style,
    ) -> ListValueRenderReturn {
        // draw headers
        for (idx, area) in ctx.areas.iter().enumerate() {
            let column = ctx
                .columns
                .get(idx)
                .expect("Expected Areas to match Column count!");
            Span::styled(&column.header_text, style).render(*area, buf);
        }

        if ctx.areas.is_empty() {
            ListValueRenderReturn {
                consumed_vertical_size: 0,
                done: true,
            }
        } else {
            ListValueRenderReturn {
                consumed_vertical_size: 1,
                done: true,
            }
        }
    }
    /// A known max length, if possible.
    ///
    /// If a max length is known, element scroll will be limited to that length.
    fn len(&self) -> Option<usize>;
    /// Determine if the list is empty and the empty messages should be displayed.
    fn is_empty(&self) -> bool;
    /// Fallback selection.
    ///
    /// Used when needing a initial selection when there was none previously.
    /// This return value does not need to be bounded by [`Self::len`] as that will be done afterwards.
    ///
    /// Example:
    /// Component is a playlist which currently at track idx 3 and we dont have a current selection.
    /// Now if we press DOWN to select something, this implementation may give idx 3 to use for that initial selection instead of always 0.
    fn fallback_select(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone)]
pub struct PlaylistTableState {
    /// The currently selected index.
    selected: Option<usize>,

    /// The last known size of the Table Data area.
    ///
    /// This excludes the block and header size.
    last_size_data: Option<Rect>,
    /// The offset to skip while generating horizontal areas, incase there are more headers than can be displayed at once.
    display_offset_horiz: usize,
    /// Items to scroll by in a single scroll up/down step.
    vertical_scroll_step: usize,

    /// First item idx to display.
    ///
    /// This is NOT the line offset (if lines can be more than 1 line).
    item_offset: usize,

    /// All columns that are gonna be present.
    columns: Vec<Column>,
}

impl Default for PlaylistTableState {
    fn default() -> Self {
        Self {
            selected: None,
            last_size_data: None,
            display_offset_horiz: 0,
            item_offset: 0,
            vertical_scroll_step: 1,
            columns: Vec::new(),
        }
    }
}

impl PlaylistTableState {
    /// Select a specific item or unselect the current node by setting it to `None`.
    ///
    /// Limits to [`ListValue::len`] if available, otherwise allows any value.
    pub fn select(&mut self, item: Option<usize>, data: &impl ListValue) -> Option<usize> {
        let Some(idx) = item else {
            self.selected.take();
            return None;
        };

        if let Some(len) = data.len() {
            if idx < len {
                self.selected = Some(idx);
            }
        } else {
            self.selected = Some(idx);
        }

        if let Some(idx) = self.selected {
            self.set_vert_offset(idx);
        }

        self.selected
    }

    /// Select the first item in the list. Resets the display offset.
    ///
    /// If the list is empty, also clears selection.
    pub fn select_first(&mut self, data: &impl ListValue) -> Option<usize> {
        self.item_offset = 0;
        if data.is_empty() {
            self.selected.take();
        } else {
            self.selected = Some(0);
        }

        self.selected
    }

    /// Select the last item in the list, if the list has a [`ListValue::len`].
    ///
    /// Otherwise, nothing can be selected.
    pub fn select_last(&mut self, data: &impl ListValue) -> Option<usize> {
        if let Some(len) = data.len() {
            self.selected = Some(len.saturating_sub(1));
        }

        if let Some(idx) = self.selected {
            self.set_vert_offset(idx);
        }

        self.selected
    }

    /// Select the next item in the list.
    ///
    /// Does not wrap around.
    ///
    /// If none was selected previously, select idx [`ListValue::fallback_select`].
    pub fn select_next(&mut self, data: &impl ListValue) -> Option<usize> {
        let next = if let Some(current) = self.selected {
            current + 1
        } else {
            data.fallback_select()
        };
        self.select(Some(next), data)
    }

    /// Select the previous item in the list.
    ///
    /// Does not wrap around.
    ///
    /// If none was selected previously, select idx [`ListValue::fallback_select`].
    pub fn select_previous(&mut self, data: &impl ListValue) -> Option<usize> {
        let previous = if let Some(current) = self.selected {
            current.saturating_sub(1)
        } else {
            data.fallback_select()
        };
        self.select(Some(previous), data)
    }

    /// Select the last element (minus [`PREVIEW_DISTANCE_DEFAULT`]) in current view, if that is already selected, scroll down by `area.height - PREVIEW_DISTANCE_DEFAULT`.
    pub fn select_pg_down(&mut self, data: &impl ListValue) -> Option<usize> {
        let Some(offset_curr_sel) = self.selected else {
            return self.select(Some(data.fallback_select()), data);
        };
        let old_offset = self.item_offset;
        let area = self.last_size_data.unwrap_or_default();
        // convert it for easier usage and remove 1 as offsets are index(0) based.
        let height_as_usize = usize::from(area.height.saturating_sub(1));

        // this calculation assumes 1 height per line, so if there is some other usage, this might not be fully correct
        let scroll_by = if (old_offset + height_as_usize).saturating_sub(PREVIEW_DISTANCE_DEFAULT)
            > offset_curr_sel
        {
            (old_offset + height_as_usize)
                .saturating_sub(PREVIEW_DISTANCE_DEFAULT)
                .saturating_sub(offset_curr_sel)
        } else {
            height_as_usize.saturating_sub(PREVIEW_DISTANCE_DEFAULT)
        };

        let new_offset = offset_curr_sel + scroll_by;
        let new_offset = if let Some(len) = data.len() {
            new_offset.min(len.saturating_sub(1))
        } else {
            new_offset
        };

        self.select(Some(new_offset), data)
    }

    /// Select the first element (minus [`PREVIEW_DISTANCE_DEFAULT`]) in current view, if that is already selected, scroll up by `area.height - PREVIEW_DISTANCE_DEFAULT`.
    pub fn select_pg_up(&mut self, data: &impl ListValue) -> Option<usize> {
        let Some(offset_curr_sel) = self.selected else {
            return self.select(Some(data.fallback_select()), data);
        };
        let old_offset = self.item_offset;
        let area = self.last_size_data.unwrap_or_default();
        // convert it for easier usage and remove 1 as offsets are index(0) based.
        let height_as_usize = usize::from(area.height.saturating_sub(1));

        let scroll_by = if old_offset < offset_curr_sel.saturating_sub(PREVIEW_DISTANCE_DEFAULT) {
            // current selection is still above old_offset (plus preview), so we dont scroll a page
            // but select the first element in the visible area (plus preview distance)
            offset_curr_sel
                .saturating_sub(old_offset)
                .saturating_sub(PREVIEW_DISTANCE_DEFAULT)
        } else {
            height_as_usize.saturating_sub(PREVIEW_DISTANCE_DEFAULT)
        };

        let new_offset = offset_curr_sel.saturating_sub(scroll_by);
        let new_offset = if let Some(len) = data.len() {
            new_offset.min(len.saturating_sub(1))
        } else {
            new_offset
        };

        self.select(Some(new_offset), data)
    }

    /// Get the current select value.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Get a copy of the current offset.
    #[expect(unused)]
    pub(crate) fn get_offset_horiz(&self) -> usize {
        self.display_offset_horiz
    }

    /// Get the horizontal offset mutably.
    #[expect(dead_code)]
    pub(crate) fn get_offset_horiz_mut(&mut self) -> &mut usize {
        &mut self.display_offset_horiz
    }

    pub(crate) fn get_item_offset(&self) -> usize {
        self.item_offset
    }

    /// Set the last known Table Data draw area.
    ///
    /// This excludes the block and header size.
    ///
    /// And clamp the display offset so that selected is always visible, if necessary.
    pub(crate) fn set_last_size_data(&mut self, area: Rect) {
        self.last_size_data = Some(area);
    }

    pub(crate) fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Calculate and set the vertical offset for the newly given index to be within view with the least disruption.
    fn set_vert_offset(&mut self, new_idx: usize) {
        let area = self.last_size_data.unwrap_or_default();
        // convert it for easier usage and remove 1 as offsets are index(0) based.
        let height_as_usize = usize::from(area.height.saturating_sub(1));

        let old_offset = self.item_offset;

        // this calculation assumes 1 height per line, so if there is some other usage, this might not be fully correct
        if (old_offset + height_as_usize).saturating_sub(PREVIEW_DISTANCE_DEFAULT) < new_idx {
            // downwards motion
            self.item_offset =
                new_idx.saturating_sub(height_as_usize.saturating_sub(PREVIEW_DISTANCE_DEFAULT));
        } else if old_offset > new_idx.saturating_sub(PREVIEW_DISTANCE_DEFAULT) {
            // upwards motion
            self.item_offset = new_idx.saturating_sub(PREVIEW_DISTANCE_DEFAULT);
        }
    }

    /// Scroll Right by the set stepping.
    pub fn scroll_right(&mut self) {
        self.display_offset_horiz += 1;
    }

    /// Scroll Left by the set stepping.
    pub fn scroll_left(&mut self) {
        self.display_offset_horiz = self.display_offset_horiz.saturating_sub(1);
    }

    /// Scroll down by 1 line, but keep the current selection within view.
    pub fn scroll_down(&mut self, data: &impl ListValue) {
        let Some(offset_curr_sel) = self.selected else {
            // nothing is selected, so there are no bounds to keep aside from the data length, if present
            let next_offset = if let Some(len) = data.len() {
                (self.item_offset + 1).min(len.saturating_sub(1))
            } else {
                self.item_offset + 1
            };
            self.item_offset = next_offset;
            return;
        };

        let old_offset = self.item_offset;

        let new_offset = (old_offset + self.vertical_scroll_step)
            .min(offset_curr_sel.saturating_sub(PREVIEW_DISTANCE_DEFAULT));

        self.item_offset = new_offset;
    }

    /// Scroll up by 1 line, but keep the current selection within view.
    pub fn scroll_up(&mut self) {
        let Some(offset_curr_sel) = self.selected else {
            // nothing is selected, so there are no bounds to keep
            self.item_offset = self.item_offset.saturating_sub(1);
            return;
        };

        let old_offset = self.item_offset;
        let area = self.last_size_data.unwrap_or_default();
        // convert it for easier usage and remove 1 as offsets are index(0) based.
        let height_as_usize = usize::from(area.height.saturating_sub(1));

        let new_offset = old_offset
            .saturating_sub(self.vertical_scroll_step)
            .max(offset_curr_sel.saturating_sub(height_as_usize) + PREVIEW_DISTANCE_DEFAULT);

        self.item_offset = new_offset;
    }

    /// Reset State.
    ///
    /// - Clears selected state
    /// - Clears item offset
    /// - Clears horizontal offset
    ///
    /// Does **not** reset `offset`.
    pub fn clear(&mut self) {
        let _ = self.selected.take();
        self.item_offset = 0;
        self.display_offset_horiz = 0;
    }
}

#[derive(Debug)]
pub struct PlaylistTable<V: for<'a> ListAcquire<'a>> {
    common: CommonProps,
    common_hg: CommonHighlight,
    props: Props,

    state: PlaylistTableState,

    data: V,
}

impl<V: for<'a> ListAcquire<'a>> PlaylistTable<V> {
    pub fn new(data: V) -> Self {
        Self {
            common: CommonProps::default(),
            common_hg: CommonHighlight::default(),
            props: Props::default(),
            state: PlaylistTableState::default(),
            data,
        }
    }

    /// Set the main foreground color for the tree.
    #[expect(unused)]
    pub fn foreground(mut self, color: Color) -> Self {
        self.attr(Attribute::Foreground, AttrValue::Color(color));

        self
    }

    /// Set the main background color for the tree.
    #[expect(unused)]
    pub fn background(mut self, color: Color) -> Self {
        self.attr(Attribute::Background, AttrValue::Color(color));

        self
    }

    /// Set the main text modifiers. This may get overwritten by individual text styles.
    #[expect(unused)]
    pub fn modifiers(mut self, m: TextModifiers) -> Self {
        self.attr(Attribute::TextProps, AttrValue::TextModifiers(m));
        self
    }

    /// Set the main style. This may get overwritten by individual text styles.
    ///
    /// This option will overwrite any previous [`foreground`](Self::foreground), [`background`](Self::background) and [`modifiers`](Self::modifiers)!
    pub fn style(mut self, style: Style) -> Self {
        self.attr(Attribute::Style, AttrValue::Style(style));
        self
    }

    /// Set a custom style to use for the block if the component is not focused.
    ///
    /// If unset, the common style is used.
    ///
    /// Note that style set in [`broder`](Self::border) will be overwritten when unfocused.
    pub fn inactive_style(mut self, style: Style) -> Self {
        self.attr(Attribute::UnfocusedBorderStyle, AttrValue::Style(style));

        self
    }

    /// Set the border style and color that surrounds the tree.
    pub fn border(mut self, border: Borders) -> Self {
        self.attr(Attribute::Borders, AttrValue::Borders(border));

        self
    }

    /// Set a title for the tree in the border.
    pub fn title<T: Into<Title>>(mut self, title: T) -> Self {
        self.attr(Attribute::Title, AttrValue::Title(title.into()));
        self
    }

    /// Set a custom highlight style that is patched ontop of the normal style.
    ///
    /// By default the highlight style is just `Style::new().add_modifier(Modifier::REVERSED)`.
    pub fn highlight_style(mut self, s: Style) -> Self {
        self.attr(Attribute::HighlightStyle, AttrValue::Style(s));

        self
    }

    /// Set a custom highlight style that is patched on-top of the highlight style when unfocused.
    #[expect(unused)]
    pub fn highlight_style_inactive(mut self, s: Style) -> Self {
        self.attr(Attribute::HighlightStyleUnfocused, AttrValue::Style(s));

        self
    }

    /// Set the current curser selection symbol.
    pub fn highlight_symbol<S: Into<LineStatic>>(mut self, s: S) -> Self {
        self.attr(Attribute::HighlightedStr, AttrValue::TextLine(s.into()));

        self
    }

    /// Set custom text for when a table is empty.
    ///
    /// This can be used to set `Loading...` for example.
    /// If this text is not applicable anymore, it can be changed via [`.attr`](Self::attr), or add a root node,
    /// whichever is more applicable.
    ///
    /// Default: [`DEFAULT_EMPTY_TABLE_TEXT`](crate::widget::DEFAULT_EMPTY_TABLE_TEXT)
    #[expect(unused)]
    pub fn empty_table_text<S: Into<String>>(mut self, val: S) -> Self {
        // TODO: make this a line?
        self.attr(
            Attribute::Custom(attr::EMPTY_TABLE),
            AttrValue::String(val.into()),
        );

        self
    }

    /// Set the vertical scroll stepping.
    ///
    /// Default: `1`
    pub fn vertical_scroll_step(mut self, value: NonZeroUsize) -> Self {
        self.state.vertical_scroll_step = value.get();
        self
    }

    /// Builder function to set the columns (including Header text) that will be available.
    ///
    /// It is guranteed that there are always the same amount of Areas as there are Columns defined.
    pub fn columns(mut self, columns: Vec<Column>) -> Self {
        self.state.columns = columns;
        self
    }

    /// Get the currently selected item index, if there is one.
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Reset the state for most things.
    ///
    /// This resets:
    /// - selected item
    /// - item offset
    /// - horizontal display offset
    pub fn reset_state(&mut self) {
        self.state.clear();
    }

    /// Handle [`Cmd::GoTo`].
    fn handle_goto(&mut self, position: Position) -> CmdResult {
        let data = self.data.acquire();
        if data.is_empty() {
            return CmdResult::NoChange;
        }

        let state_before = self.state.selected();

        match position {
            Position::Begin => {
                if self.state.select_first(&data) == state_before {
                    CmdResult::NoChange
                } else {
                    CmdResult::Visual
                }
            }
            Position::End => {
                if self.state.select_last(&data) == state_before {
                    CmdResult::NoChange
                } else {
                    CmdResult::Visual
                }
            }
            Position::At(idx) => {
                if self.state.select(Some(idx), &data) == state_before {
                    CmdResult::NoChange
                } else {
                    CmdResult::Visual
                }
            }
        }
    }

    /// Handle [`Cmd::Move`].
    fn handle_move(&mut self, direction: Direction) -> CmdResult {
        let data = self.data.acquire();
        if data.is_empty() {
            return CmdResult::NoChange;
        }

        let state_before = self.state.selected();

        match direction {
            Direction::Down => {
                if self.state.select_next(&data) == state_before {
                    CmdResult::NoChange
                } else {
                    CmdResult::Visual
                }
            }
            Direction::Up => {
                if self.state.select_previous(&data) == state_before {
                    CmdResult::NoChange
                } else {
                    CmdResult::Visual
                }
            }

            _ => CmdResult::Invalid(Cmd::Move(direction)),
        }
    }

    /// Handle [`Cmd::Scroll`].
    fn handle_scroll(&mut self, direction: Direction) -> CmdResult {
        let data = self.data.acquire();
        if data.is_empty() {
            return CmdResult::NoChange;
        }

        match direction {
            Direction::Down => self.state.scroll_down(&data),
            Direction::Up => self.state.scroll_up(),

            Direction::Left => self.state.scroll_left(),
            Direction::Right => self.state.scroll_right(),
        }

        CmdResult::Visual
    }

    /// Handle [`Cmd::Custom(cmd::PG_DOWN)`].
    fn handle_pg_down(&mut self) -> CmdResult {
        let data = self.data.acquire();
        if data.is_empty() {
            return CmdResult::NoChange;
        }

        let state_before = self.state.selected();

        if self.state.select_pg_down(&data) == state_before {
            CmdResult::NoChange
        } else {
            CmdResult::Visual
        }
    }

    /// Handle [`Cmd::Custom(cmd::PG_UP)`].
    fn handle_pg_up(&mut self) -> CmdResult {
        let data = self.data.acquire();
        if data.is_empty() {
            return CmdResult::NoChange;
        }

        let state_before = self.state.selected();

        if self.state.select_pg_up(&data) == state_before {
            CmdResult::NoChange
        } else {
            CmdResult::Visual
        }
    }
}

impl<V> Clone for PlaylistTable<V>
where
    V: for<'a> ListAcquire<'a> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            common: self.common.clone(),
            common_hg: self.common_hg.clone(),
            state: self.state.clone(),
            data: self.data.clone(),
            props: self.props.clone(),
        }
    }
}

impl<V: for<'a> ListAcquire<'a>> Component for PlaylistTable<V> {
    fn view(&mut self, frame: &mut tuirealm::ratatui::prelude::Frame<'_>, area: Rect) {
        if !self.common.display {
            return;
        }

        let data = self.data.acquire();

        let empty_table_text = self
            .props
            .get(Attribute::Custom(attr::EMPTY_TABLE))
            .and_then(AttrValue::as_string);

        let mut widget = PlaylistTableWidget::new(&data)
            .style(self.common.style)
            // .hg_draw_behavior(hg_behavior)
            // .hg_width(hg_width)
            .hg_style(
                self.common_hg
                    .get_style_focus(self.common.style, self.common.is_active()),
            );

        if let Some(symbol) = self.common_hg.get_symbol() {
            widget = widget.hg_str(symbol);
        }
        if let Some(block) = self.common.get_block() {
            widget = widget.block(block);
        }
        if let Some(empty_table_text) = empty_table_text {
            widget = widget.empty_table_text(empty_table_text);
        }

        frame.render_stateful_widget(widget, area, &mut self.state);
    }

    fn query(&self, attr: Attribute) -> Option<tuirealm::props::QueryResult<'_>> {
        if let Some(value) = self
            .common
            .get_for_query(attr)
            .or_else(|| self.common_hg.get_for_query(attr))
        {
            return Some(value);
        }

        match attr {
            Attribute::Custom(attr::VERT_SCROLL_STEP) => {
                Some(AttrValueRef::Length(self.state.vertical_scroll_step).into())
            }
            _ => self.props.get_for_query(attr),
        }
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        if let Some(value) = self
            .common
            .set(attr, value)
            .and_then(|value| self.common_hg.set(attr, value))
        {
            match attr {
                Attribute::Custom(attr::VERT_SCROLL_STEP) => {
                    let val = value.unwrap_length();
                    if val != 0 {
                        self.state.vertical_scroll_step = val;
                    }
                }
                _ => self.props.set(attr, value),
            }
        }
    }

    fn state(&self) -> State {
        // TODO: state values are not accurate or represent much other than "has something been selected"
        // maybe we can update it if "Any" becomes stable <https://github.com/veeso/tui-realm/pull/120>

        // get the actual node value to see if it is still valid within the tree
        if self.state.selected.is_some() {
            State::Single(StateValue::Bool(true))
        } else {
            State::None
        }
    }

    /// This Component implements the following Commands:
    /// - [`Cmd::Move`]:
    ///   - [`Direction::Down`] & [`Direction::Up`]: change selection in that direction, if possible
    /// - [`Cmd::GoTo`]:
    ///   - [`Position::Begin`]: change selection to be the first item
    ///   - [`Position::End`]: change selection to be the last item, if possible
    ///   - [`Position::At`]: change seletion to be a specific item
    /// - [`Cmd::Scroll`]:
    ///   - [`Direction::Down`] & [`Direction::Up`]: scroll down / up without moving the selection; always keeps the selection within view
    ///   - [`Direction::Left`] & [`Direction::Right`]: scroll left / right (scroll is unbounded)
    /// - [`Cmd::Custom`]:
    ///   - [`cmd::PG_DOWN`]: change selection to be one page down (based on last known draw height)
    ///   - [`cmd::PG_UP`]: change selection to be one page up (based on last known draw height)
    ///
    /// Note that [`Cmd::Submit`] and [`Cmd::Delete`] are **NOT** implemented and need to be done manually (ex via [`get_current_selected_node`](Self::get_current_selected_node) on submit action).
    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(direction) => self.handle_move(direction),
            Cmd::Scroll(direction) => self.handle_scroll(direction),
            Cmd::GoTo(position) => self.handle_goto(position),

            Cmd::Custom(cmd::PG_DOWN) => self.handle_pg_down(),
            Cmd::Custom(cmd::PG_UP) => self.handle_pg_up(),

            _ => CmdResult::Invalid(cmd),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tuirealm::{
        command::{Cmd, Direction},
        component::Component,
        ratatui::{layout::Rect, text::Span, widgets::Widget},
    };

    use crate::ui::components::playlist::playlist_mock::{
        Column, ListAcquire, ListValue, ListValueRenderReturn,
    };

    use super::PlaylistTable;

    /// Helper that implements [`ListValue`] for a simple 2D string array
    #[repr(transparent)]
    struct ListVecString(Vec<Vec<String>>);

    impl<'a> ListAcquire<'a> for ListVecString {
        type Value = &'a Self;

        fn acquire(&'a mut self) -> Self::Value {
            self
        }
    }

    impl ListValue for &ListVecString {
        fn render(
            &self,
            buf: &mut tuirealm::ratatui::prelude::Buffer,
            ctx: &super::PlaylistTableContext<'_>,
            style: tuirealm::ratatui::prelude::Style,
        ) -> super::ListValueRenderReturn {
            let Some(item) = self.0.get(ctx.item_offset) else {
                return ListValueRenderReturn::EMPTY;
            };

            for area in ctx.areas {
                // we only draw with Spans here, which can only be 1 height.
                let rect = Rect { height: 1, ..*area };
                buf.set_style(rect, style);
            }

            if ctx.is_selected {
                Span::styled(">", style).render(ctx.areas[0], buf);
            }

            for (idx, elem) in item.iter().enumerate() {
                Span::styled(elem, style).render(ctx.areas[idx], buf);
            }

            ListValueRenderReturn {
                consumed_vertical_size: 1,
                done: false,
            }
        }

        fn len(&self) -> Option<usize> {
            Some(self.0.len())
        }

        fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }

    #[test]
    fn should_work_basic() {
        let data = ListVecString(vec![vec![
            "Row 1".to_string(),
            "Row 2".to_string(),
            "Row 3".to_string(),
        ]]);
        let mut table = PlaylistTable::new(data).columns(vec![
            Column::new("", 1, 1),
            Column::new("Col 1", 1, 5),
            Column::new("Col 2", 1, 5),
            Column::new("Col 3", 1, 5),
        ]);

        assert_eq!(table.selected(), None);
        assert_eq!(table.state.item_offset, 0);

        table.perform(Cmd::Move(Direction::Down));

        assert_eq!(table.selected(), Some(0));
        assert_eq!(table.state.item_offset, 0);
    }
}
