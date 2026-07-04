use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    render_header(frame, app, chunks[0]);

    let t = app.theme;
    let header_bg = if app.header_bg_soft { t.bg } else { t.header_bg };

    // separator above tabs: ▄ top half = header bg, bottom half = tab bar bg
    let sep_top = chunks[1];
    frame.render_widget(
        Paragraph::new(Span::styled(
            "▄".repeat(sep_top.width as usize),
            Style::default().fg(t.header_bg).bg(header_bg),
        )),
        sep_top,
    );

    render_tabs(frame, app, chunks[2]);

    // separator below tabs: ▀ top half = tab bar bg, bottom half = content bg
    let sep_bot = chunks[3];
    frame.render_widget(
        Paragraph::new(Span::styled(
            "▀".repeat(sep_bot.width as usize),
            Style::default().fg(t.header_bg).bg(t.bg),
        ))
        .style(Style::default().bg(t.bg)),
        sep_bot,
    );

    // Fill content area with t.bg when solid background is enabled
    if app.content_bg_solid {
        frame.render_widget(Block::default().style(Style::default().bg(t.bg)), chunks[4]);
    }

    if app.detail_open {
        if app.detail_height == 0 {
            app.detail_height = (chunks[4].height * 70 / 100).max(10);
        }
        let detail_h = app.detail_height.min(chunks[4].height.saturating_sub(4));
        let content_h = chunks[4].height.saturating_sub(detail_h);
        let splits = Layout::vertical([
            Constraint::Length(content_h),
            Constraint::Length(detail_h),
        ])
        .split(chunks[4]);
        render_content(frame, app, splits[0]);
        crate::ui::detail_panel::render(frame, app, splits[1]);
        let border_y = splits[0].y + splits[0].height.saturating_sub(1);
        app.detail_resize_area = Some(Rect {
            x: splits[1].x,
            y: border_y,
            width: splits[1].width,
            height: 2,
        });
    } else if app.create_modal_open {
        if app.create_panel_height == 0 {
            app.create_panel_height = (frame.area().height * 60 / 100).max(12);
        }
        let panel_h = app.create_panel_height.min(chunks[4].height.saturating_sub(4));
        let content_h = chunks[4].height.saturating_sub(panel_h);
        let splits = Layout::vertical([
            Constraint::Length(content_h),
            Constraint::Length(panel_h),
        ])
        .split(chunks[4]);
        render_content(frame, app, splits[0]);
        crate::ui::create_modal::render(frame, app, splits[1]);
        let border_y = splits[0].y + splits[0].height.saturating_sub(1);
        app.create_resize_area = Some(Rect {
            x: splits[1].x,
            y: border_y,
            width: splits[1].width,
            height: 2,
        });
    } else {
        app.detail_height = 0;
        render_content(frame, app, chunks[4]);
    }

    render_footer(frame, app.theme, chunks[5]);

}

