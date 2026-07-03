use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::App;
use crate::theme;

const TABS: &[&str] = &["themes", "config", "about"];

pub fn render(frame: &mut Frame, app: &mut App) {
    let t = app.theme;
    let screen = frame.area();
    let w = 70u16.min(screen.width);
    let h = 20u16.min(screen.height);
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

    let outer = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);

    let body = outer[0];

    let title_area = Rect { x: body.x + 1, y: body.y, width: body.width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(
            "settings",
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        )),
        title_area,
    );

    let tab_area = Rect { x: body.x, y: body.y + 1, width: body.width, height: 1 };
    let mut tab_spans: Vec<Span> = Vec::new();
    let mut tab_click_areas: Vec<Rect> = Vec::new();
    let mut x_offset = tab_area.x + 1;
    for (i, &tab) in TABS.iter().enumerate() {
        let is_active = i == app.settings_selected;
        let style = if is_active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        };
        if i == 0 {
            tab_spans.push(Span::raw(" "));
        }
        tab_click_areas.push(Rect {
            x: x_offset,
            y: tab_area.y,
            width: tab.len() as u16,
            height: 1,
        });
        tab_spans.push(Span::styled(tab.to_string(), style));
        tab_spans.push(Span::raw("   "));
        x_offset += tab.len() as u16 + 3;
    }
    app.settings_tab_areas = tab_click_areas;
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), tab_area);

    let divider_area = Rect { x: body.x, y: body.y + 2, width: body.width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(body.width as usize),
            Style::default().fg(t.border),
        )),
        divider_area,
    );

    let content_area = Rect {
        x: body.x,
        y: body.y + 3,
        width: body.width,
        height: body.height.saturating_sub(3),
    };

    match app.settings_selected {
        0 => render_theme_tab(frame, app, content_area),
        1 => render_config_tab(frame, app, content_area),
        _ => render_about_tab(frame, app, content_area),
    }

    let footer_area = outer[1];
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ←→", Style::default().fg(t.accent)),
            Span::styled(" section  ", Style::default().fg(t.text_dim)),
            Span::styled("↑↓", Style::default().fg(t.accent)),
            Span::styled(" select", Style::default().fg(t.text_dim)),
        ])),
        footer_area,
    );

    let apply_label = " ↵ apply ";
    let close_label = " esc close ";
    let total: u16 = 9 + 1 + 11;
    let start_x = footer_area.x + footer_area.width.saturating_sub(total) / 2;
    app.settings_apply_area = Some(Rect { x: start_x, y: footer_area.y, width: 9, height: 1 });
    app.settings_close_area = Some(Rect { x: start_x + 10, y: footer_area.y, width: 11, height: 1 });

    let apply_footer = Paragraph::new(Line::from(vec![
        Span::styled(apply_label, Style::default().fg(t.bg).bg(t.accent)),
        Span::raw(" "),
        Span::styled(close_label, Style::default().fg(t.text).bg(t.border)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(apply_footer, footer_area);
}

fn render_theme_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let items: Vec<ListItem> = theme::ALL_THEMES
        .iter()
        .enumerate()
        .map(|(i, th)| {
            let is_selected = i == app.theme_selected;
            let is_confirmed = i == app.theme_confirmed;
            let style = if is_selected {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            let prefix = if is_selected { " ▸ " } else { "   " };
            let suffix = if is_confirmed { " ✓" } else { "" };
            ListItem::new(format!("{}{}{}", prefix, th.name, suffix)).style(style)
        })
        .collect();

    let list_area = Rect {
        x: area.x,
        y: area.y,
        width: (area.width / 3).max(24),
        height: area.height,
    };
    frame.render_widget(List::new(items), list_area);

    app.settings_theme_areas = (0..theme::ALL_THEMES.len())
        .filter(|i| (*i as u16) < list_area.height)
        .map(|i| Rect {
            x: list_area.x,
            y: list_area.y + i as u16,
            width: list_area.width,
            height: 1,
        })
        .collect();
}

fn render_config_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let val_style = |active: bool| {
        if active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        }
    };

    let header_y = area.y + 1;
    let header_label = "Header background: ";
    let soft_x = area.x + 3 + header_label.len() as u16;
    let hard_x = soft_x + 6 + 1;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ▸ ", Style::default().fg(t.accent)),
            Span::styled(header_label, Style::default().fg(t.text)),
            Span::styled(" soft ", val_style(app.header_bg_soft)),
            Span::raw(" "),
            Span::styled(" hard ", val_style(!app.header_bg_soft)),
        ])),
        Rect {
            x: area.x,
            y: header_y,
            width: area.width.saturating_sub(2),
            height: 1,
        },
    );
    app.settings_header_soft_area = Some(Rect { x: soft_x, y: header_y, width: 6, height: 1 });
    app.settings_header_hard_area = Some(Rect { x: hard_x, y: header_y, width: 6, height: 1 });

    let hint_area = Rect {
        x: area.x + 3,
        y: area.y + 3,
        width: area.width.saturating_sub(4),
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("space", Style::default().fg(t.accent)),
            Span::styled(" toggle  ", Style::default().fg(t.text_dim)),
            Span::styled("↵", Style::default().fg(t.accent)),
            Span::styled(" save", Style::default().fg(t.text_dim)),
        ])),
        hint_area,
    );
}

fn render_about_tab(frame: &mut Frame, app: &App, area: Rect) {
    let t = app.theme;

    let lines = vec![
        Line::from(Span::styled(
            "kojira",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("Jira TUI", Style::default().fg(t.text_dim))),
        Line::from(""),
        Line::from(vec![
            Span::styled("made by ", Style::default().fg(t.text)),
            Span::styled(
                "@ragamo",
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "https://github.com/ragamo/kojira",
            Style::default().fg(t.text_dim),
        )),
    ];

    let about_area = Rect {
        x: area.x + 3,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(1),
    };
    frame.render_widget(Paragraph::new(lines), about_area);
}
