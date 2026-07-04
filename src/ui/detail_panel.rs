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
        Constraint::Length(1),
        Constraint::Min(0),
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

    // Transition button (right side of header, line 1)
    let mut transition_btn_area: Option<Rect> = None;
    if !app.detail_transitions.is_empty() {
        let current_status = &issue.fields.status.name;
        let btn_label = format!(" {} ⏷ ", current_status);
        let btn_width = btn_label.chars().count() as u16;
        let btn_area = Rect {
            x: inner.x + inner.width.saturating_sub(btn_width),
            y: chunks[0].y,
            width: btn_width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(Span::styled(
                btn_label,
                Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD),
            )),
            btn_area,
        );
        app.detail_transition_btn_area = Some(btn_area);
        transition_btn_area = Some(btn_area);
    }

    // Separator
    frame.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(chunks[1].width as usize),
            Style::default().fg(t.border),
        )),
        chunks[1],
    );

    // Description or placeholder
    let desc = if let Some(ref desc) = app.detail_description {
        desc.clone()
    } else {
        "No description".into()
    };
    let desc_widget = Paragraph::new(desc)
        .style(Style::default().fg(t.text))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    frame.render_widget(desc_widget, chunks[2]);

    // Transition dropdown (rendered last to overlay everything)
    if app.detail_transition_open {
        if let Some(btn_area) = transition_btn_area {
            let dropdown_height = (app.detail_transitions.len() as u16 + 2).min(10);
            let dropdown_width = app
                .detail_transitions
                .iter()
                .map(|tr| tr.name.chars().count() as u16 + 4)
                .max()
                .unwrap_or(20)
                .max(btn_area.width);
            let dropdown_area = Rect {
                x: btn_area.x + btn_area.width.saturating_sub(dropdown_width),
                y: btn_area.y + 1,
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
