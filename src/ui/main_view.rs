use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    render_header(frame, app, chunks[0]);
    render_tabs(frame, app, chunks[1]);
    render_content(frame, app, chunks[2]);
    render_footer(frame, app.theme, chunks[3]);

    if app.project_selector_open {
        render_project_dropdown(frame, app, chunks[0]);
    }
}

fn render_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;
    let header_bg = if app.header_bg_soft { t.bg } else { t.header_bg };

    let bg_block = Block::default().style(Style::default().bg(header_bg));
    frame.render_widget(bg_block, area);

    let header_layout = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Min(20),
    ])
    .split(area);

    let project_label = app
        .projects
        .get(app.selected_project)
        .map(|p| format!("{} - {}", p.key, p.name))
        .unwrap_or_else(|| "No project".into());

    let max_label_len = 34usize;
    let display_label = if project_label.len() > max_label_len {
        format!("{}…", &project_label[..max_label_len - 1])
    } else {
        project_label
    };

    let selector_text = format!(" ⏷ {} ", display_label);
    let selector_width = 40u16.min(header_layout[0].width.saturating_sub(7));

    let selector = Paragraph::new(Span::styled(
        &selector_text,
        Style::default()
            .fg(t.text)
            .bg(header_bg)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.accent)),
    );

    let selector_area = Rect {
        x: header_layout[0].x,
        y: header_layout[0].y,
        width: selector_width,
        height: header_layout[0].height,
    };
    frame.render_widget(selector, selector_area);
    app.click_regions.header.project_selector = Some(selector_area);

    let find_link = Paragraph::new(Span::styled(
        " Find",
        Style::default().fg(t.accent),
    ));
    let find_area = Rect {
        x: selector_area.x + selector_area.width + 1,
        y: selector_area.y + 1,
        width: 5,
        height: 1,
    };
    frame.render_widget(find_link, find_area);
    app.click_regions.header.find_link = Some(find_area);

    let right_area = header_layout[1];

    let right_text = if app.logged_in {
        let name = app
            .user_display_name
            .as_deref()
            .or(app.user_email.as_deref())
            .unwrap_or("user");
        vec![
            Span::styled("kojira", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(t.text_dim)),
            Span::styled(name, Style::default().fg(t.success)),
            Span::styled("  logout", Style::default().fg(t.text_dim)),
            Span::raw(" "),
        ]
    } else {
        vec![
            Span::styled("kojira", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(t.text_dim)),
            Span::styled("login", Style::default().fg(t.text_dim)),
            Span::raw(" "),
        ]
    };

    let right_widget = Paragraph::new(Line::from(right_text)).alignment(Alignment::Right);
    frame.render_widget(right_widget, right_area);

    if app.logged_in {
        let logout_width = 8u16;
        app.click_regions.header.logout_link = Some(Rect {
            x: right_area.x + right_area.width.saturating_sub(logout_width + 1),
            y: right_area.y,
            width: logout_width,
            height: 1,
        });
    } else {
        let login_width = 6u16;
        app.click_regions.header.login_link = Some(Rect {
            x: right_area.x + right_area.width.saturating_sub(login_width + 1),
            y: right_area.y,
            width: login_width,
            height: 1,
        });
    }
}

