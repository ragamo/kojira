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

fn status_color(issue: &JiraIssue, theme: &Theme) -> Color {
    let name = issue.fields.status.name.to_lowercase();
    let cat = issue
        .fields
        .status
        .status_category
        .as_ref()
        .map(|c| c.key.as_str())
        .unwrap_or("");

    // Match by common status names first
    if name.contains("done") || name.contains("closed") || name.contains("resolved") || name.contains("complete") {
        return theme.success;
    }
    if name.contains("progress") || name.contains("review") || name.contains("testing") || name.contains("dev") {
        return theme.info;
    }
    if name.contains("block") || name.contains("impediment") || name.contains("rejected") {
        return theme.error;
    }
    if name.contains("todo") || name.contains("to do") || name.contains("open") || name.contains("backlog") {
        return theme.text_dim;
    }
    if name.contains("ready") || name.contains("approved") || name.contains("validated") {
        return Color::Rgb(170, 210, 90); // lime
    }
    if name.contains("hold") || name.contains("wait") || name.contains("pending") {
        return theme.warning;
    }

    // Fallback to category
    match cat {
        "done" => theme.success,
        "indeterminate" => theme.info,
        _ => theme.text_dim,
    }
}

pub fn render(frame: &mut Frame, app: &mut App, list_id: u64, area: Rect) {
    let t = app.theme;

    let tab = match app.list_tabs.iter().find(|t| t.id == list_id).cloned() {
        Some(t) => t,
        None => return,
    };

    // Filter bar
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    render_filters(frame, app, &tab, chunks[0]);
    let content_area = chunks[1];

    if tab.loading {
        let loading = Paragraph::new("Loading backlog...")
            .style(Style::default().fg(t.text_dim))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(t.border)),
            );
        frame.render_widget(loading, content_area);
        return;
    }

    if tab.issues.is_empty() {
        let msg = if let Some(ref err) = tab.error {
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
        frame.render_widget(empty, content_area);
        return;
    }

    let area = content_area;

    let assignee_filter = app.assignee_filter_active.clone();
    let filtered_issues: Vec<&JiraIssue> = tab
        .issues
        .iter()
        .filter(|issue| match &tab.filter {
            None => true,
            Some(f) => issue.fields.status.name == *f,
        })
        .filter(|issue| match &assignee_filter {
            None => true,
            Some(name) => issue.fields.assignee.as_ref().map(|u| &u.display_name) == Some(name),
        })
        .collect();

    if filtered_issues.is_empty() {
        let empty = Paragraph::new("No issues match this filter")
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

    let item_count = filtered_issues.len();
    let visible_rows = area.height.saturating_sub(3) as usize;

    // Update nav state on the actual tab
    if let Some(real_tab) = app.list_tabs.iter_mut().find(|t| t.id == list_id) {
        real_tab.nav.visible_rows = visible_rows;
        real_tab.nav.clamp(item_count);
    }

    let nav_offset = app.list_tabs.iter().find(|t| t.id == list_id).map(|t| t.nav.offset).unwrap_or(0);
    let nav_selected = app.list_tabs.iter().find(|t| t.id == list_id).and_then(|t| t.nav.selected);

    let visible_issues: Vec<&&JiraIssue> = filtered_issues
        .iter()
        .skip(nav_offset)
        .take(visible_rows)
        .collect();

    let header = Row::new(vec!["Key", "Type", "Summary", "Epic", "Status", "Assignee", "Updated"])
        .style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = visible_issues
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let actual_index = nav_offset + i;
            let is_selected = nav_selected == Some(actual_index);

            let status_color = status_color(issue, t);

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

    let scroll_indicator = if item_count > visible_rows {
        format!(
            " {}-{}/{} ",
            nav_offset + 1,
            (nav_offset + visible_issues.len()).min(item_count),
            item_count
        )
    } else {
        String::new()
    };

    let key_width = filtered_issues
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

    let row_start_y = area.y + 2;
    app.click_regions.backlog.row_areas = (0..visible_issues.len())
        .map(|i| Rect {
            x: area.x + 1,
            y: row_start_y + i as u16,
            width: area.width.saturating_sub(2),
            height: 1,
        })
        .collect();

    if item_count > visible_rows {
        let mut scrollbar_state =
            ScrollbarState::new(item_count.saturating_sub(visible_rows))
                .position(nav_offset);
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

fn render_filters(frame: &mut Frame, app: &mut App, tab: &crate::app::ListTab, area: Rect) {
    let t = app.theme;
    let mut filter_areas = Vec::new();
    let mut x_offset = area.x;

    let all_active = tab.filter.is_none();
    let all_style = if all_active {
        Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text_dim)
    };
    let all_label = " all ";
    let all_width = all_label.len() as u16;
    filter_areas.push(Rect { x: x_offset, y: area.y, width: all_width, height: 1 });
    x_offset += all_width + 1;

    let mut spans: Vec<Span> = vec![
        Span::styled(all_label, all_style),
        Span::raw(" "),
    ];

    for status in &tab.statuses.clone() {
        let is_active = tab.filter.as_deref() == Some(status.as_str());
        let style = if is_active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        };
        let label = format!(" {} ", status.to_lowercase());
        let width = label.len() as u16;
        filter_areas.push(Rect { x: x_offset, y: area.y, width, height: 1 });
        x_offset += width + 1;
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
    }

    if let Some(ref assignee) = app.assignee_filter_active {
        spans.push(Span::raw("| "));
        let label = format!(" @{} ", assignee);
        let style = Style::default().fg(t.bg).bg(t.warning).add_modifier(Modifier::BOLD);
        spans.push(Span::styled(label, style));
    }

    app.click_regions.backlog.filter_areas = filter_areas;
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
