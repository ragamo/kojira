use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::{App, CreateField, PRIORITIES};

pub fn render(frame: &mut Frame, app: &App) {
    let t = app.theme;
    let screen = frame.area();
    let w = 76u16.min(screen.width);
    let h = 28u16.min(screen.height);
    let area = Rect {
        x: screen.x + screen.width.saturating_sub(w) / 2,
        y: screen.y + screen.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    };

    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.border))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    let body = chunks[0];
    let footer_area = chunks[1];
    let content_width = body.width.saturating_sub(4);
    let label_w = 13u16;
    let field_w = content_width.saturating_sub(label_w);
    let field_x = body.x + 2 + label_w;

    let modal = &app.create_modal;

    // Title
    let title_area = Rect { x: body.x + 1, y: body.y, width: body.width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(
            "New Issue",
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        )),
        title_area,
    );

    // --- Title field ---
    let y = body.y + 2;
    let label_area = Rect { x: body.x + 2, y, width: label_w, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("Title", Style::default().fg(t.text))),
        label_area,
    );

    let title_border = if modal.active_field == CreateField::Title { t.accent } else { t.border };
    let title_input_area = Rect { x: field_x, y, width: field_w, height: 3 };
    let cursor = if modal.active_field == CreateField::Title { "▌" } else { "" };
    let title_widget = Paragraph::new(format!("{}{}", modal.title, cursor))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(title_border)),
        )
        .style(Style::default().fg(t.text));
    frame.render_widget(title_widget, title_input_area);

    // --- Description field ---
    let y = y + 4;
    let label_area = Rect { x: body.x + 2, y, width: label_w, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("Description", Style::default().fg(t.text))),
        label_area,
    );

    let desc_border = if modal.active_field == CreateField::Description { t.accent } else { t.border };
    let desc_h = 5u16;
    let desc_area = Rect { x: field_x, y, width: field_w, height: desc_h };
    let desc_text: String = modal.description.join("\n");
    let desc_cursor = if modal.active_field == CreateField::Description { "▌" } else { "" };
    let display_text = if modal.active_field == CreateField::Description {
        let mut lines = modal.description.clone();
        let row = modal.description_cursor_row;
        let col = modal.description_cursor_col;
        if row < lines.len() {
            lines[row].insert_str(col, desc_cursor);
        }
        lines.join("\n")
    } else {
        desc_text
    };
    let desc_widget = Paragraph::new(display_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(desc_border)),
        )
        .style(Style::default().fg(t.text));
    frame.render_widget(desc_widget, desc_area);

    // --- Assignee field ---
    let y = y + desc_h + 1;
    let label_area = Rect { x: body.x + 2, y, width: label_w, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("Assignee", Style::default().fg(t.text))),
        label_area,
    );

    let assignee_border = if modal.active_field == CreateField::Assignee { t.accent } else { t.border };
    let assignee_area = Rect { x: field_x, y, width: field_w, height: 3 };
    let assignee_text = if modal.loading_assignees {
        "Loading...".to_string()
    } else if let Some(idx) = modal.assignee_idx {
        modal.assignees.get(idx).map(|u| u.display_name.clone()).unwrap_or_default()
    } else {
        "Select...".to_string()
    };
    let assignee_widget = Paragraph::new(Span::styled(&assignee_text, Style::default().fg(t.text)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(assignee_border)),
        );
    frame.render_widget(assignee_widget, assignee_area);

    // Assignee dropdown
    if modal.list_open && modal.active_field == CreateField::Assignee {
        render_dropdown(frame, t, field_x, y + 3, field_w, &modal.assignees.iter().map(|u| u.display_name.as_str()).collect::<Vec<_>>(), modal.list_scroll);
    }

    // --- Epic field ---
    let y = y + 4;
    let label_area = Rect { x: body.x + 2, y, width: label_w, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("Epic", Style::default().fg(t.text))),
        label_area,
    );

    let epic_border = if modal.active_field == CreateField::Epic { t.accent } else { t.border };
    let epic_area = Rect { x: field_x, y, width: field_w, height: 3 };
    let epic_text = if modal.loading_epics {
        "Loading...".to_string()
    } else if let Some(idx) = modal.epic_idx {
        modal.epics.get(idx).map(|e| format!("{} - {}", e.key, e.fields.summary)).unwrap_or_default()
    } else {
        "Select...".to_string()
    };
    let epic_widget = Paragraph::new(Span::styled(&epic_text, Style::default().fg(t.text)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(epic_border)),
        );
    frame.render_widget(epic_widget, epic_area);

    if modal.list_open && modal.active_field == CreateField::Epic {
        let items: Vec<&str> = modal.epics.iter().map(|e| e.fields.summary.as_str()).collect();
        render_dropdown(frame, t, field_x, y + 3, field_w, &items, modal.list_scroll);
    }

    // --- Priority field ---
    let y = y + 4;
    let label_area = Rect { x: body.x + 2, y, width: label_w, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("Priority", Style::default().fg(t.text))),
        label_area,
    );

    let priority_border = if modal.active_field == CreateField::Priority { t.accent } else { t.border };
    let priority_area = Rect { x: field_x, y, width: field_w, height: 3 };
    let priority_widget = Paragraph::new(Span::styled(
        PRIORITIES[modal.priority_idx],
        Style::default().fg(t.text),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(priority_border)),
    );
    frame.render_widget(priority_widget, priority_area);

    if modal.list_open && modal.active_field == CreateField::Priority {
        render_dropdown(frame, t, field_x, y + 3, field_w, &PRIORITIES.iter().copied().collect::<Vec<_>>(), modal.list_scroll);
    }

    // --- Buttons ---
    let y = y + 4;
    let save_style = if modal.active_field == CreateField::Save {
        Style::default().fg(t.bg).bg(t.accent)
    } else {
        Style::default().fg(t.text).bg(t.border)
    };
    let cancel_style = if modal.active_field == CreateField::Cancel {
        Style::default().fg(t.bg).bg(t.accent)
    } else {
        Style::default().fg(t.text).bg(t.border)
    };

    let btn_x = field_x + field_w / 2 - 10;
    let save_area = Rect { x: btn_x, y, width: 8, height: 1 };
    frame.render_widget(Paragraph::new(Span::styled("  Save  ", save_style)), save_area);

    let cancel_area = Rect { x: btn_x + 10, y, width: 10, height: 1 };
    frame.render_widget(Paragraph::new(Span::styled("  Cancel  ", cancel_style)), cancel_area);

    // --- Error ---
    if let Some(ref err) = modal.error {
        let err_area = Rect { x: body.x + 2, y: y + 1, width: content_width, height: 1 };
        frame.render_widget(
            Paragraph::new(err.as_str())
                .style(Style::default().fg(t.error))
                .wrap(Wrap { trim: true }),
            err_area,
        );
    }

    // --- Footer ---
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Tab", Style::default().fg(t.accent)),
        Span::styled(" navigate  ", Style::default().fg(t.text_dim)),
        Span::styled(" ↵ select/submit ", Style::default().fg(t.bg).bg(t.accent)),
        Span::raw(" "),
        Span::styled(" esc close ", Style::default().fg(t.text).bg(t.border)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, footer_area);
}

fn render_dropdown(frame: &mut Frame, t: &crate::theme::Theme, x: u16, y: u16, w: u16, items: &[&str], selected: usize) {
    let max_visible = 5usize;
    let count = items.len().min(max_visible);
    let h = count as u16 + 2;
    let dropdown_area = Rect { x, y, width: w, height: h };

    frame.render_widget(Clear, dropdown_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(dropdown_area);
    frame.render_widget(block, dropdown_area);

    let scroll_start = if selected >= max_visible {
        selected - max_visible + 1
    } else {
        0
    };

    for (i, idx) in (scroll_start..items.len().min(scroll_start + max_visible)).enumerate() {
        let item_area = Rect { x: inner.x, y: inner.y + i as u16, width: inner.width, height: 1 };
        let style = if idx == selected {
            Style::default().fg(t.bg).bg(t.accent)
        } else {
            Style::default().fg(t.text)
        };
        let text = if idx < items.len() { items[idx] } else { "" };
        frame.render_widget(Paragraph::new(Span::styled(text, style)), item_area);
    }
}
