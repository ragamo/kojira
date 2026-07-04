use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

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

    if app.detail_open {
        if app.detail_height == 0 {
            app.detail_height = (chunks[2].height * 70 / 100).max(10);
        }
        let detail_h = app.detail_height.min(chunks[2].height.saturating_sub(4));
        let content_h = chunks[2].height.saturating_sub(detail_h);
        let splits = Layout::vertical([
            Constraint::Length(content_h),
            Constraint::Length(detail_h),
        ])
        .split(chunks[2]);
        render_content(frame, app, splits[0]);
        crate::ui::detail_panel::render(frame, app, splits[1]);
        // Extend resize area to cover bottom border of content + top border of panel
        let border_y = splits[0].y + splits[0].height.saturating_sub(1);
        app.detail_resize_area = Some(Rect {
            x: splits[1].x,
            y: border_y,
            width: splits[1].width,
            height: 2,
        });
    } else {
        app.detail_height = 0;
        render_content(frame, app, chunks[2]);
    }

    render_footer(frame, app.theme, chunks[3]);

}

fn render_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;
    let header_bg = if app.header_bg_soft { t.bg } else { t.header_bg };

    let bg_block = Block::default().style(Style::default().bg(header_bg));
    frame.render_widget(bg_block, area);

    let right_area = area;

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
    let mut global_idx = 0usize;

    // List tabs
    for lt in &app.list_tabs.clone() {
        let label = format!(" {} ", lt.project_key);
        let w = label.len() as u16;
        if x + w > area.x + area.width.saturating_sub(15) {
            break;
        }
        let is_active = app.active_tab == Tab::List(lt.id);
        let style = if is_active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        };
        let tab_area = Rect { x, y: area.y, width: w, height: 1 };
        frame.render_widget(Paragraph::new(Span::styled(&label, style)), tab_area);
        tab_areas.push((tab_area, global_idx));
        x += w + 1;
        global_idx += 1;
    }

    // Board tabs
    for bt in &app.board_tabs.clone() {
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
        tab_areas.push((tab_area, global_idx));
        x += w + 1;
        global_idx += 1;
    }

    // "+ new tab" button
    let add_label = " + new tab ";
    let add_w = add_label.len() as u16;
    let add_area = Rect { x, y: area.y, width: add_w, height: 1 };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" + ", Style::default().fg(t.accent)),
            Span::styled("new tab ", Style::default().fg(t.text_dim)),
        ])),
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
    match app.active_tab.clone() {
        Tab::List(id) => crate::ui::backlog_view::render(frame, app, id, area),
        Tab::Board(id) => crate::ui::board_view::render(frame, app, id, area),
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

use crate::app::Tab;
