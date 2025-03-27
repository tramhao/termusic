use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};

// /// Get block
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

/// Draw an area (`WxH / 3`) in the middle of the parent area
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
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_utils_ui_draw_area_in() {
        let area: Rect = Rect::new(0, 0, 1024, 512);
        let child: Rect = draw_area_in_relative(area, 75, 30);
        assert_eq!(child, Rect::new(123, 179, 768, 154));
    }
}
