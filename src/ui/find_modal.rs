use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let screen = frame.area();

    let panel_open = app.find_board_panel_open;

    let base_w = (screen.width * 55 / 100).max(40).min(screen.width);
    let w = if panel_open {
        (base_w + base_w.min(36)).min(screen.width)
    } else {
        base_w
    };
    let h = (screen.height * 55 / 100).max(12).min(screen.height);
    let area = Rect {
        x: screen.x + screen.width.saturating_sub(w) / 2,
        y: screen.y + screen.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    };

    frame.render_widget(Clear, area);

    if panel_open {
        render_with_panel(frame, app, area);
    } else {
        render_search_only(frame, app, area);
    }
}

fn render_search_only(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let block = Block::default()
        .title(" Find Project ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    render_search_pane(frame, app, inner);

    app.click_regions.find_modal.bounds = Some(area);
}

fn render_with_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    // Split horizontally: left = search, right = board panel
    let left_w = area.width * 55 / 100;
    let right_w = area.width.saturating_sub(left_w);

    let left_area = Rect { x: area.x, y: area.y, width: left_w, height: area.height };
    let right_area = Rect {
        x: area.x + left_w,
        y: area.y,
        width: right_w,
        height: area.height,
    };

    // Left pane – search (dimmed border, not focused)
    let left_block = Block::default()
        .title(" Find Project ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.border));
    let left_inner = left_block.inner(left_area);
    frame.render_widget(left_block, left_area);
    render_search_pane(frame, app, left_inner);

    // Right pane – board panel (accent border, focused)
    let panel_title = app
        .find_panel_project
        .as_ref()
        .map(|p| format!(" {} ", p.key))
        .unwrap_or_else(|| " Boards ".to_string());

    let right_block = Block::default()
        .title(panel_title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));
    let right_inner = right_block.inner(right_area);
    frame.render_widget(right_block, right_area);

    render_board_panel(frame, app, right_inner);

    app.click_regions.find_modal.bounds = Some(area);
}

fn render_search_pane(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    let cursor = "▌";
    let input_text = format!("{}{}", app.find_input, cursor);
    let input = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(t.border))
                .title(" Search ")
                .title_style(Style::default().fg(t.text_dim)),
        )
        .style(Style::default().fg(t.text));
    frame.render_widget(input, chunks[0]);

    let status = if app.find_loading {
        Paragraph::new("Searching...").style(Style::default().fg(t.warning))
    } else if !app.find_results.is_empty() {
        Paragraph::new(format!("{} results", app.find_results.len()))
            .style(Style::default().fg(t.text_dim))
    } else if !app.find_input.is_empty() {
        Paragraph::new("Press Enter to search").style(Style::default().fg(t.text_dim))
    } else {
        Paragraph::new("Type a keyword to search projects")
            .style(Style::default().fg(t.text_dim))
    };
    frame.render_widget(status, chunks[1]);

    let results_area = chunks[2];
    let mut result_areas = Vec::new();
    let mut star_areas = Vec::new();

    let items: Vec<ListItem> = app
        .find_results
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = i == app.find_selected;
            let star = if p.is_favorite { "★" } else { "☆" };

            let style = if is_selected {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };

            let star_style = if p.is_favorite {
                Style::default().fg(t.warning)
            } else {
                Style::default().fg(t.text_dim)
            };

            let line = Line::from(vec![
                Span::styled(format!(" {} ", star), star_style),
                Span::styled(format!("{} - {}", p.key, p.name), style),
            ]);

            result_areas.push(Rect {
                x: results_area.x,
                y: results_area.y + i as u16,
                width: results_area.width,
                height: 1,
            });
            star_areas.push(Rect {
                x: results_area.x,
                y: results_area.y + i as u16,
                width: 3,
                height: 1,
            });

            ListItem::new(line)
        })
        .collect();

    app.click_regions.find_modal.result_areas = result_areas;
    app.click_regions.find_modal.star_areas = star_areas;

    let list = List::new(items);
    frame.render_widget(list, results_area);

    let footer = if app.find_board_panel_open {
        Paragraph::new(Line::from(vec![
            Span::styled(" Esc", Style::default().fg(t.accent)),
            Span::styled(" close", Style::default().fg(t.text_dim)),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(" Enter", Style::default().fg(t.accent)),
            Span::styled(" select ", Style::default().fg(t.text_dim)),
            Span::styled(" s", Style::default().fg(t.accent)),
            Span::styled(" favorite ", Style::default().fg(t.text_dim)),
            Span::styled(" Esc", Style::default().fg(t.accent)),
            Span::styled(" close", Style::default().fg(t.text_dim)),
        ]))
    };
    frame.render_widget(footer.alignment(Alignment::Center), chunks[3]);
}

fn render_board_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    let items_area = chunks[0];
    let footer_area = chunks[1];

    let mut panel_item_areas: Vec<Rect> = Vec::new();

    if app.find_panel_loading {
        let loading = Paragraph::new("Loading boards...")
            .style(Style::default().fg(t.warning))
            .alignment(Alignment::Center);
        frame.render_widget(loading, items_area);
    } else {
        // Build item list: index 0 = "List" (backlog), then boards
        let total = app.find_panel_boards.len() + 1;
        if app.find_panel_selected.len() < total {
            app.find_panel_selected.resize(total, false);
        }

        let items: Vec<ListItem> = (0..total)
            .map(|i| {
                let is_cursor = i == app.find_panel_cursor;
                let is_checked = app.find_panel_selected.get(i).copied().unwrap_or(false);

                let checkbox = if is_checked { "[x]" } else { "[ ]" };

                let label = if i == 0 {
                    " List (Backlog)".to_string()
                } else {
                    let board = &app.find_panel_boards[i - 1];
                    format!(" {} ({})", board.name, board.board_type)
                };

                let style = if is_cursor {
                    Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
                } else if is_checked {
                    Style::default().fg(t.success)
                } else {
                    Style::default().fg(t.text)
                };

                let check_style = if is_checked {
                    Style::default().fg(t.success)
                } else {
                    Style::default().fg(t.text_dim)
                };

                let line = Line::from(vec![
                    Span::styled(format!(" {} ", checkbox), check_style),
                    Span::styled(label, style),
                ]);

                panel_item_areas.push(Rect {
                    x: items_area.x,
                    y: items_area.y + i as u16,
                    width: items_area.width,
                    height: 1,
                });

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, items_area);
    }

    app.click_regions.find_modal.panel_item_areas = panel_item_areas;

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Space", Style::default().fg(t.accent)),
        Span::styled(" toggle ", Style::default().fg(t.text_dim)),
        Span::styled(" Enter", Style::default().fg(t.accent)),
        Span::styled(" confirm ", Style::default().fg(t.text_dim)),
        Span::styled(" Esc", Style::default().fg(t.accent)),
        Span::styled(" back", Style::default().fg(t.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, footer_area);
}
