use tuirealm::{
    props::Style,
    ratatui::{
        buffer::Buffer,
        layout::{Constraint, Layout, Rect},
        text::Line,
        widgets::{Block, StatefulWidget, Widget},
    },
};

use crate::ui::components::playlist::playlist_mock::{
    ListValue, PlaylistTableContext, PlaylistTableHeaderContext, PlaylistTableState,
};

// /// Default Highlight Symbol draw width.
// pub const DEFAULT_HG_WIDTH: u16 = 2;
/// Default text to display if the Table is empty.
pub const DEFAULT_EMPTY_TABLE_TEXT: &str = "No Data";

/// The ratatui widget to draw.
///
/// Likely not what you want to use.
#[derive(Debug, Clone)]
#[must_use]
pub struct PlaylistTableWidget<'a, V: ListValue> {
    /// The main style of the tree
    main_style: Style,
    /// The Highlight Style for the currently highlighted element
    hg_style: Style,
    /// The Highlight symbol for the currently highlighted element
    hg_str: Option<Line<'a>>,
    // /// The Highlight symbol draw width
    // hg_width: u16,
    // /// The Highlight symbol draw behavior
    // hg_draw_behavior: HighlightDrawBehavior,
    /// Optional block to render around the widget itself
    block: Option<Block<'a>>,

    /// Text to display if the table is empty
    empty_text: &'a str,

    /// The Data to render
    data: &'a V,
}

impl<'a, V> PlaylistTableWidget<'a, V>
where
    V: ListValue,
{
    /// Create a new widget with the given data and otherwise default values.
    pub fn new(data: &'a V) -> Self {
        Self {
            data,
            main_style: Style::default(),
            hg_style: Style::default(),
            hg_str: None,
            // hg_width: DEFAULT_HG_WIDTH,
            // hg_draw_behavior: HighlightDrawBehavior::default(),
            block: None,
            empty_text: DEFAULT_EMPTY_TABLE_TEXT,
        }
    }

    /// Set a custom block to draw around the main widget.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);

        self
    }

    /// Set the main style of the Tree.
    pub fn style(mut self, style: Style) -> Self {
        self.main_style = style;

        self
    }

    /// Set the Highlight style of the currently selected node.
    pub fn hg_style(mut self, style: Style) -> Self {
        self.hg_style = style;

        self
    }

    /// Set the Highlight Symbol to draw in addition to the line itself.
    pub fn hg_str(mut self, val: Line<'a>) -> Self {
        self.hg_str = Some(val);

        self
    }

    // /// Set the Highlight Symbol draw width.
    // ///
    // /// By default [`DEFAULT_HG_WIDTH`].
    // pub fn hg_width(mut self, width: u16) -> Self {
    //     self.hg_width = width;

    //     self
    // }

    // /// Set the Highlight Symbol Draw behavior.
    // ///
    // /// See [`HighlightDrawBehavior`] for available and default behavior.
    // pub fn hg_draw_behavior(mut self, behavior: HighlightDrawBehavior) -> Self {
    // 	self.hg_draw_behavior = behavior;

    // 	return self;
    // }

    /// Set a empty table text.
    ///
    /// Default: [`DEFAULT_EMPTY_TABLE_TEXT`]
    pub fn empty_table_text(mut self, val: &'a str) -> Self {
        self.empty_text = val;

        self
    }
}

impl<V> StatefulWidget for PlaylistTableWidget<'_, V>
where
    V: ListValue,
{
    type State = PlaylistTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.main_style);
        // render the block, if set
        let area = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if area.is_empty() {
            return;
        }

        let mut remaining_area = area;

        // generate areas once and only modify their vertical offset and height
        // TODO: This is only a implementation to get something to draw and tries to have all columns always visible. Consider replacing with custom kasuari logic.
        let (areas, spacers) = Layout::horizontal(state.columns().iter().map(|v| {
            if v.max_size != 0 {
                Constraint::Max(v.max_size)
            } else {
                Constraint::Min(v.min_size)
            }
        }))
        .spacing(2)
        .split_with_spacers(remaining_area);
        let mut areas = areas.to_vec();
        let mut spacers = spacers.to_vec();

        // always try to draw headers
        loop {
            for area in &mut areas {
                area.y = remaining_area.y;
                area.height = remaining_area.height;
            }
            for spacer in &mut spacers {
                spacer.y = remaining_area.y;
                spacer.height = remaining_area.height;
            }
            let ctx = PlaylistTableHeaderContext {
                areas: &areas,
                areas_spacer: &spacers,
                columns: state.columns(),
            };
            let ret = self.data.render_header(buf, &ctx, self.main_style);

            // always subtract the used amount, even when indicating it is done
            if ret.consumed_vertical_size > 0 {
                let used_vert = ret.consumed_vertical_size;
                remaining_area.y = remaining_area.y.saturating_add(used_vert);
                remaining_area.height = remaining_area.height.saturating_sub(used_vert);
            }
            if ret.consumed_vertical_size == 0 || ret.done || remaining_area.is_empty() {
                break;
            }
        }

        state.set_last_size_data(remaining_area);

        // if data is empty, dont try to draw data, draw empty message instead
        if self.data.is_empty() {
            self.empty_text.render(remaining_area, buf);

            return;
        }

        // draw the actual data
        let mut item_offset = state.get_item_offset();

        while !remaining_area.is_empty() {
            for area in &mut areas {
                area.y = remaining_area.y;
                area.height = remaining_area.height;
            }
            for spacer in &mut spacers {
                spacer.y = remaining_area.y;
                spacer.height = remaining_area.height;
            }

            let is_selected = state.selected().is_some_and(|v| v == item_offset);

            let ctx = PlaylistTableContext {
                areas: &areas,
                areas_spacer: &spacers,
                item_offset,
                is_selected,
            };

            let use_style = if is_selected {
                self.hg_style
            } else {
                self.main_style
            };
            let ret = self.data.render(buf, &ctx, use_style);

            // stop if the render function did not draw anything anymore or indicates there is nothing more to draw
            if ret.consumed_vertical_size == 0 || ret.done {
                break;
            }

            let used_vert = ret.consumed_vertical_size;

            remaining_area.y = remaining_area.y.saturating_add(used_vert);
            remaining_area.height = remaining_area.height.saturating_sub(used_vert);
            item_offset += 1;
        }
    }
}
