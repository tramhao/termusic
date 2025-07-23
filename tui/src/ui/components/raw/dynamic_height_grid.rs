use std::{cell::RefCell, num::NonZeroUsize, rc::Rc};

use lru::LruCache;
use tuirealm::ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Offset, Rect},
    widgets::{Block, Paragraph, Widget},
};

type Rects = Rc<[Rect]>;
type Cache = LruCache<(Rect, DynamicHeightGrid), Rects>;

const CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(10).unwrap();

// Cache the computations of the layout, so that we dont have to much work on re-prints if nothing changed.
thread_local! {
    static LAYOUT_CACHE: RefCell<Cache> = RefCell::new(Cache::new(
        CACHE_SIZE
    ));
}

// NOTE: the struct below could likely be changed to allow dynamic elems widths / heights, but currently only allows one size.

/// A dynamic grid where all elements are of `elem_width` and dynamic height.
/// Split automatically into rows and colums, if there is not enough space on the current row.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DynamicHeightGrid {
    elems_height: Box<[u16]>,
    elem_width: u16,
    row_spacing: u16,
    draw_row_low_space: bool,
    distribute_row_space: bool,
}

impl DynamicHeightGrid {
    pub fn new<E: Into<Box<[u16]>>>(elems: E, elem_width: u16) -> Self {
        Self {
            elems_height: elems.into(),
            elem_width,
            row_spacing: 0,
            draw_row_low_space: false,
            distribute_row_space: false,
        }
    }

    /// Set spacing between the dynamic rows.
    ///
    /// Default: `0`
    pub fn with_row_spacing(mut self, spacing: u16) -> Self {
        self.row_spacing = spacing;
        self
    }

    /// Draw the last row even if there is not enough space for a full element.
    ///
    /// Default: `false`
    pub fn draw_row_low_space(mut self) -> Self {
        self.draw_row_low_space = true;
        self
    }

    /// Distribute remaining row space among the elements in the row.
    ///
    /// Default: `false`
    pub fn distribute_row_space(mut self) -> Self {
        self.distribute_row_space = true;
        self
    }

    /// Split `area` into `elems`, will always be `elems` amount.
    /// Elements areas that dont fit are `Rect(0,0,0,0)`.
    pub fn split(&self, area: Rect) -> Rects {
        LAYOUT_CACHE.with_borrow_mut(|c| {
            let key = (area, self.clone());
            c.get_or_insert(key, || self.get_areas_inner(area)).clone()
        })
    }

    /// Internal function for [`split`](Self::split) to reduce nesting.
    fn get_areas_inner(&self, area: Rect) -> Rects {
        let mut remaining_area = area;
        let mut cells = Vec::new();

        let mut remaining_elems = self.elems_height.iter().peekable();

        let elems_per_row = remaining_area.width / self.elem_width;

        while remaining_elems
            .peek()
            .is_some_and(|v| (remaining_area.height >= **v) || self.draw_row_low_space)
            && !remaining_area.is_empty()
            && remaining_area.height > 0
        {
            // dont add a spacing for the first row
            let spacing = if remaining_area == area {
                0
            } else {
                self.row_spacing
            };
            let [_row_spacer, spaced_area] =
                Layout::vertical([Constraint::Length(spacing), Constraint::Fill(0)])
                    .areas(remaining_area);
            remaining_area = spaced_area;

            let chunks = if self.distribute_row_space {
                let constraints = (0..elems_per_row).map(|_| Constraint::Min(self.elem_width));

                Layout::horizontal(constraints).split(remaining_area)
            } else {
                let constraints = (0..elems_per_row).map(|_| Constraint::Length(self.elem_width));

                Layout::horizontal(constraints).split(remaining_area)
            };

            let mut highest_height = 0;

            for chunk in chunks.iter() {
                let mut chunk = *chunk;
                // only add as many cells as there are requested elements
                let Some(elem_height) = remaining_elems.peek().copied().copied() else {
                    break;
                };

                // dont add the chunk if the height is not enough
                if remaining_area.height < elem_height && !self.draw_row_low_space {
                    break;
                }
                // actuall advance the iterator
                remaining_elems.next();

                chunk.height = elem_height.min(remaining_area.height);
                highest_height = highest_height.max(chunk.height);

                cells.push(chunk);
            }

            remaining_area.height = remaining_area.height.saturating_sub(highest_height);
            remaining_area = remaining_area.offset(Offset {
                x: 0,
                y: i32::from(highest_height),
            });
        }

        // fill the vec to be "elems" amount with 0-width/height rects.
        for _ in remaining_elems {
            cells.push(Rect::default());
        }

        cells.into()
    }
}

