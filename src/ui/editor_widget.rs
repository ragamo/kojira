use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::SimpleEditor;
use crate::theme::Theme;

pub fn render_editor(frame: &mut Frame, editor: &mut SimpleEditor, area: Rect, t: &Theme, focused: bool) {
    let border_color = if focused { t.accent } else { t.border };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height as usize;
    if visible_height == 0 {
        return;
    }

    let cursor_row = editor.cursor_row;

    if focused {
        if cursor_row < editor.scroll as usize {
            editor.scroll = cursor_row as u16;
        } else if cursor_row >= editor.scroll as usize + visible_height {
            editor.scroll = (cursor_row + 1 - visible_height) as u16;
        }
    }

    let scroll = editor.scroll as usize;
    let lines_to_render: Vec<Line> = editor
        .lines
        .iter()
        .enumerate()
        .skip(scroll)
        .take(visible_height)
        .map(|(row_idx, line)| {
            let is_cursor_row = focused && row_idx == cursor_row;
            if is_cursor_row {
                let col = editor.cursor_col.min(line.chars().count());
                let before: String = line.chars().take(col).collect();
                let cursor_char: String = line.chars().nth(col).map(|c| c.to_string()).unwrap_or(" ".to_string());
                let after: String = line.chars().skip(col + 1).collect();
                Line::from(vec![
                    Span::styled(before, Style::default().fg(t.text)),
                    Span::styled(cursor_char, Style::default().fg(t.bg).bg(t.accent)),
                    Span::styled(after, Style::default().fg(t.text)),
                ])
            } else {
                Line::from(Span::styled(line.as_str(), Style::default().fg(t.text)))
            }
        })
        .collect();

    frame.render_widget(Paragraph::new(lines_to_render), inner);
}