fn render_header(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = app.theme;
    let header_bg = if app.header_bg_soft { t.bg } else { t.header_bg };

    let bg_block = Block::default().style(Style::default().bg(header_bg));
    frame.render_widget(bg_block, area);

    // Left: " kojira │ settings │ create"
    let settings_label = " settings";
    let settings_w = settings_label.len() as u16;
    let create_label = " create";
    let create_w = create_label.len() as u16;
    let left_line = Line::from(vec![
        Span::styled(" kojira", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" │", Style::default().fg(t.text_dim)),
        Span::styled(settings_label, Style::default().fg(t.text_dim)),
        Span::styled(" │", Style::default().fg(t.text_dim)),
        Span::styled(create_label, Style::default().fg(t.text_dim)),
    ]);
    frame.render_widget(Paragraph::new(left_line), area);

    // Right: "@author  logout" or "login"
    let right_line = if app.logged_in {
        let name = app
            .user_display_name
            .as_deref()
            .or(app.user_email.as_deref())
            .unwrap_or("user");
        Line::from(vec![
            Span::styled(format!("@{}", name), Style::default().fg(t.success)),
            Span::styled("  logout ", Style::default().fg(t.text_dim)),
        ])
    } else {
        Line::from(vec![
            Span::styled("login ", Style::default().fg(t.text_dim)),
        ])
    };
    frame.render_widget(Paragraph::new(right_line).alignment(Alignment::Right), area);

    // Click regions
    // "settings" starts at x = len(" kojira │") = 9
    let settings_x = area.x + 9;
    app.click_regions.header.settings_link = Some(Rect {
        x: settings_x,
        y: area.y,
        width: settings_w,
        height: 1,
    });
    // "create" starts after " settings │" = settings_x + settings_w + 2
    let create_x = settings_x + settings_w + 2;
    app.click_regions.header.create_link = Some(Rect {
        x: create_x,
        y: area.y,
        width: create_w,
        height: 1,
    });

    if app.logged_in {
        let logout_w = 7u16; // "logout "
        app.click_regions.header.logout_link = Some(Rect {
            x: area.x + area.width.saturating_sub(logout_w),
            y: area.y,
            width: logout_w,
            height: 1,
        });
    } else {
        let login_w = 6u16; // "login "
        app.click_regions.header.login_link = Some(Rect {
            x: area.x + area.width.saturating_sub(login_w),
            y: area.y,
            width: login_w,
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

    let drag_insert = if app.tab_dragging.is_some() { app.tab_drag_insert_pos } else { None };

    for (global_idx, tab) in app.tab_order.clone().iter().enumerate() {
        // Draw insert indicator before this tab if needed
        if drag_insert == Some(global_idx) {
            let ind_x = x.saturating_sub(1);
            let ind_area = Rect { x: ind_x, y: area.y, width: 1, height: 1 };
            frame.render_widget(
                Paragraph::new(Span::styled("|", Style::default().fg(t.accent).add_modifier(Modifier::BOLD))),
                ind_area,
            );
        }

        let (label, is_active) = match tab {
            Tab::List(id) => {
                let label = app.list_tabs.iter()
                    .find(|t| t.id == *id)
                    .map(|t| format!(" {} ", t.project_key))
                    .unwrap_or_default();
                (label, app.active_tab == Tab::List(*id))
            }
            Tab::Board(id) => {
                let label = app.board_tabs.iter()
                    .find(|t| t.board_id == *id)
                    .map(|t| format!(" {} ", t.board_name))
                    .unwrap_or_default();
                (label, app.active_tab == Tab::Board(*id))
            }
        };
        if label.is_empty() { continue; }
        let w = label.len() as u16;
        if x + w > area.x + area.width.saturating_sub(15) {
            break;
        }
        let style = if is_active {
            Style::default().fg(t.bg).bg(t.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_dim)
        };
        let tab_area = Rect { x, y: area.y, width: w, height: 1 };
        frame.render_widget(Paragraph::new(Span::styled(&label, style)), tab_area);
        tab_areas.push((tab_area, global_idx));
        x += w + 1;
    }

    // Insert indicator at the end
    if drag_insert == Some(app.tab_order.len()) {
        let ind_x = x.saturating_sub(1);
        let ind_area = Rect { x: ind_x, y: area.y, width: 1, height: 1 };
        frame.render_widget(
            Paragraph::new(Span::styled("|", Style::default().fg(t.accent).add_modifier(Modifier::BOLD))),
            ind_area,
        );
    }

    // "+ new tab" button
    let add_label = " + new tab ";
    let add_w = add_label.len() as u16;
    let add_area = Rect { x, y: area.y, width: add_w, height: 1 };
    let btn_bg = match (t.header_bg, t.bg) {
        (Color::Rgb(hr, hg, hb), Color::Rgb(br, bg_, bb)) => Color::Rgb(
            ((hr as u16 + br as u16) / 2) as u8,
            ((hg as u16 + bg_ as u16) / 2) as u8,
            ((hb as u16 + bb as u16) / 2) as u8,
        ),
        _ => t.header_bg,
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" + ", Style::default().fg(t.accent).bg(btn_bg)),
            Span::styled("new tab ", Style::default().fg(t.text_dim).bg(btn_bg)),
        ])),
        add_area,
    );
    app.click_regions.header.tab_add = Some(add_area);

    app.click_regions.header.tab_areas = tab_areas;
    app.click_regions.header.tab_row_y = Some(area.y);
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
        Span::styled("x", Style::default().fg(theme.accent)),
        Span::styled(" close tab ", Style::default().fg(theme.text_dim)),
        Span::styled("n", Style::default().fg(theme.accent)),
        Span::styled(" new tab ", Style::default().fg(theme.text_dim)),
        Span::styled(",", Style::default().fg(theme.accent)),
        Span::styled(" settings", Style::default().fg(theme.text_dim)),
    ];

    let footer = Paragraph::new(Line::from(keys))
        .style(Style::default().bg(theme.bg));
    frame.render_widget(footer, area);
}

use crate::app::Tab;
