use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

const AUTHOR_COLORS: &[(u8, u8, u8)] = &[
    (97, 175, 239),   // blue
    (229, 192, 123),  // yellow
    (152, 195, 121),  // green
    (224, 108, 117),  // red
    (198, 120, 221),  // purple
    (86, 182, 194),   // cyan
    (255, 121, 198),  // pink
    (189, 147, 249),  // violet
    (241, 250, 140),  // lime
    (139, 233, 253),  // sky
    (255, 183, 77),   // orange
    (128, 203, 196),  // teal
];

const EPIC_COLORS: &[(u8, u8, u8)] = &[
    (136, 71, 209),
    (0, 135, 90),
    (192, 108, 22),
    (30, 102, 245),
    (195, 55, 100),
    (56, 132, 0),
    (165, 97, 24),
    (23, 146, 153),
    (120, 71, 189),
    (180, 52, 52),
];

fn color_for(name: &str, palette: &[(u8, u8, u8)]) -> Color {
    let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let idx = (hash as usize) % palette.len();
    let (r, g, b) = palette[idx];
    Color::Rgb(r, g, b)
}

pub fn render(frame: &mut Frame, app: &mut App, board_id: u64, area: Rect) {
    let t = app.theme;

    let tab = match app.board_tabs.iter().find(|t| t.board_id == board_id) {
        Some(t) => t,
        None => return,
    };

    if tab.loading {
        let loading = Paragraph::new("Loading board...")
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
        return;
    }

    if let Some(ref err) = tab.error {
        let msg = format!("Error: {} — press r to retry", err);
        let error = Paragraph::new(msg)
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(error, area);
        return;
    }

    if tab.columns.is_empty() {
        let empty = Paragraph::new("No columns configured")
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let hide_subtasks = app.board_hide_subtasks;
    let hide_backlog = app.board_hide_backlog_col;

    let visible_columns: Vec<&_> = tab
        .columns
        .iter()
        .filter(|col| !(hide_backlog && col.name.eq_ignore_ascii_case("backlog")))
        .collect();

    let col_count = visible_columns.len() as u16;
    let col_width = area.width / col_count.max(1);

    for (i, col) in visible_columns.iter().enumerate() {
        let x = area.x + (i as u16) * col_width;
        let w = if i as u16 == col_count - 1 {
            area.width - (i as u16) * col_width
        } else {
            col_width
        };
        let col_area = Rect { x, y: area.y, width: w, height: area.height };

        let visible_issues: Vec<&_> = col
            .issues
            .iter()
            .filter(|issue| {
                if hide_subtasks && issue.fields.issue_type.name.eq_ignore_ascii_case("sub-task") {
                    return false;
                }
                true
            })
            .collect();

        let title = format!(" {} ({}) ", col.name, visible_issues.len());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.border))
            .title(title)
            .title_style(Style::default().fg(t.text).add_modifier(Modifier::BOLD));
        let inner = block.inner(col_area);
        frame.render_widget(block, col_area);

        let mut y_offset = 0u16;

        for issue in &visible_issues {
            let content_width = inner.width as usize;

            // Row 1+: Summary (wrapped)
            let summary_lines = wrap_str(&issue.fields.summary, content_width);
            let card_height = (summary_lines.len() as u16) + 2; // summary + epic + key/assignee

            if y_offset + card_height > inner.height {
                break;
            }

            let card_area = Rect {
                x: inner.x,
                y: inner.y + y_offset,
                width: inner.width,
                height: card_height,
            };

            let mut lines: Vec<Line> = summary_lines
                .iter()
                .map(|s| Line::from(Span::styled(s.as_str(), Style::default().fg(t.text))))
                .collect();

            // Row 2: Epic with background color
            let line2 = if let Some(ref parent) = issue.fields.parent {
                let epic_name = parent
                    .fields
                    .as_ref()
                    .map(|f| f.summary.as_str())
                    .unwrap_or(&parent.key);
                let bg = color_for(epic_name, EPIC_COLORS);
                let display = format!(" {} ", truncate_str(epic_name, content_width.saturating_sub(2)));
                Line::from(Span::styled(display, Style::default().fg(Color::White).bg(bg)))
            } else {
                Line::from(Span::styled("", Style::default().fg(t.text_dim)))
            };

            // Row 3: Key left, Assignee right
            let key_span = Span::styled(&issue.key, Style::default().fg(t.accent));
            let assignee_name = issue
                .fields
                .assignee
                .as_ref()
                .map(|a| a.display_name.as_str())
                .unwrap_or("");
            let assignee_color = if assignee_name.is_empty() {
                t.text_dim
            } else {
                color_for(assignee_name, AUTHOR_COLORS)
            };

            let key_len = issue.key.len();
            let available = content_width.saturating_sub(key_len + 1);
            let assignee_display = truncate_str(assignee_name, available);
            let padding = content_width.saturating_sub(key_len + assignee_display.len());
            let line3 = Line::from(vec![
                key_span,
                Span::raw(" ".repeat(padding)),
                Span::styled(assignee_display, Style::default().fg(assignee_color)),
            ]);

            lines.push(line2);
            lines.push(line3);

            let card = Paragraph::new(lines);
            frame.render_widget(card, card_area);

            y_offset += card_height + 1;
        }
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}

fn wrap_str(s: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        return vec![s.to_string()];
    }
    let mut lines = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + width).min(chars.len());
        let line: String = chars[start..end].iter().collect();
        lines.push(line);
        start = end;
    }
    lines
}
