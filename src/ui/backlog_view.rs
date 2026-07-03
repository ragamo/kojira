use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table};

use crate::app::App;
use crate::provider::types::JiraIssue;
use crate::theme::Theme;

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

fn epic_color_for(name: &str) -> Color {
    let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u32));
    let idx = (hash as usize) % EPIC_COLORS.len();
    let (r, g, b) = EPIC_COLORS[idx];
    Color::Rgb(r, g, b)
}

fn status_category_color(issue: &JiraIssue, theme: &Theme) -> Color {
    let cat = issue
        .fields
        .status
        .status_category
        .as_ref()
        .map(|c| c.key.as_str())
        .unwrap_or("");
    match cat {
        "done" => theme.success,
        "indeterminate" => theme.info,
        _ => theme.text_dim,
    }
}

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    if app.backlog_loading {
        let loading = Paragraph::new("Loading backlog...")
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(loading, area);
        return;
    }

    if app.backlog_issues.is_empty() {
        let msg = if let Some(ref err) = app.backlog_error {
            format!("Error: {} — press r to retry", err)
        } else if app.projects.is_empty() {
            "No project selected".into()
        } else {
            "No issues found — press r to refresh".into()
        };
        let empty = Paragraph::new(msg)
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let visible_rows = area.height.saturating_sub(3) as usize;
    app.backlog_nav.visible_rows = visible_rows;
    app.backlog_nav.clamp(app.backlog_issues.len());

    let visible_issues: Vec<&JiraIssue> = app
        .backlog_issues
        .iter()
        .skip(app.backlog_nav.offset)
        .take(visible_rows)
        .collect();

    let header = Row::new(vec!["Key", "Type", "Summary", "Epic", "Status", "Assignee", "Updated"])
        .style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = visible_issues
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let actual_index = app.backlog_nav.offset + i;
            let is_selected = app.backlog_nav.selected == Some(actual_index);

            let status_color = status_category_color(issue, t);

            let epic_cell = if let Some(ref parent) = issue.fields.parent {
                let epic_name = parent
                    .fields
                    .as_ref()
                    .map(|f| f.summary.as_str())
                    .unwrap_or(&parent.key);
                let bg = epic_color_for(epic_name);
                Cell::from(format!(" {} ", epic_name))
                    .style(Style::default().fg(Color::Rgb(30, 30, 30)).bg(bg))
            } else {
                Cell::from("").style(Style::default().fg(t.text_dim))
            };

            let assignee = issue
                .fields
                .assignee
                .as_ref()
                .map(|a| a.display_name.as_str())
                .unwrap_or("-");

            let updated = if issue.fields.updated.len() >= 10 {
                &issue.fields.updated[..10]
            } else {
                &issue.fields.updated
            };

            let row = Row::new(vec![
                Cell::from(issue.key.clone()).style(Style::default().fg(t.accent)),
                Cell::from(issue.fields.issue_type.name.clone()).style(Style::default().fg(t.text_dim)),
                Cell::from(issue.fields.summary.clone()).style(Style::default().fg(t.text)),
                epic_cell,
                Cell::from(issue.fields.status.name.clone()).style(Style::default().fg(status_color)),
                Cell::from(assignee.to_string()).style(Style::default().fg(t.warning)),
                Cell::from(updated.to_string()).style(Style::default().fg(t.text_dim)),
            ]);

            if is_selected {
                row.style(Style::default().bg(t.border))
            } else {
                row
            }
        })
        .collect();

    let scroll_indicator = if app.backlog_issues.len() > visible_rows {
        format!(
            " {}-{}/{} ",
            app.backlog_nav.offset + 1,
            (app.backlog_nav.offset + visible_issues.len()).min(app.backlog_issues.len()),
            app.backlog_issues.len()
        )
    } else {
        String::new()
    };

    let key_width = app
        .backlog_issues
        .iter()
        .map(|i| i.key.len() as u16)
        .max()
        .unwrap_or(8)
        .max(5);

    let table = Table::new(
        rows,
        [
            Constraint::Length(key_width + 1),
            Constraint::Length(10),
            Constraint::Min(20),
            Constraint::Length(22),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.border))
            .title_bottom(Line::from(scroll_indicator).right_aligned())
            .title_style(Style::default().fg(t.text_dim)),
    );

    frame.render_widget(table, area);

    if app.backlog_issues.len() > visible_rows {
        let mut scrollbar_state =
            ScrollbarState::new(app.backlog_issues.len().saturating_sub(visible_rows))
                .position(app.backlog_nav.offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(t.border))
            .thumb_style(Style::default().fg(t.accent));
        let scrollbar_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y + 1,
            width: 1,
            height: area.height.saturating_sub(2),
        };
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}
