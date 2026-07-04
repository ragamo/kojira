use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;

const EPIC_COLORS: &[(u8, u8, u8)] = &[
    (126, 156, 216),  // blue
    (90, 195, 170),   // teal
    (149, 127, 184),  // violet
    (152, 187, 108),  // green
    (100, 175, 230),  // sky
    (170, 120, 190),  // mauve
    (230, 195, 132),  // yellow
    (127, 180, 202),  // cyan
    (80, 190, 210),   // aqua
    (170, 210, 90),   // lime
    (200, 130, 170),  // pink
    (220, 175, 80),   // gold
    (255, 160, 102),  // orange
    (240, 130, 100),  // coral
    (228, 104, 118),  // red
];

fn epic_color(name: &str) -> Color {
    let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u32));
    let idx = (hash as usize) % EPIC_COLORS.len();
    let (r, g, b) = EPIC_COLORS[idx];
    Color::Rgb(r, g, b)
}

const AUTHOR_COLORS: &[(u8, u8, u8)] = &[
    (97, 175, 239),
    (229, 192, 123),
    (152, 195, 121),
    (224, 108, 117),
    (198, 120, 221),
    (86, 182, 194),
    (255, 121, 198),
    (189, 147, 249),
    (241, 250, 140),
    (139, 233, 253),
    (255, 183, 77),
    (128, 203, 196),
];

