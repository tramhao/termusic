use anyhow::Result;
use std::cell::Cell;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::{backend::Backend, Frame};
/// the main app type
pub struct App {
    requires_redraw: Cell<bool>,
}

// public interface
impl App {
    ///
    #[allow(clippy::too_many_lines)]
    pub fn new() -> Self {
        Self {
            requires_redraw: Cell::new(false),
        }
    }

    ///
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>) -> Result<()> {
        let fsize = f.size();

        let chunks_main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(3, 10), Constraint::Ratio(7, 10)].as_ref())
            .split(fsize);

        let chunks_side = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(9)].as_ref())
            .split(chunks_main[1]);
        let copyright1 = Paragraph::new("pet-CLI 2020 - all rights reserved")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Copyright")
                    .border_type(BorderType::Plain),
            );
        let copyright2 = Paragraph::new("pet-CLI 2020 - all rights reserved")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Copyright")
                    .border_type(BorderType::Plain),
            );
        let copyright3 = Paragraph::new("pet-CLI 2020 - all rights reserved")
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Copyright")
                    .border_type(BorderType::Plain),
            );

        f.render_widget(copyright1, chunks_main[0]);
        f.render_widget(copyright2, chunks_side[0]);
        f.render_widget(copyright3, chunks_side[1]);

        // self.draw_popups(f)?;

        Ok(())
    }
    ///
    pub fn requires_redraw(&self) -> bool {
        if self.requires_redraw.get() {
            self.requires_redraw.set(false);
            true
        } else {
            false
        }
    }
}

// private impls
impl App {}
