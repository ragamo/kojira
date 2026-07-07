use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let t = app.theme;
    let screen = frame.area();

    let width = 40u16.min(screen.width.saturating_sub(4));
    let list_len = app.assignee_filter_list.len() as u16;
    let height = (list_len + 2).min(screen.height.saturating_sub(4)); // +2 for border
    let x = (screen.width.saturating_sub(width)) / 2;
    let y = (screen.height.saturating_sub(height)) / 2;
    let area = Rect::new(x, y, width, height);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Filter by Assignee ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));
    frame.render_widget(block, area);

    app.click_regions.assignee_modal.bounds = Some(area);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), area.height.saturating_sub(2));
    let mut item_areas = Vec::new();

    for (i, item) in app.assignee_filter_list.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let row_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        item_areas.push(row_area);

        let is_selected = i == app.assignee_filter_selected;
        let is_active = match (&app.assignee_filter_active, item) {
            (None, None) => true,
            (Some(active), Some(name)) => active == name,
            _ => false,
        };

        let label = match item {
            None => " All ".to_string(),
            Some(name) => format!(" {} ", name),
        };

        let style = if is_selected {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text)
        };

        let paragraph = Paragraph::new(label).style(style);
        frame.render_widget(paragraph, row_area);
    }

    app.click_regions.assignee_modal.item_areas = item_areas;
}