fn color_for_author(name: &str) -> Color {
    let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let idx = (hash as usize) % AUTHOR_COLORS.len();
    let (r, g, b) = AUTHOR_COLORS[idx];
    Color::Rgb(r, g, b)
}

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let issue = match &app.detail_issue {
        Some(i) => i.clone(),
        None => return,
    };
    let t = app.theme;

    // Resize area = top border row
    app.detail_resize_area = Some(Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    });

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // [X] close
    let close_area = Rect {
        x: area.x + area.width.saturating_sub(4),
        y: area.y,
        width: 3,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled("[X]", Style::default().fg(t.text_dim))),
        close_area,
    );
    app.detail_close_area = Some(close_area);

    let status_color = match issue
        .fields
        .status
        .status_category
        .as_ref()
        .map(|c| c.key.as_str())
    {
        Some("done") => t.success,
        Some("indeterminate") => t.info,
        _ => t.text_dim,
    };

    let assignee = issue
        .fields
        .assignee
        .as_ref()
        .map(|a| a.display_name.as_str())
        .unwrap_or("-");

    // Line 1: Key + Summary
    let line1 = Line::from(vec![
        Span::styled(
            &issue.key,
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            &issue.fields.summary,
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        ),
    ]);

    // Line 2: Type | Status | Assignee | Epic tag
    let mut line2_spans = vec![
        Span::styled(
            &issue.fields.issue_type.name,
            Style::default().fg(t.text_dim),
        ),
        Span::styled(" │ ", Style::default().fg(t.text_dim)),
        Span::styled(&issue.fields.status.name, Style::default().fg(status_color)),
        Span::styled(" │ ", Style::default().fg(t.text_dim)),
        Span::styled(assignee, Style::default().fg(t.warning)),
    ];

    if let Some(ref parent) = issue.fields.parent {
        let epic_name = parent
            .fields
            .as_ref()
            .map(|f| f.summary.as_str())
            .unwrap_or(&parent.key);
        let display = if epic_name.chars().count() > 22 {
            let truncated: String = epic_name.chars().take(21).collect();
            format!(" {}… ", truncated)
        } else {
            format!(" {} ", epic_name)
        };
        let bg = epic_color(epic_name);
        line2_spans.push(Span::styled(" │ ", Style::default().fg(t.text_dim)));
        line2_spans.push(Span::styled(
            display,
            Style::default().fg(Color::Rgb(30, 30, 30)).bg(bg),
        ));
    }

    let line2 = Line::from(line2_spans);

    // Line 3: URL
    let base_url = app
        .config
        .jira
        .base_url
        .as_deref()
        .unwrap_or("https://jira.atlassian.net");
    let url = format!("{}/browse/{}", base_url, issue.key);
    let url_len = url.len() as u16;
    let line3 = Line::from(Span::styled(url, Style::default().fg(t.text_dim)));

    let header_lines = vec![line1, line2, line3];
    let header_height = header_lines.len() as u16;

    let chunks = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Length(1), // blank
        Constraint::Length(1), // tabs
        Constraint::Length(1), // separator
        Constraint::Min(0),   // content
        Constraint::Length(1), // statusbar
    ])
    .split(inner);

    // Register URL click area (line 3 of header)
    app.detail_url_area = Some(Rect {
        x: chunks[0].x,
        y: chunks[0].y + 2,
        width: url_len,
        height: 1,
    });

    frame.render_widget(Paragraph::new(header_lines), chunks[0]);

    // Tabs row
    const DETAIL_TABS: &[&str] = &["overview", "comments", "transitions"];
    let tab_area = chunks[2];
    let mut tab_click_areas: Vec<Rect> = Vec::new();
    let mut x_offset = tab_area.x;
    let mut tab_spans: Vec<Span> = Vec::new();
    for (i, &label) in DETAIL_TABS.iter().enumerate() {
        let is_active = i == app.detail_tab;
        let style = if is_active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        };
        let text = format!(" {} ", label);
        let width = text.len() as u16;
        tab_click_areas.push(Rect { x: x_offset, y: tab_area.y, width, height: 1 });
        tab_spans.push(Span::styled(text, style));
        tab_spans.push(Span::raw(" "));
        x_offset += width + 1;
    }
    app.detail_tab_areas = tab_click_areas;
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), tab_area);

    // Transition button (right side of header, 3 rows tall like lazyglab merge btn)
    let mut transition_btn_area: Option<Rect> = None;
    if !app.detail_transitions.is_empty() {
        let current_status = &issue.fields.status.name;
        let text = format!("{} ⏷", current_status);
        let text_width = text.chars().count() as u16;
        let btn_width = 2 + text_width + 2; // padding: 2 left + text + 2 right
        let btn_height: u16 = 3;
        let right_margin: u16 = 1;

        if inner.width >= btn_width + right_margin + 4 && chunks[0].height >= btn_height {
            let btn_area = Rect {
                x: inner.x + inner.width.saturating_sub(btn_width + right_margin),
                y: chunks[0].y,
                width: btn_width,
                height: btn_height,
            };

            let btn_bg = t.accent;
            let fg = t.bg;
            let outer_bg = t.bg;

            // Top row: ▄
            let top_row = Rect { x: btn_area.x, y: btn_area.y, width: btn_width, height: 1 };
            frame.render_widget(
                Paragraph::new(Span::styled(
                    "▄".repeat(btn_width as usize),
                    Style::default().fg(btn_bg).bg(outer_bg),
                )),
                top_row,
            );

            // Middle row: padding + text + padding
            let mid_row = Rect { x: btn_area.x, y: btn_area.y + 1, width: btn_width, height: 1 };
            let bg_style = Style::default().fg(fg).bg(btn_bg);
            let text_style = bg_style.add_modifier(Modifier::BOLD);
            let mid_line = Line::from(vec![
                Span::styled("  ", bg_style),
                Span::styled(&text, text_style),
                Span::styled("  ", bg_style),
            ]);
            frame.render_widget(Paragraph::new(mid_line), mid_row);

            // Bottom row: ▀
            let bot_row = Rect { x: btn_area.x, y: btn_area.y + 2, width: btn_width, height: 1 };
            frame.render_widget(
                Paragraph::new(Span::styled(
                    "▀".repeat(btn_width as usize),
                    Style::default().fg(btn_bg).bg(outer_bg),
                )),
                bot_row,
            );

            app.detail_transition_btn_area = Some(btn_area);
            transition_btn_area = Some(btn_area);
        }
    }

    // Separator
    frame.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(chunks[3].width as usize),
            Style::default().fg(t.border),
        )),
        chunks[3],
    );

    // Split content into main (left) and metadata (right)
    let content_area = chunks[4];
    let statusbar_area = chunks[5];
    let meta_width = 28u16;
    let content_splits = Layout::horizontal([
        Constraint::Min(20),
        Constraint::Length(meta_width),
    ])
    .split(content_area);

    let desc_area = content_splits[0];
    let meta_area = content_splits[1];

    // If editing, render inline editor and metadata, then return
    if app.detail_editing && app.detail_tab == 0 {
        crate::ui::editor_widget::render_editor(frame, &mut app.detail_editor, desc_area, t, true);
        render_metadata(frame, app, t, &issue, meta_area);
        let edit_statusbar = Line::from(vec![
            Span::styled(" Ctrl+S", Style::default().fg(t.accent)),
            Span::styled(" save  ", Style::default().fg(t.text_dim)),
            Span::styled("Esc", Style::default().fg(t.accent)),
            Span::styled(" cancel", Style::default().fg(t.text_dim)),
        ]);
        frame.render_widget(Paragraph::new(edit_statusbar), statusbar_area);
        return;
    }

    // Tab content
    let content_lines: Vec<Line> = match app.detail_tab {
        0 => {
            let text = if let Some(ref desc) = app.detail_description {
                desc.clone()
            } else {
                "Loading...  press e to edit".into()
            };
            text.lines()
                .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(t.text))))
                .collect()
        }
        1 => {
            if app.detail_comments.is_empty() {
                vec![Line::from(Span::styled("No comments", Style::default().fg(t.text_dim)))]
            } else {
                let mut lines = Vec::new();
                for (idx, c) in app.detail_comments.iter().enumerate() {
                    let date = if c.created.len() >= 10 { &c.created[..10] } else { &c.created };
                    let author_color = color_for_author(&c.author.display_name);
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("@{}", c.author.display_name),
                            Style::default().fg(author_color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!("  {}", date), Style::default().fg(t.text_dim)),
                    ]));
                    let body = c
                        .body
                        .as_ref()
                        .map(|b| crate::provider::jira::adf_to_text(b))
                        .unwrap_or_default();
                    for line in body.lines() {
                        lines.push(Line::from(Span::styled(
                            line.to_string(),
                            Style::default().fg(t.text),
                        )));
                    }
                    if idx < app.detail_comments.len() - 1 {
                        lines.push(Line::from(""));
                    }
                }
                lines
            }
        }
        2 => {
            if app.detail_changelog.is_empty() {
                vec![Line::from(Span::styled(
                    "No transition history",
                    Style::default().fg(t.text_dim),
                ))]
            } else {
                let mut lines = Vec::new();
                const STATUS_W: usize = 18;
                for entry in app.detail_changelog.iter() {
                    let date = if entry.created.len() >= 10 { &entry.created[..10] } else { &entry.created };
                    let author_color = color_for_author(&entry.author.display_name);
                    for item in entry.items.iter().filter(|i| i.field == "status") {
                        let from = pad_or_truncate(item.from_string.as_deref().unwrap_or("?"), STATUS_W);
                        let to = pad_or_truncate(item.to_string.as_deref().unwrap_or("?"), STATUS_W);
                        lines.push(Line::from(vec![
                            Span::styled(from, Style::default().fg(t.text_dim)),
                            Span::styled(" → ", Style::default().fg(t.border)),
                            Span::styled(to, Style::default().fg(t.info).add_modifier(Modifier::BOLD)),
                            Span::styled(format!("  {} ", date), Style::default().fg(t.text_dim)),
                            Span::styled(
                                &entry.author.display_name,
                                Style::default().fg(author_color),
                            ),
                        ]));
                    }
                }
                lines
            }
        }
        _ => Vec::new(),
    };

    let desc_area_height = desc_area.height;
    let total_lines_count = content_lines.len() as u16;
    app.detail_max_scroll = total_lines_count.saturating_sub(desc_area_height);

    let content_widget = Paragraph::new(content_lines)
        .style(Style::default().fg(t.text))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    frame.render_widget(content_widget, desc_area);

    // Metadata panel
    render_metadata(frame, app, t, &issue, meta_area);

    // Status bar
    let mut statusbar_spans: Vec<Span> = Vec::new();
    if app.detail_tab == 0 {
        statusbar_spans.push(Span::styled(" e", Style::default().fg(t.accent)));
        statusbar_spans.push(Span::styled(" edit  ", Style::default().fg(t.text_dim)));
    }
    statusbar_spans.push(Span::styled("t", Style::default().fg(t.accent)));
    statusbar_spans.push(Span::styled(" transition  ", Style::default().fg(t.text_dim)));
    statusbar_spans.push(Span::styled("←→", Style::default().fg(t.accent)));
    statusbar_spans.push(Span::styled(" tabs  ", Style::default().fg(t.text_dim)));
    statusbar_spans.push(Span::styled("↑↓", Style::default().fg(t.accent)));
    statusbar_spans.push(Span::styled(" scroll  ", Style::default().fg(t.text_dim)));
    statusbar_spans.push(Span::styled("Esc", Style::default().fg(t.accent)));
    statusbar_spans.push(Span::styled(" close", Style::default().fg(t.text_dim)));
    let statusbar = Line::from(statusbar_spans);
    frame.render_widget(Paragraph::new(statusbar), statusbar_area);

    // Transition dropdown (rendered last to overlay everything)
    if app.detail_transition_open {
        if let Some(btn_area) = transition_btn_area {
            let dropdown_height = (app.detail_transitions.len() as u16 + 2).min(10);
            let dropdown_width = app
                .detail_transitions
                .iter()
                .map(|tr| tr.name.chars().count() as u16 + 7) // " ▸ " + name + border*2
                .max()
                .unwrap_or(20)
                .max(btn_area.width);
            let dropdown_x = (btn_area.x + btn_area.width).saturating_sub(dropdown_width);
            let dropdown_area = Rect {
                x: dropdown_x,
                y: btn_area.y + btn_area.height,
                width: dropdown_width,
                height: dropdown_height,
            };
            frame.render_widget(ratatui::widgets::Clear, dropdown_area);

            let items: Vec<ratatui::widgets::ListItem> = app
                .detail_transitions
                .iter()
                .enumerate()
                .map(|(i, tr)| {
                    let is_selected = i == app.detail_transition_selected;
                    let style = if is_selected {
                        Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(t.text)
                    };
                    let prefix = if is_selected { " ▸ " } else { "   " };
                    ratatui::widgets::ListItem::new(format!("{}{}", prefix, tr.name)).style(style)
                })
                .collect();

            let list = ratatui::widgets::List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.accent))
                    .style(Style::default().bg(t.bg)),
            );
            frame.render_widget(list, dropdown_area);
        }
    }
}

