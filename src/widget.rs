use crate::{App, header::ConnectionStatus};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, StatefulWidget,
        Widget,
    },
};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Rust Amnezia WireGuard ".bold());
        let instructions = Line::from(vec![
            " Down: ".into(),
            "<j>".blue().bold(),
            " Up: ".into(),
            "<k>".blue().bold(),
            " Quit: ".into(),
            "<q>".blue().bold(),
            " (Dis)Connect: ".into(),
            "<Enter>".blue().bold(),
        ]);

        let items: Vec<ListItem> = self
            .servers
            .iter()
            .map(|s| {
                let label = match &s.status {
                    ConnectionStatus::Connected => format!("{} ({})", s.name, s.status.as_str()),
                    ConnectionStatus::Disconnected => s.name.clone(),
                };
                ListItem::new(label)
            })
            .collect();

        let block_app = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::EMPTY);

        let block_list = Block::bordered()
            .title("Servers: ")
            .padding(Padding::new(0, 1, 1, 0))
            .border_set(border::EMPTY);
        let block_list_area = block_app.inner(area);

        let centered_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(block_list_area)[1];

        let centered_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(centered_area)[1];

        block_app.render(area, buf);

        StatefulWidget::render(
            List::new(items)
                .block(block_list)
                .highlight_style(Style::default().red().bold())
                .highlight_symbol(""),
            centered_area,
            buf,
            &mut self.list_state,
        );
        if self.show_auth_popup {
            self.render_popup(area, buf);
        }
    }
}

impl App {
    pub fn popup_rect(area: Rect) -> Rect {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(vertical_layout[1])[1]
    }
    pub fn render_popup(&self, area: Rect, buf: &mut Buffer) {
        let popup_area = App::popup_rect(area);

        Clear.render(popup_area, buf);

        let masked_password = "*".repeat(self.input_buffer.len());
        let title = Line::from("Sudo Password".bold());
        let title_bottom = Line::from(vec![
            " Enter: ".into(),
            " <Enter> ".blue().bold().into(),
            " Close: ".into(),
            " <Esc> ".blue().bold().into(),
        ]);

        let block = Block::default()
            .title(title.centered())
            .title_bottom(title_bottom)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::White));

        Paragraph::new(masked_password)
            .block(block)
            .alignment(Alignment::Center)
            .render(popup_area, buf);
    }
}
