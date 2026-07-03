use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let t = app.theme;
    let screen = frame.area();
    let w = (screen.width * 55 / 100).max(40).min(screen.width);
    let h = (screen.height * 50 / 100).max(12).min(screen.height);
    let area = Rect {
        x: screen.x + screen.width.saturating_sub(w) / 2,
        y: screen.y + screen.height.saturating_sub(h) / 2,
        width: w,
        height: h,
    };

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Find Project ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(t.accent));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

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

    app.click_regions.find_modal.bounds = Some(area);
    app.click_regions.find_modal.result_areas = result_areas;
    app.click_regions.find_modal.star_areas = star_areas;

    let list = List::new(items);
    frame.render_widget(list, results_area);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Enter", Style::default().fg(t.accent)),
        Span::styled(" search/select ", Style::default().fg(t.text_dim)),
        Span::styled(" s", Style::default().fg(t.accent)),
        Span::styled(" favorite ", Style::default().fg(t.text_dim)),
        Span::styled(" Esc", Style::default().fg(t.accent)),
        Span::styled(" close", Style::default().fg(t.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}
