use crate::file_explorer::FileExplorer;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render_file_explorer(f: &mut Frame, area: Rect, explorer: &FileExplorer) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Текущий путь
            Constraint::Min(0),    // Список файлов
            Constraint::Length(3), // Подсказки
        ])
        .split(area);

    // --- Текущий путь ---
    let path_text = explorer.current_dir.to_string_lossy().to_string();
    let path_widget =
        Paragraph::new(path_text).block(Block::default().borders(Borders::ALL).title("  Path "));
    f.render_widget(path_widget, chunks[0]);

    // --- Список файлов ---
    let visible_height = chunks[1].height as usize - 2; // минус бордеры
    let items: Vec<ListItem> = explorer
        .entries
        .iter()
        .enumerate()
        .skip(explorer.scroll_offset)
        .take(visible_height)
        .map(|(i, entry)| {
            let is_selected = i == explorer.selected;
            let (icon, style) = if entry.is_dir {
                (" ", Style::default().fg(Color::Blue))
            } else if entry.is_conf {
                ("⚙️  ", Style::default().fg(Color::Green))
            } else {
                ("   ", Style::default().fg(Color::Gray))
            };

            let style = if is_selected {
                style.add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                style
            };

            ListItem::new(Line::from(vec![
                Span::raw(icon),
                Span::styled(&entry.name, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" AmneziaWG Config Selector "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = ListState::default();
    state.select(Some(
        explorer.selected.saturating_sub(explorer.scroll_offset),
    ));

    f.render_stateful_widget(list, chunks[1], &mut state);

    // --- Подсказки ---
    let help = Paragraph::new("<J> <K> Navigate  <Enter>: Open/Select  <Esc>: Cancel")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}
