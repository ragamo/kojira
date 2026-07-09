use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::app::{App, DragTransitionState};

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

fn color_for(name: &str, palette: &[(u8, u8, u8)]) -> Color {
    let hash = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u32));
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

    app.board_content_area = Some(area);

    let hide_subtasks = app.board_hide_subtasks;
    let hide_backlog = app.board_hide_backlog_col;

    let visible_columns: Vec<&_> = tab
        .columns
        .iter()
        .filter(|col| !(hide_backlog && col.name.eq_ignore_ascii_case("backlog")))
        .collect();

    let col_count = visible_columns.len() as u16;
    let col_width = area.width / col_count.max(1);

    let visible_col_indices: Vec<usize> = tab
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| !(hide_backlog && c.name.eq_ignore_ascii_case("backlog")))
        .map(|(i, _)| i)
        .collect();

    let mut clamped_scrolls: Vec<(usize, usize)> = Vec::new();

    for (i, col) in visible_columns.iter().enumerate() {
        let x = area.x + (i as u16) * col_width;
        let w = if i as u16 == col_count - 1 {
            area.width - (i as u16) * col_width
        } else {
            col_width
        };
        let col_area = Rect { x, y: area.y, width: w, height: area.height };

        let scroll_offset = visible_col_indices
            .get(i)
            .and_then(|&idx| tab.col_scroll.get(idx))
            .copied()
            .unwrap_or(0);

        let assignee_filter = &app.assignee_filter_active;
        let visible_issues: Vec<&_> = col
            .issues
            .iter()
            .filter(|issue| {
                if hide_subtasks && issue.fields.issue_type.name.eq_ignore_ascii_case("sub-task") {
                    return false;
                }
                if let Some(name) = assignee_filter {
                    if issue.fields.assignee.as_ref().map(|u| &u.display_name) != Some(name) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let title = format!(" {} ({}) ", col.name.to_uppercase(), visible_issues.len());
        let borders = if (i as u16) < col_count - 1 {
            Borders::RIGHT
        } else {
            Borders::NONE
        };
        let block = Block::default()
            .borders(borders)
            .border_style(Style::default().fg(t.border))
            .title(title)
            .title_style(Style::default().fg(t.text_dim).add_modifier(Modifier::BOLD));
        let inner = block.inner(col_area);
        frame.render_widget(block, col_area);

        // Clamp scroll: find the max offset where the last card still fills the bottom
        let col_height = inner.height;
        let mut cumulative_heights: Vec<u16> = Vec::with_capacity(visible_issues.len());
        for issue in &visible_issues {
            let summary_lines = wrap_str(&issue.fields.summary, (inner.width as usize).saturating_sub(2));
            let has_epic = issue.fields.parent.is_some();
            let card_h = (summary_lines.len() as u16) + 1 + if has_epic { 1 } else { 0 } + 1; // card + bottom padding
            cumulative_heights.push(card_h);
        }
        let total_height: u16 = cumulative_heights.iter().sum();
        let max_scroll = if total_height <= col_height {
            0
        } else {
            let mut sum_from_end: u16 = 0;
            let mut max_s = visible_issues.len();
            for (j, h) in cumulative_heights.iter().enumerate().rev() {
                sum_from_end += h;
                if sum_from_end >= col_height {
                    max_s = j + 1;
                    break;
                }
            }
            max_s
        };
        let scroll_offset = scroll_offset.min(max_scroll);

        if let Some(&actual_idx) = visible_col_indices.get(i) {
            clamped_scrolls.push((actual_idx, scroll_offset));
        }

        let mut y_offset = 0u16;
        let issues_to_render: Vec<&_> = visible_issues.iter().skip(scroll_offset).copied().collect();

        // Determine placeholder state for this column during drag
        let is_source_col = app.card_dragging.as_ref().map(|d| d.source_col == i).unwrap_or(false);
        let drag_placeholder = app.card_drag_target
            .filter(|&(col_idx, _)| col_idx == i)
            .map(|(_, row)| row);
        let drag_target_row = drag_placeholder;

        // Compute placeholder style and content based on transition state
        let ph_style = if drag_target_row.is_some() {
            if is_source_col {
                match &app.card_drag_transition_state {
                    Some(DragTransitionState::Loading) => {
                        let spinner_frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                        let tick = (std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .subsec_millis() / 100) as usize;
                        let spinner = spinner_frames[tick % spinner_frames.len()];
                        Some((t.header_bg, vec![
                            Span::styled(format!("{} ", spinner), Style::default().fg(t.accent)),
                            Span::styled("loading…", Style::default().fg(t.text_dim)),
                        ]))
                    }
                    _ => Some((t.accent, vec![]))
                }
            } else {
                match &app.card_drag_transition_state {
                    Some(DragTransitionState::Loading) => {
                        let spinner_frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                        let tick = (std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .subsec_millis() / 100) as usize;
                        let spinner = spinner_frames[tick % spinner_frames.len()];
                        Some((t.header_bg, vec![
                            Span::styled(format!("{} ", spinner), Style::default().fg(t.accent)),
                            Span::styled("loading…", Style::default().fg(t.text_dim)),
                        ]))
                    }
                    Some(DragTransitionState::Loaded(allowed)) => {
                        let is_allowed = allowed.iter().any(|n| n.eq_ignore_ascii_case(&col.name));
                        if is_allowed {
                            Some((t.accent, vec![
                                Span::styled("✓ allowed", Style::default().fg(t.bg).add_modifier(Modifier::BOLD)),
                            ]))
                        } else {
                            Some((t.error, vec![
                                Span::styled("✗ invalid transition", Style::default().fg(t.bg).add_modifier(Modifier::BOLD)),
                            ]))
                        }
                    }
                    None => Some((t.accent, vec![]))
                }
            }
        } else {
            None
        };

        let mut card_index = 0usize;
        for issue in &issues_to_render {
            // Insert placeholder before this card if it matches the insert position
            if let Some(target_row) = drag_target_row {
                if card_index == target_row {
                    if let Some((bg, ref spans)) = ph_style {
                        let ph_h = 3u16;
                        if y_offset + ph_h <= inner.height {
                            let ph_area = Rect {
                                x: inner.x,
                                y: inner.y + y_offset,
                                width: inner.width,
                                height: ph_h,
                            };
                            let content = Line::from(spans.clone());
                            let ph = Paragraph::new(content)
                                .alignment(Alignment::Center)
                                .block(Block::default().padding(Padding::vertical(1)))
                                .style(Style::default().bg(bg));
                            frame.render_widget(ph, ph_area);
                            y_offset += ph_h;
                        }
                    }
                }
            }
            card_index += 1;
            let content_width = (inner.width as usize).saturating_sub(2);

            // Row 1+: Summary (wrapped)
            let summary_lines = wrap_str(&issue.fields.summary, content_width);
            let has_epic = issue.fields.parent.is_some();
            let card_height = (summary_lines.len() as u16) + 1 + if has_epic { 1 } else { 0 } + 1; // summary + key/assignee [+ epic] + bottom padding

            if y_offset + card_height > inner.height {
                break;
            }

            let card_area = Rect {
                x: inner.x,
                y: inner.y + y_offset,
                width: inner.width,
                height: card_height,
            };

            let priority_indicator = issue.fields.priority.as_ref().map(|p| {
                let name = p.name.to_lowercase();
                if name.contains("highest") || name.contains("critical") || name.contains("blocker") {
                    Some(("!!", t.error))
                } else if name.contains("high") {
                    Some(("!", t.warning))
                } else {
                    None
                }
            }).flatten();

            let mut lines: Vec<Line> = summary_lines
                .iter()
                .enumerate()
                .map(|(idx, s)| {
                    if idx == 0 {
                        if let Some((indicator, color)) = priority_indicator {
                            Line::from(vec![
                                Span::styled(indicator, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                                Span::styled(" ", Style::default()),
                                Span::styled(s.as_str(), Style::default().fg(t.text)),
                            ])
                        } else {
                            Line::from(Span::styled(s.as_str(), Style::default().fg(t.text)))
                        }
                    } else {
                        Line::from(Span::styled(s.as_str(), Style::default().fg(t.text)))
                    }
                })
                .collect();

            // Row 2: Epic with background color (only if present)
            if let Some(ref parent) = issue.fields.parent {
                let epic_name = parent
                    .fields
                    .as_ref()
                    .map(|f| f.summary.as_str())
                    .unwrap_or(&parent.key);
                let bg = color_for(epic_name, EPIC_COLORS);
                let display = format!(" {} ", truncate_str(epic_name, content_width.saturating_sub(2)));
                lines.push(Line::from(Span::styled(display, Style::default().fg(Color::Rgb(30, 30, 30)).bg(bg))));
            }

            // Row 3: Type icon + Key left, Assignee right
            let type_icon = match issue.fields.issue_type.name.to_lowercase().as_str() {
                n if n.contains("bug")        => ("●", t.error),
                n if n.contains("epic")       => ("⬡", t.highlight),
                n if n.contains("story")      => ("◈", t.success),
                n if n.contains("task")       => ("◻", t.info),
                n if n.contains("sub-task") || n.contains("subtask") => ("◽", t.text_dim),
                n if n.contains("spike")      => ("◇", t.warning),
                n if n.contains("feature")    => ("★", t.accent),
                n if n.contains("improvement") || n.contains("enhancement") => ("▲", t.info),
                _                             => ("◦", t.text_dim),
            };
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

            let icon_prefix = 2usize; // icon + space
            let key_len = issue.key.len();
            let available = content_width.saturating_sub(icon_prefix + key_len + 2);
            let assignee_display = truncate_str(assignee_name, available);
            let padding = content_width.saturating_sub(icon_prefix + key_len + assignee_display.len()).max(1);
            let line3 = Line::from(vec![
                Span::styled(type_icon.0, Style::default().fg(type_icon.1)),
                Span::raw(" "),
                key_span,
                Span::raw(" ".repeat(padding)),
                Span::styled(assignee_display, Style::default().fg(assignee_color)),
            ]);

            lines.push(line3);

            let mouse = app.mouse_pos;
            let hovered = mouse.0 >= card_area.x
                && mouse.0 < card_area.x + card_area.width
                && mouse.1 >= card_area.y
                && mouse.1 < card_area.y + card_area.height;

            app.click_regions.board_cards.cards.push((card_area, issue.key.clone(), i));

            let is_being_dragged = app.card_dragging.as_ref().map(|d| d.issue_key == issue.key).unwrap_or(false);
            let card_bg = if is_being_dragged {
                t.bg
            } else if hovered {
                t.header_bg
            } else {
                t.bg
            };
            let card_fg = if is_being_dragged { t.text_dim } else { t.text };
            let card = Paragraph::new(lines)
                .block(Block::default().padding(Padding { left: 1, right: 1, bottom: 1, ..Padding::ZERO }))
                .style(Style::default().bg(card_bg).fg(card_fg));
            frame.render_widget(card, card_area);

            y_offset += card_height;
        }

        // Placeholder at end of column if target_row >= card_index
        if let Some(target_row) = drag_target_row {
            if target_row >= card_index {
                if let Some((bg, ref spans)) = ph_style {
                    let ph_h = 3u16;
                    if y_offset + ph_h <= inner.height {
                        let ph_area = Rect {
                            x: inner.x,
                            y: inner.y + y_offset,
                            width: inner.width,
                            height: ph_h,
                        };
                        let content = Line::from(spans.clone());
                        let ph = Paragraph::new(content)
                            .alignment(Alignment::Center)
                            .block(Block::default().padding(Padding::vertical(1)))
                            .style(Style::default().bg(bg));
                        frame.render_widget(ph, ph_area);
                    }
                }
            }
        }
    }

    // Apply clamped scrolls
    if let Some(tab_mut) = app.board_tabs.iter_mut().find(|t| t.board_id == board_id) {
        for (idx, val) in clamped_scrolls {
            if let Some(s) = tab_mut.col_scroll.get_mut(idx) {
                *s = val;
            }
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
