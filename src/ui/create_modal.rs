use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::{App, CreateField, PRIORITIES};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Close button [X]
    let close_x = area.x + area.width.saturating_sub(5);
    let close_area = Rect { x: close_x, y: area.y, width: 3, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("[X]", Style::default().fg(t.text_dim))),
        close_area,
    );
    app.create_close_area = Some(close_area);

    // Layout: body + footer
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    let body = chunks[0];
    let footer_area = chunks[1];

    // Split body into left and right columns
    let cols = Layout::horizontal([
        Constraint::Min(20),
        Constraint::Length(34),
    ])
    .split(body);

    let left = cols[0];
    let right = cols[1];

    // Clear field areas for this frame
    app.create_field_areas.clear();
    app.create_dropdown_areas.clear();

    // === LEFT COLUMN: Title (labeled "Create") + Description ===

    let title_border = if app.create_modal.active_field == CreateField::Title { t.accent } else { t.border };
    let title_area = Rect { x: left.x + 1, y: left.y, width: left.width.saturating_sub(2), height: 3 };
    let cursor = if app.create_modal.active_field == CreateField::Title { "▌" } else { "" };
    let title_widget = Paragraph::new(format!("{}{}", app.create_modal.title, cursor))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(title_border))
                .title(" title ")
                .title_style(Style::default().fg(t.text_dim)),
        )
        .style(Style::default().fg(t.text));
    frame.render_widget(title_widget, title_area);
    app.create_field_areas.push((title_area, CreateField::Title));

    // Description
    let desc_y = left.y + 4;
    let desc_h = left.height.saturating_sub(5);
    let desc_area = Rect { x: left.x + 1, y: desc_y, width: left.width.saturating_sub(2), height: desc_h };
    let focused = app.create_modal.active_field == CreateField::Description;
    render_editor_with_label(frame, &mut app.create_modal.description_editor, desc_area, t, focused, "description");
    app.create_field_areas.push((desc_area, CreateField::Description));

    // === RIGHT COLUMN: IssueType, Priority, Epic, Assignee ===
    let right_inner_x = right.x + 1;
    let right_inner_w = right.width.saturating_sub(2);

    // Issue Type
    let issue_type_border = if app.create_modal.active_field == CreateField::IssueType { t.accent } else { t.border };
    let issue_type_area = Rect { x: right_inner_x, y: right.y, width: right_inner_w, height: 3 };
    let issue_type_text = if app.create_modal.loading_issue_types {
        "Loading...".to_string()
    } else {
        app.create_modal.issue_types
            .get(app.create_modal.issue_type_idx)
            .cloned()
            .unwrap_or_else(|| "Task".to_string())
    };
    let issue_type_widget = Paragraph::new(Span::styled(&issue_type_text, Style::default().fg(t.text)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(issue_type_border))
                .title(" type ")
                .title_style(Style::default().fg(t.text_dim)),
        );
    frame.render_widget(issue_type_widget, issue_type_area);
    app.create_field_areas.push((issue_type_area, CreateField::IssueType));

    // Priority
    let priority_y = right.y + 4;
    let priority_border = if app.create_modal.active_field == CreateField::Priority { t.accent } else { t.border };
    let priority_area = Rect { x: right_inner_x, y: priority_y, width: right_inner_w, height: 3 };
    let priority_widget = Paragraph::new(Span::styled(
        PRIORITIES[app.create_modal.priority_idx],
        Style::default().fg(t.text),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(priority_border))
            .title(" priority ")
            .title_style(Style::default().fg(t.text_dim)),
    );
    frame.render_widget(priority_widget, priority_area);
    app.create_field_areas.push((priority_area, CreateField::Priority));

    // Epic
    let epic_y = priority_y + 4;
    let epic_border = if app.create_modal.active_field == CreateField::Epic { t.accent } else { t.border };
    let epic_area = Rect { x: right_inner_x, y: epic_y, width: right_inner_w, height: 3 };
    let epic_text = if app.create_modal.loading_epics {
        "Loading...".to_string()
    } else if let Some(idx) = app.create_modal.epic_idx {
        app.create_modal.epics.get(idx).map(|e| format!("{} - {}", e.key, e.fields.summary)).unwrap_or_default()
    } else {
        "Select...".to_string()
    };
    let epic_widget = Paragraph::new(Span::styled(&epic_text, Style::default().fg(t.text)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(epic_border))
                .title(" epic ")
                .title_style(Style::default().fg(t.text_dim)),
        );
    frame.render_widget(epic_widget, epic_area);
    app.create_field_areas.push((epic_area, CreateField::Epic));

    // Assignee
    let assignee_y = epic_y + 4;
    let assignee_border = if app.create_modal.active_field == CreateField::Assignee { t.accent } else { t.border };
    let assignee_area = Rect { x: right_inner_x, y: assignee_y, width: right_inner_w, height: 3 };
    let assignee_text = if app.create_modal.loading_assignees {
        "Loading...".to_string()
    } else if let Some(idx) = app.create_modal.assignee_idx {
        app.create_modal.assignees.get(idx).map(|u| u.display_name.clone()).unwrap_or_default()
    } else {
        "Select...".to_string()
    };
    let assignee_widget = Paragraph::new(Span::styled(&assignee_text, Style::default().fg(t.text)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(assignee_border))
                .title(" assignee ")
                .title_style(Style::default().fg(t.text_dim)),
        );
    frame.render_widget(assignee_widget, assignee_area);
    app.create_field_areas.push((assignee_area, CreateField::Assignee));

    // Buttons: Save | Cancel
    let btn_y = assignee_y + 4;
    let btn_w = (right_inner_w / 2).saturating_sub(1);
    let save_active = app.create_modal.active_field == CreateField::Save;
    let cancel_active = app.create_modal.active_field == CreateField::Cancel;
    let save_style = if save_active {
        Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text).bg(t.border)
    };
    let cancel_style = if cancel_active {
        Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text).bg(t.border)
    };
    let save_area = Rect { x: right_inner_x, y: btn_y, width: btn_w, height: 1 };
    let cancel_area = Rect { x: right_inner_x + btn_w + 1, y: btn_y, width: btn_w, height: 1 };
    frame.render_widget(Paragraph::new(Span::styled(" save ", save_style)).alignment(Alignment::Center), save_area);
    frame.render_widget(Paragraph::new(Span::styled(" cancel ", cancel_style)).alignment(Alignment::Center), cancel_area);
    app.create_field_areas.push((save_area, CreateField::Save));
    app.create_field_areas.push((cancel_area, CreateField::Cancel));

    // Error message
    if let Some(ref err) = app.create_modal.error {
        let err_y = btn_y + 2;
        let err_area = Rect { x: right_inner_x, y: err_y, width: right_inner_w, height: 1 };
        frame.render_widget(
            Paragraph::new(err.as_str()).style(Style::default().fg(t.error)),
            err_area,
        );
    }

    // Dropdown overlay (rendered last to be on top)
    if app.create_modal.list_open {
        let (dropdown_x, dropdown_y, dropdown_w, items): (u16, u16, u16, Vec<String>) = match app.create_modal.active_field {
            CreateField::Assignee => (
                assignee_area.x,
                assignee_area.y + assignee_area.height,
                assignee_area.width,
                app.create_modal.assignees.iter().map(|u| u.display_name.clone()).collect(),
            ),
            CreateField::Epic => (
                epic_area.x,
                epic_area.y + epic_area.height,
                epic_area.width,
                app.create_modal.epics.iter().map(|e| format!("{} - {}", e.key, e.fields.summary)).collect(),
            ),
            CreateField::Priority => (
                priority_area.x,
                priority_area.y + priority_area.height,
                priority_area.width,
                PRIORITIES.iter().map(|s| s.to_string()).collect(),
            ),
            CreateField::IssueType => (
                issue_type_area.x,
                issue_type_area.y + issue_type_area.height,
                issue_type_area.width,
                app.create_modal.issue_types.clone(),
            ),
            _ => (0, 0, 0, Vec::new()),
        };

        if !items.is_empty() {
            let max_visible = 5usize;
            let count = items.len().min(max_visible);
            let h = count as u16 + 2;
            let dropdown_area = Rect { x: dropdown_x, y: dropdown_y, width: dropdown_w, height: h };

            frame.render_widget(Clear, dropdown_area);
            let dd_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.accent))
                .style(Style::default().bg(t.bg));
            let dd_inner = dd_block.inner(dropdown_area);
            frame.render_widget(dd_block, dropdown_area);

            let scroll_start = if app.create_modal.list_scroll >= max_visible {
                app.create_modal.list_scroll - max_visible + 1
            } else {
                0
            };

            for (i, idx) in (scroll_start..items.len().min(scroll_start + max_visible)).enumerate() {
                let item_area = Rect { x: dd_inner.x, y: dd_inner.y + i as u16, width: dd_inner.width, height: 1 };
                let style = if idx == app.create_modal.list_scroll {
                    Style::default().fg(t.bg).bg(t.accent)
                } else {
                    Style::default().fg(t.text)
                };
                let text = &items[idx];
                let display: String = if text.chars().count() > dd_inner.width as usize {
                    text.chars().take(dd_inner.width as usize - 1).collect::<String>() + "…"
                } else {
                    text.clone()
                };
                frame.render_widget(Paragraph::new(Span::styled(display, style)), item_area);
                app.create_dropdown_areas.push(item_area);
            }
        }
    }

    // Saving overlay
    if app.create_modal.saving {
        let overlay = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height,
        };
        frame.render_widget(Clear, overlay);
        let saving_text = Paragraph::new(Line::from(vec![
            Span::styled("◐ ", Style::default().fg(t.accent)),
            Span::styled("Saving...", Style::default().fg(t.text)),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().bg(t.bg));
        let center_y = inner.y + inner.height / 2;
        let saving_area = Rect { x: inner.x, y: center_y, width: inner.width, height: 1 };
        frame.render_widget(saving_text, saving_area);
    }

    // Footer
    let footer = if app.create_modal.saving {
        Paragraph::new(Line::from(vec![
            Span::styled(" saving... ", Style::default().fg(t.text_dim)),
        ]))
        .alignment(Alignment::Center)
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(" Tab", Style::default().fg(t.accent)),
            Span::styled(" navigate  ", Style::default().fg(t.text_dim)),
            Span::styled(" ↵ select ", Style::default().fg(t.bg).bg(t.accent)),
            Span::raw(" "),
            Span::styled(" Ctrl+S save ", Style::default().fg(t.bg).bg(t.accent)),
            Span::raw(" "),
            Span::styled(" esc close ", Style::default().fg(t.text).bg(t.border)),
        ]))
        .alignment(Alignment::Center)
    };
    frame.render_widget(footer, footer_area);
}

fn render_editor_with_label(
    frame: &mut Frame,
    editor: &mut crate::app::SimpleEditor,
    area: Rect,
    t: &crate::theme::Theme,
    focused: bool,
    label: &str,
) {
    let border_color = if focused { t.accent } else { t.border };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(format!(" {} ", label))
        .title_style(Style::default().fg(t.text_dim));
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