use crate::provider::types::JiraIssue;
use crate::theme::Theme;

fn render_metadata(frame: &mut Frame, app: &mut App, t: &Theme, issue: &JiraIssue, area: Rect) {
    use crate::app::DetailField;

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(t.border));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.detail_field_areas.clear();
    app.detail_field_dropdown_areas.clear();

    let label_style = Style::default().fg(t.text_dim);
    let value_style = Style::default().fg(t.text);
    let x = inner.x + 1;
    let w = inner.width.saturating_sub(1);
    let mut y = inner.y;

    // Assignee (clickable - bordered)
    let assignee = issue
        .fields
        .assignee
        .as_ref()
        .map(|a| a.display_name.as_str())
        .unwrap_or("-");
    let assignee_area = Rect { x, y, width: w, height: 3 };
    let assignee_widget = Paragraph::new(Span::styled(assignee, Style::default().fg(t.warning)))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border))
            .title(" assignee ")
            .title_style(label_style));
    frame.render_widget(assignee_widget, assignee_area);
    app.detail_field_areas.push((assignee_area, DetailField::Assignee));
    y += 4;

    // Reporter (read-only - no border)
    if let Some(ref meta) = app.detail_metadata {
        if let Some(ref reporter) = meta.reporter {
            frame.render_widget(Paragraph::new(Span::styled("reporter", label_style)), Rect { x, y, width: w, height: 1 });
            y += 1;
            frame.render_widget(Paragraph::new(Span::styled(reporter.as_str(), value_style)), Rect { x, y, width: w, height: 1 });
            y += 2;
        }
    }

    // Parent/Epic (clickable - bordered)
    let parent_display = if let Some(ref parent) = issue.fields.parent {
        let epic_name = parent.fields.as_ref().map(|f| f.summary.as_str()).unwrap_or(&parent.key);
        format!("{} {}", parent.key, epic_name)
    } else {
        "-".to_string()
    };
    let parent_area = Rect { x, y, width: w, height: 3 };
    let parent_widget = Paragraph::new(Span::styled(&parent_display, Style::default().fg(t.accent)))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border))
            .title(" parent ")
            .title_style(label_style));
    frame.render_widget(parent_widget, parent_area);
    app.detail_field_areas.push((parent_area, DetailField::Parent));
    y += 4;

    // Priority (clickable - bordered)
    let priority_name = issue.fields.priority.as_ref().map(|p| p.name.as_str()).unwrap_or("-");
    let priority_area = Rect { x, y, width: w, height: 3 };
    let priority_widget = Paragraph::new(Span::styled(priority_name, value_style))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(t.border))
            .title(" priority ")
            .title_style(label_style));
    frame.render_widget(priority_widget, priority_area);
    app.detail_field_areas.push((priority_area, DetailField::Priority));
    y += 4;

    // Type (read-only - no border)
    frame.render_widget(Paragraph::new(Span::styled("type", label_style)), Rect { x, y, width: w, height: 1 });
    y += 1;
    frame.render_widget(Paragraph::new(Span::styled(&issue.fields.issue_type.name, value_style)), Rect { x, y, width: w, height: 1 });
    y += 2;

    // Labels (read-only - no border)
    if let Some(ref meta) = app.detail_metadata {
        if !meta.labels.is_empty() {
            frame.render_widget(Paragraph::new(Span::styled("labels", label_style)), Rect { x, y, width: w, height: 1 });
            y += 1;
            frame.render_widget(Paragraph::new(Span::styled(meta.labels.join(", "), value_style)), Rect { x, y, width: w, height: 1 });
            y += 2;
        }
    }

    // Dates (read-only - no border)
    if let Some(ref meta) = app.detail_metadata {
        if let Some(ref created) = meta.created {
            let date = if created.len() >= 10 { &created[..10] } else { created };
            frame.render_widget(Paragraph::new(Span::styled("created", label_style)), Rect { x, y, width: w, height: 1 });
            y += 1;
            frame.render_widget(Paragraph::new(Span::styled(date, value_style)), Rect { x, y, width: w, height: 1 });
            let _ = y;
        }
    }

    // Dropdown overlay for field edit
    if let Some(ref edit) = app.detail_field_edit {
        let field_area = app.detail_field_areas.iter()
            .find(|(_, f)| *f == edit.field)
            .map(|(a, _)| *a);
        if let Some(fa) = field_area {
            let dropdown_x = fa.x;
            let dropdown_y = fa.y + fa.height;
            let max_visible = 5usize;
            let count = edit.items.len().min(max_visible);
            let h = count as u16 + 2;
            let dropdown_area = Rect { x: dropdown_x, y: dropdown_y, width: w, height: h };

            frame.render_widget(ratatui::widgets::Clear, dropdown_area);
            let dd_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.accent))
                .style(Style::default().bg(t.bg));
            let dd_inner = dd_block.inner(dropdown_area);
            frame.render_widget(dd_block, dropdown_area);

            let scroll_start = if edit.selected >= max_visible {
                edit.selected - max_visible + 1
            } else {
                0
            };

            for (i, idx) in (scroll_start..edit.items.len().min(scroll_start + max_visible)).enumerate() {
                let item_area = Rect { x: dd_inner.x, y: dd_inner.y + i as u16, width: dd_inner.width, height: 1 };
                let style = if idx == edit.selected {
                    Style::default().fg(t.bg).bg(t.accent)
                } else {
                    Style::default().fg(t.text)
                };
                let (_, ref display) = edit.items[idx];
                let text: String = if display.chars().count() > dd_inner.width as usize {
                    display.chars().take(dd_inner.width as usize - 1).collect::<String>() + "…"
                } else {
                    display.clone()
                };
                frame.render_widget(Paragraph::new(Span::styled(text, style)), item_area);
                app.detail_field_dropdown_areas.push(item_area);
            }
        }
    }
}

fn pad_or_truncate(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        let truncated: String = s.chars().take(width.saturating_sub(1)).collect();
        format!("{}…", truncated)
    } else {
        format!("{:<width$}", s, width = width)
    }
}