fn render_tabs(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;

    let bg_block = Block::default().style(Style::default().bg(t.header_bg));
    frame.render_widget(bg_block, area);

    let mut x = area.x;
    let mut tab_areas: Vec<(Rect, usize)> = Vec::new();

    // Tab 0: Backlog
    let backlog_label = " list ";
    let backlog_w = backlog_label.len() as u16;
    let backlog_style = if app.active_tab == Tab::Backlog {
        Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text_dim)
    };
    let backlog_area = Rect { x, y: area.y, width: backlog_w, height: 1 };
    frame.render_widget(Paragraph::new(Span::styled(backlog_label, backlog_style)), backlog_area);
    tab_areas.push((backlog_area, 0));
    x += backlog_w + 1;

    // Board tabs
    for (i, bt) in app.board_tabs.iter().enumerate() {
        let label = format!(" {} ", bt.board_name);
        let w = label.len() as u16;
        if x + w > area.x + area.width.saturating_sub(15) {
            break;
        }
        let is_active = app.active_tab == Tab::Board(bt.board_id);
        let style = if is_active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        };
        let tab_area = Rect { x, y: area.y, width: w, height: 1 };
        frame.render_widget(Paragraph::new(Span::styled(&label, style)), tab_area);
        tab_areas.push((tab_area, i + 1));
        x += w + 1;
    }

    // "+" button
    let add_label = " + ";
    let add_w = add_label.len() as u16;
    let add_area = Rect { x, y: area.y, width: add_w, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(add_label, Style::default().fg(t.accent))),
        add_area,
    );
    app.click_regions.header.tab_add = Some(add_area);

    app.click_regions.header.tab_areas = tab_areas;

    // Settings link on the header row above
    let remaining = Rect {
        x: add_area.x + add_w,
        y: area.y.saturating_sub(1),
        width: area.x + area.width - (add_area.x + add_w),
        height: 1,
    };
    let settings_link = Paragraph::new(Span::styled(
        "settings ",
        Style::default().fg(t.text_dim),
    ))
    .alignment(Alignment::Right);
    frame.render_widget(settings_link, remaining);

    let settings_width = 9u16;
    app.click_regions.header.settings_link = Some(Rect {
        x: remaining.x + remaining.width.saturating_sub(settings_width),
        y: remaining.y,
        width: settings_width,
        height: 1,
    });
}

fn render_content(frame: &mut Frame, app: &mut App, area: Rect) {
    match &app.active_tab.clone() {
        Tab::Backlog => crate::ui::backlog_view::render(frame, app, area),
        Tab::Board(id) => crate::ui::board_view::render(frame, app, *id, area),
    }
}

fn render_footer(frame: &mut Frame, theme: &Theme, area: Rect) {
    let keys = vec![
        Span::styled(" q", Style::default().fg(theme.accent)),
        Span::styled(" quit ", Style::default().fg(theme.text_dim)),
        Span::styled("Tab", Style::default().fg(theme.accent)),
        Span::styled(" switch tab ", Style::default().fg(theme.text_dim)),
        Span::styled("p", Style::default().fg(theme.accent)),
        Span::styled(" projects ", Style::default().fg(theme.text_dim)),
        Span::styled("f", Style::default().fg(theme.accent)),
        Span::styled(" find ", Style::default().fg(theme.text_dim)),
        Span::styled(",", Style::default().fg(theme.accent)),
        Span::styled(" settings", Style::default().fg(theme.text_dim)),
    ];

    let footer = Paragraph::new(Line::from(keys))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(footer, area);
}

fn render_project_dropdown(frame: &mut Frame, app: &mut App, header_area: Rect) {
    let t = app.theme;

    if app.projects.is_empty() {
        return;
    }

    let max_item_len = app
        .projects
        .iter()
        .map(|p| p.key.len() as u16 + p.name.len() as u16 + 16)
        .max()
        .unwrap_or(30);
    let dropdown_width = (max_item_len + 2).max(50).min(header_area.width);
    let dropdown_height = (app.projects.len() as u16 + 2).min(10);

    let dropdown_area = Rect {
        x: header_area.x,
        y: header_area.y + header_area.height,
        width: dropdown_width,
        height: dropdown_height,
    };

    frame.render_widget(ratatui::widgets::Clear, dropdown_area);

    let items: Vec<ratatui::widgets::ListItem> = app
        .projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let is_selected = i == app.selected_project;
            let style = if is_selected {
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.text)
            };
            let prefix = if is_selected { " ▸ ★ " } else { "   ★ " };
            let suffix = if is_selected { "  [s] unfav" } else { "" };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(t.warning)),
                Span::styled(format!("{} - {}", p.key, p.name), style),
                Span::styled(suffix, Style::default().fg(t.text_dim)),
            ]);
            ratatui::widgets::ListItem::new(line)
        })
        .collect();

    let list = ratatui::widgets::List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(t.accent))
            .title(" Favorites ")
            .title_style(Style::default().fg(t.accent))
            .style(Style::default().bg(t.bg)),
    );

    frame.render_widget(list, dropdown_area);

    app.click_regions.project_dropdown.bounds = Some(dropdown_area);
    app.click_regions.project_dropdown.items = (0..app.projects.len())
        .map(|i| Rect {
            x: dropdown_area.x + 1,
            y: dropdown_area.y + 1 + i as u16,
            width: dropdown_area.width.saturating_sub(2),
            height: 1,
        })
        .collect();
}

use crate::app::Tab;
