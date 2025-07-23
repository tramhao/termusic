use std::{cell::RefCell, num::NonZeroUsize, rc::Rc};

use lru::LruCache;
use tuirealm::ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, Widget},
};

type Rects = Rc<[Rect]>;
type Cache = LruCache<(Rect, UniformDynamicGrid), Rects>;

const CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(10).unwrap();

// Cache the computations of the layout, so that we dont have to much work on re-prints if nothing changed.
thread_local! {
    static LAYOUT_CACHE: RefCell<Cache> = RefCell::new(Cache::new(
        CACHE_SIZE
    ));
}

// NOTE: the struct below could likely be changed to allow dynamic elems widths / heights, but currently only allows one size.

/// A dynamic grid where all elements are of `elem_height` and `elem_width`.
/// Split automatically into rows and colums, if there is not enough space on the current row.
#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
pub struct UniformDynamicGrid {
    elems: usize,
    elem_width: u16,
    elem_height: u16,
    row_spacing: u16,
    draw_row_low_space: bool,
    distribute_row_space: bool,
}

impl UniformDynamicGrid {
    pub fn new(elems: usize, elem_height: u16, elem_width: u16) -> Self {
        Self {
            elems,
            elem_width,
            elem_height,
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
            let key = (area, *self);
            c.get_or_insert(key, || self.get_areas_inner(area)).clone()
        })
    }

    /// Internal function for [`split`](Self::split) to reduce nesting.
    fn get_areas_inner(&self, area: Rect) -> Rects {
        let mut remaining_area = area;
        let mut cells = Vec::new();

        let mut remaining_elems = 0..self.elems;

        let elems_per_row = remaining_area.width / self.elem_width;

        while !remaining_elems.is_empty()
            && !remaining_area.is_empty()
            && (remaining_area.height >= self.elem_height || self.draw_row_low_space)
        {
            // dont add a spacing for the first row
            let spacing = if remaining_area == area {
                0
            } else {
                self.row_spacing
            };
            let [_row_spacer, row_area, remainder] = Layout::vertical([
                Constraint::Length(spacing),
                Constraint::Length(self.elem_height),
                Constraint::Fill(0),
            ])
            .areas(remaining_area);
            remaining_area = remainder;

            let chunks = if self.distribute_row_space {
                let constraints = (0..elems_per_row).map(|_| Constraint::Min(self.elem_width));

                Layout::horizontal(constraints).split(row_area)
            } else {
                let constraints = (0..elems_per_row).map(|_| Constraint::Length(self.elem_width));

                Layout::horizontal(constraints).split(row_area)
            };

            for chunk in chunks.iter() {
                // only add as many cells as there are requested elements
                if remaining_elems.next().is_none() {
                    break;
                }

                cells.push(*chunk);
            }
        }

        // fill the vec to be "elems" amount with 0-width/height rects.
        for _ in remaining_elems {
            cells.push(Rect::default());
        }

        cells.into()
    }
}

/// Debug-only visualization of the areas
impl Widget for UniformDynamicGrid {
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

    use super::UniformDynamicGrid;

    #[test]
    fn should_split_all_single_row() {
        let area = Rect::new(0, 0, 30, 3);

        let areas = UniformDynamicGrid::new(3, 3, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(20, 0, 10, 3));
    }

    #[test]
    fn should_split_all_2_rows() {
        let area = Rect::new(0, 0, 20, 6);

        let areas = UniformDynamicGrid::new(3, 3, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 3, 10, 3));
    }

    #[test]
    fn should_not_split_new_row_low_space() {
        let area = Rect::new(0, 0, 20, 3);

        let areas = UniformDynamicGrid::new(3, 3, 10).split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 0, 0, 0));
    }

    #[test]
    fn should_split_new_row_low_space() {
        let area = Rect::new(0, 0, 20, 3);

        let areas = UniformDynamicGrid::new(3, 3, 10)
            .draw_row_low_space()
            .split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 0, 0, 0));
    }

    #[test]
    fn should_have_row_spacing() {
        let area = Rect::new(0, 0, 20, 7);

        let areas = UniformDynamicGrid::new(3, 3, 10)
            .with_row_spacing(1)
            .split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 10, 3));
        assert_eq!(areas[1], Rect::new(10, 0, 10, 3));
        assert_eq!(areas[2], Rect::new(0, 4, 10, 3));
    }

    #[test]
    fn should_split_all_single_row_no_leftover_space() {
        let area = Rect::new(0, 0, 33, 3);

        let areas = UniformDynamicGrid::new(3, 3, 10)
            .distribute_row_space()
            .split(area);

        assert_eq!(areas.len(), 3);
        assert_eq!(areas[0], Rect::new(0, 0, 11, 3));
        assert_eq!(areas[1], Rect::new(11, 0, 11, 3));
        assert_eq!(areas[2], Rect::new(22, 0, 11, 3));
    }
}
