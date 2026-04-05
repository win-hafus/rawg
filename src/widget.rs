use crate::{App, header::ConnectionStatus};

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, StatefulWidget, Widget,
    },
};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Три зоны как в file_explorer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Заголовок
                Constraint::Min(0),    // Список серверов
                Constraint::Length(3), // Подсказки
            ])
            .split(area);

        // --- Заголовок ---
        let title_widget = Paragraph::new(" Rust Amnezia WireGuard ")
            .block(Block::default().borders(Borders::ALL).title(" RAWG "))
            .style(Style::default().bold());
        title_widget.render(chunks[0], buf);

        // --- Список серверов ---
        let items: Vec<ListItem> = self
            .servers
            .iter()
            .map(|s| match s.status {
                ConnectionStatus::Connected => ListItem::new(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(s.name.clone(), Style::default().fg(Color::White)),
                    Span::raw(" — "),
                    Span::styled(
                        "Connected",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])),
                ConnectionStatus::Disconnected => ListItem::new(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(s.name.clone(), Style::default().fg(Color::DarkGray)),
                ])),
            })
            .collect();

        StatefulWidget::render(
            List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Servers "))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("*"),
            chunks[1],
            buf,
            &mut self.list_state,
        );

        // --- Подсказки ---
        let help_text = if let Some(msg) = &self.status_message {
            format!(" ⚠ {}  <Esc>: Dismiss", msg)
        } else {
            "<J> <K> Navigate  <Enter>: (Dis)Connect  <A>: Add  <D>: Delete  <Q>: Quit".to_string()
        };

        let help_style = if self.status_message.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Gray)
        };

        Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL))
            .style(help_style)
            .render(chunks[2], buf);
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
        let title = Line::from(" Sudo Password ".bold());
        let title_bottom = Line::from(vec![
            " Confirm: ".into(),
            "<Enter>".blue().bold(),
            " Cancel: ".into(),
            "<Esc>".blue().bold(),
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
