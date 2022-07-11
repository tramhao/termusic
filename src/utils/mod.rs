use lazy_static::lazy_static;
use pinyin::ToPinyin;
use regex::Regex;
use std::path::Path;
use tuirealm::props::Color;
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};

lazy_static! {
    /**
     * Regex matches:
     * - group 1: Red
     * - group 2: Green
     * - group 3: Blue
     */
    static ref COLOR_HEX_REGEX: Regex = Regex::new(r"#(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})(:?[0-9a-fA-F]{2})").unwrap();
}

pub fn get_pin_yin(input: &str) -> String {
    let mut b = String::new();
    for (index, f) in input.to_pinyin().enumerate() {
        match f {
            Some(p) => {
                b.push_str(p.plain());
            }
            None => {
                if let Some(c) = input.to_uppercase().chars().nth(index) {
                    b.push(c);
                }
            }
        }
    }
    b
}

pub fn parse_hex_color(color: &str) -> Option<Color> {
    COLOR_HEX_REGEX.captures(color).map(|groups| {
        Color::Rgb(
            u8::from_str_radix(groups.get(1).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(2).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
            u8::from_str_radix(groups.get(3).unwrap().as_str(), 16)
                .ok()
                .unwrap(),
        )
    })
}

pub fn filetype_supported(current_node: &str) -> bool {
    let p = Path::new(current_node);

    #[cfg(any(feature = "mpv", feature = "gst"))]
    if let Some(ext) = p.extension() {
        if ext == "opus" {
            return true;
        }
        if ext == "aiff" {
            return true;
        }
        if ext == "webm" {
            return true;
        }
    }

    match p.extension() {
        Some(ext) if ext == "mp3" => true,
        // Some(ext) if ext == "aiff" => true,
        Some(ext) if ext == "flac" => true,
        Some(ext) if ext == "m4a" => true,
        // Some(ext) if ext == "opus" => true,
        Some(ext) if ext == "ogg" => true,
        Some(ext) if ext == "wav" => true,
        // Some(ext) if ext == "webm" => true,
        Some(_) | None => false,
    }
}

pub fn is_playlist(current_node: &str) -> bool {
    let p = Path::new(current_node);

    match p.extension() {
        Some(ext) if ext == "m3u" => true,
        Some(ext) if ext == "m3u8" => true,
        Some(ext) if ext == "pls" => true,
        Some(ext) if ext == "asx" => true,
        Some(ext) if ext == "xspf" => true,
        Some(_) | None => false,
    }
}

///
/// Get block
// pub fn get_block<'a>(props: &Borders, title: (String, Alignment), focus: bool) -> Block<'a> {
//     Block::default()
//         .borders(props.sides)
//         .border_style(if focus {
//             props.style()
//         } else {
//             Style::default().fg(Color::Reset).bg(Color::Reset)
//         })
//         .border_type(props.modifiers)
//         .title(title.0)
//         .title_alignment(title.1)
// }

// Draw an area (WxH / 3) in the middle of the parent area
pub fn draw_area_in_relative(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Percentage(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - width) / 2),
                Constraint::Percentage(width),
                Constraint::Percentage((100 - width) / 2),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

pub fn draw_area_in_absolute(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((parent.height - height) / 2),
                Constraint::Length(height),
                Constraint::Length((parent.height - height) / 2),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((parent.width - width) / 2),
                Constraint::Length(width),
                Constraint::Length((parent.width - width) / 2),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

pub fn draw_area_top_right_absolute(parent: Rect, width: u16, height: u16) -> Rect {
    let new_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(height),
                Constraint::Length(parent.height - height - 1),
            ]
            .as_ref(),
        )
        .split(parent);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(parent.width - width - 1),
                Constraint::Length(width),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(new_area[1])[1]
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {

    use crate::utils::get_pin_yin;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pin_yin() {
        assert_eq!(get_pin_yin("陈一发儿"), "chenyifaer".to_string());
        assert_eq!(get_pin_yin("Gala乐队"), "GALAledui".to_string());
        assert_eq!(get_pin_yin("乐队Gala乐队"), "leduiGALAledui".to_string());
        assert_eq!(get_pin_yin("Annett Louisan"), "ANNETT LOUISAN".to_string());
    }

    use super::*;

    #[test]
    fn test_utils_ui_draw_area_in() {
        let area: Rect = Rect::new(0, 0, 1024, 512);
        let child: Rect = draw_area_in_relative(area, 75, 30);
        assert_eq!(child.x, 43);
        assert_eq!(child.y, 63);
        assert_eq!(child.width, 271);
        assert_eq!(child.height, 54);
    }
}
