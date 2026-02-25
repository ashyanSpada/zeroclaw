use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::state::AppState;

pub fn ui(frame: &mut Frame, app: &mut AppState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let title = Paragraph::new("ZeroClaw TUI")
        .block(Block::default().title(" Dashboard ").borders(Borders::ALL));
    frame.render_widget(title, outer[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(10)])
        .split(outer[1]);

    let items: Vec<ListItem<'_>> = app
        .items
        .iter()
        .map(|item| ListItem::new(item.title()))
        .collect();
    let menu = List::new(items)
        .block(Block::default().title(" Commands ").borders(Borders::ALL))
        .highlight_symbol("› ")
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(menu, body[0], &mut app.menu);

    let output = app.output.join("\n");
    let output_widget = Paragraph::new(output)
        .block(Block::default().title(" Output ").borders(Borders::ALL));
    frame.render_widget(output_widget, body[1]);

    let footer = Paragraph::new("↑/↓ move • Enter run • q quit")
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, outer[2]);
}
