use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let t = app.theme;
    let screen = frame.area();
    let w = 50u16.min(screen.width);
    let h = 16u16.min(screen.height);
    let area = Rect {
        x: screen.x + screen.width.saturating_sub(w) / 2,
        y: screen.y + screen.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    };

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Select Board ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    if app.board_picker_loading {
        frame.render_widget(
            Paragraph::new("Loading boards...")
                .style(Style::default().fg(t.text_dim)),
            chunks[1],
        );
    } else if app.board_picker_boards.is_empty() {
        frame.render_widget(
            Paragraph::new("No boards found for this project")
                .style(Style::default().fg(t.text_dim)),
            chunks[1],
        );
    } else {
        let items: Vec<ListItem> = app
            .board_picker_boards
            .iter()
            .enumerate()
            .map(|(i, board)| {
                let is_selected = i == app.board_picker_selected;
                let already_open = app.board_tabs.iter().any(|t| t.board_id == board.id);
                let style = if is_selected {
                    Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(t.text)
                };
                let prefix = if is_selected { " ▸ " } else { "   " };
                let suffix = if already_open { " (open)" } else { "" };
                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(t.accent)),
                    Span::styled(&board.name, style),
                    Span::styled(
                        format!(" [{}]", board.board_type),
                        Style::default().fg(t.text_dim),
                    ),
                    Span::styled(suffix, Style::default().fg(t.success)),
                ]);
                ListItem::new(line)
            })
            .collect();

        frame.render_widget(List::new(items), chunks[1]);
    }

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ↵", Style::default().fg(t.accent)),
        Span::styled(" select  ", Style::default().fg(t.text_dim)),
        Span::styled("Esc", Style::default().fg(t.accent)),
        Span::styled(" close", Style::default().fg(t.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}
