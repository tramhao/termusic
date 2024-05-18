use crate::config::v2::tui::{Alignment, CoverArtPosition};
use anyhow::{bail, Result};
use image::DynamicImage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentWrap(Alignment);

impl AlignmentWrap {
    const fn x(&self, absolute_x: u32, width: u32) -> u32 {
        match self.0 {
            Alignment::BottomRight | Alignment::TopRight => {
                Self::get_size_substract(absolute_x, width)
            }
            Alignment::BottomLeft | Alignment::TopLeft => absolute_x,
        }
    }
    const fn y(&self, absolute_y: u32, height: u32) -> u32 {
        match self.0 {
            Alignment::BottomRight | Alignment::BottomLeft => {
                Self::get_size_substract(absolute_y, height / 2)
            }
            Alignment::TopRight | Alignment::TopLeft => absolute_y,
        }
    }

    const fn get_size_substract(absolute_size: u32, size: u32) -> u32 {
        if absolute_size > size {
            return absolute_size - size;
        }
        0
    }
}

#[derive(Clone, Debug)]
pub struct Xywh {
    pub x_between_1_100: u32,
    pub y_between_1_100: u32,
    pub width_between_1_100: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub align: AlignmentWrap,
}

impl From<&CoverArtPosition> for Xywh {
    fn from(value: &CoverArtPosition) -> Self {
        // TODO: actually apply scale
        Self {
            align: AlignmentWrap(value.align),
            ..Default::default()
        }
    }
}

impl Default for Xywh {
    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation)]
    fn default() -> Self {
        let width = 20_u32;
        let height = 20_u32;
        let (term_width, term_height) = Self::get_terminal_size_u32();
        let x = term_width - 1;
        let y = term_height - 9;

        Self {
            x_between_1_100: 100,
            y_between_1_100: 77,
            width_between_1_100: width,
            x,
            y,
            width,
            height,
            align: AlignmentWrap(Alignment::default()),
        }
    }
}
impl Xywh {
    pub fn move_left(&mut self) {
        self.x_between_1_100 = self.x_between_1_100.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        self.x_between_1_100 += 1;
        self.x_between_1_100 = self.x_between_1_100.min(100);
    }

    pub fn move_up(&mut self) {
        self.y_between_1_100 = self.y_between_1_100.saturating_sub(2);
    }

    pub fn move_down(&mut self) {
        self.y_between_1_100 += 2;
        self.y_between_1_100 = self.y_between_1_100.min(100);
    }
    pub fn zoom_in(&mut self) {
        self.width_between_1_100 += 1;
        self.width_between_1_100 = self.width_between_1_100.min(100);
    }

    pub fn zoom_out(&mut self) {
        self.width_between_1_100 = self.width_between_1_100.saturating_sub(1);
    }

    pub fn update_size(&self, image: &DynamicImage) -> Result<Self> {
        let (term_width, term_height) = Self::get_terminal_size_u32();
        let (x, y, width, height) = self.calculate_xywh(term_width, term_height, image)?;
        Ok(Self {
            x_between_1_100: self.x_between_1_100,
            y_between_1_100: self.y_between_1_100,
            width_between_1_100: self.width_between_1_100,
            x,
            y,
            width,
            height,
            align: self.align.clone(),
        })
    }
    fn calculate_xywh(
        &self,
        term_width: u32,
        term_height: u32,
        image: &DynamicImage,
    ) -> Result<(u32, u32, u32, u32)> {
        let width = self.get_width(term_width)?;
        let height = Self::get_height(width, term_height, image)?;
        let (absolute_x, absolute_y) = (
            self.x_between_1_100 * term_width / 100,
            self.y_between_1_100 * term_height / 100,
        );
        let (x, y) = (
            self.align.x(absolute_x, width),
            self.align.y(absolute_y, height),
        );
        Ok((x, y, width, height))
    }

    fn get_width(&self, term_width: u32) -> Result<u32> {
        let width = self.width_between_1_100 * term_width / 100;
        Self::safe_guard_width_or_height(width, term_width)
    }

    fn safe_guard_width_or_height(size: u32, size_max: u32) -> Result<u32> {
        if size > size_max {
            bail!("image width is too big, please reduce image width");
        }
        Ok(size)
    }

    fn get_height(width: u32, term_height: u32, image: &DynamicImage) -> Result<u32> {
        let (pic_width_orig, pic_height_orig) = image::GenericImageView::dimensions(image);
        let height = (width * pic_height_orig) / (pic_width_orig);
        Self::safe_guard_width_or_height(height, term_height * 2)
    }

    pub fn get_terminal_size_u32() -> (u32, u32) {
        let (term_width, term_height) = viuer::terminal_size();
        (u32::from(term_width), u32::from(term_height))
    }
}