/// Debug-only visualization of the areas
impl Widget for DynamicHeightGrid {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cells = self.split(area);

        for (i, cell) in cells.iter().enumerate() {
            Paragraph::new(format!("Area {:02}", i + 1))
                .block(Block::bordered())
                .render(*cell, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tuirealm::ratatui::layout::Rect;

    use super::DynamicHeightGrid;

    #[test]
    fn should_split_all_single_row_uniform() {
        let area = Rect::new(0, 0, 30, 3);
        let elems = [3, 3, 3];

        let areas = DynamicHeightGrid::new(elems, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(20, 0, 10, 3));
    }

    #[test]
    fn should_split_all_2_rows_uniform() {
        let area = Rect::new(0, 0, 20, 6);
        let elems = [3, 3, 3];

        let areas = DynamicHeightGrid::new(elems, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 3, 10, 3));
    }

    #[test]
    fn should_not_split_new_row_low_space_uniform() {
        let area = Rect::new(0, 0, 20, 3);
        let elems = [3, 3, 3];

        let areas = DynamicHeightGrid::new(elems, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 0, 0, 0));
    }

    #[test]
    fn should_split_new_row_low_space_uniform() {
        let area = Rect::new(0, 0, 20, 3);
        let elems = [3, 3, 3];

        let areas = DynamicHeightGrid::new(elems, 10)
            .draw_row_low_space()
            .split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 0, 0, 0));
    }

    #[test]
    fn should_have_row_spacing_uniform() {
        let area = Rect::new(0, 0, 20, 7);
        let elems = [3, 3, 3];

        let areas = DynamicHeightGrid::new(elems, 10)
            .with_row_spacing(1)
            .split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 4, 10, 3));
    }

    #[test]
    fn should_split_all_single_row_no_leftover_space_uniform() {
        let area = Rect::new(0, 0, 33, 3);
        let elems = [3, 3, 3];

        let areas = DynamicHeightGrid::new(elems, 10)
            .distribute_row_space()
            .split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 11, 3));
        assert_eq!(areas[1], Rect::new(11, 0, 11, 3));
        assert_eq!(areas[2], Rect::new(22, 0, 11, 3));
    }

    #[test]
    fn should_split_all_single_row() {
        let area = Rect::new(0, 0, 30, 4);
        let elems = [3, 4, 3];

        let areas = DynamicHeightGrid::new(elems, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 4));
        assert_eq!(areas[2], Rect::new(20, 0, 10, 3));
    }

    #[test]
    fn should_split_all_2_rows() {
        let area = Rect::new(0, 0, 20, 7);
        let elems = [3, 4, 3];

        let areas = DynamicHeightGrid::new(elems, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 4));
        assert_eq!(areas[2], Rect::new(0, 4, 10, 3));
    }

    #[test]
    fn should_not_overflow_given_area() {
        let area = Rect::new(0, 0, 20, 6);
        let elems = [3, 4, 3];

        let areas = DynamicHeightGrid::new(elems, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 4));
        assert_eq!(areas[2], Rect::new(0, 0, 0, 0));

        assert!(area.contains(areas[1].positions().last().unwrap()));
    }

    #[test]
    fn should_not_overflow_given_area_draw_last_row() {
        let area = Rect::new(0, 0, 20, 6);
        let elems = [3, 4, 3];

        let areas = DynamicHeightGrid::new(elems, 10)
            .draw_row_low_space()
            .split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 4));
        assert_eq!(areas[2], Rect::new(0, 4, 10, 2));

        assert!(area.contains(areas[2].positions().last().unwrap()));
    }
}
