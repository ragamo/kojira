use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::app::{App, AuthField};

pub fn render(frame: &mut Frame, app: &App) {
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
    let content_width = body.width.saturating_sub(4);

    // Title
    let title_area = Rect { x: body.x + 1, y: body.y, width: body.width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled(
            "login",
            Style::default().fg(t.text).add_modifier(Modifier::BOLD),
        )),
        title_area,
    );

    // --- Subdomain field ---
    let sub_label_y = body.y + 2;
    let sub_label_area = Rect { x: body.x + 2, y: sub_label_y, width: content_width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("Instance", Style::default().fg(t.text))),
        sub_label_area,
    );

    let sub_input_y = sub_label_y + 1;
    let sub_border_color = if app.auth_field == AuthField::Subdomain {
        t.accent
    } else {
        t.border
    };
    let sub_input_area = Rect { x: body.x + 2, y: sub_input_y, width: content_width, height: 3 };

    let sub_cursor = if !app.is_validating && app.auth_field == AuthField::Subdomain {
        "▌"
    } else {
        ""
    };
    let sub_widget = Paragraph::new(Line::from(vec![
        Span::styled("https://", Style::default().fg(t.text_dim)),
        Span::styled(
            format!("{}{}", app.subdomain_input, sub_cursor),
            Style::default().fg(t.text),
        ),
        Span::styled(".atlassian.net", Style::default().fg(t.text_dim)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(sub_border_color)),
    )
    .style(Style::default().fg(t.text));
    frame.render_widget(sub_widget, sub_input_area);

    // --- Email field ---
    let email_label_y = sub_input_y + 3;
    let email_label_area = Rect { x: body.x + 2, y: email_label_y, width: content_width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("Email", Style::default().fg(t.text))),
        email_label_area,
    );

    let email_input_y = email_label_y + 1;
    let email_border_color = if app.auth_field == AuthField::Email {
        t.accent
    } else {
        t.border
    };
    let email_input_area = Rect { x: body.x + 2, y: email_input_y, width: content_width, height: 3 };
    let email_cursor = if !app.is_validating && app.auth_field == AuthField::Email {
        "▌"
    } else {
        ""
    };
    let email_widget = Paragraph::new(format!("{}{}", app.email_input, email_cursor))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(email_border_color)),
        )
        .style(Style::default().fg(t.text));
    frame.render_widget(email_widget, email_input_area);

    // --- Token field ---
    let token_label_y = email_input_y + 3;
    let token_label_area = Rect { x: body.x + 2, y: token_label_y, width: content_width, height: 1 };
    frame.render_widget(
        Paragraph::new(Span::styled("API Token", Style::default().fg(t.text))),
        token_label_area,
    );

    let token_input_y = token_label_y + 1;
    let token_border_color = if app.auth_field == AuthField::Token {
        t.accent
    } else {
        t.border
    };
    let token_input_area = Rect { x: body.x + 2, y: token_input_y, width: content_width, height: 3 };
    let masked: String = "●".repeat(app.token_input.len());
    let token_cursor = if !app.is_validating && app.auth_field == AuthField::Token {
        "▌"
    } else {
        ""
    };
    let token_widget = Paragraph::new(format!("{}{}", masked, token_cursor))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(token_border_color)),
        )
        .style(Style::default().fg(t.warning));
    frame.render_widget(token_widget, token_input_area);

    // --- Error message ---
    let msg_y = token_input_y + 3;
    let msg_area = Rect { x: body.x + 2, y: msg_y, width: content_width, height: 1 };
    if let Some(ref error) = app.auth_error {
        frame.render_widget(
            Paragraph::new(error.as_str())
                .style(Style::default().fg(t.error))
                .wrap(Wrap { trim: true }),
            msg_area,
        );
    }

    // --- Footer ---
    let footer_area = outer[1];
    if app.is_validating {
        frame.render_widget(
            Paragraph::new(Span::styled(
                " validating...",
                Style::default().fg(t.text_dim),
            )),
            footer_area,
        );
    } else {
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" Tab", Style::default().fg(t.accent)),
            Span::styled(" switch  ", Style::default().fg(t.text_dim)),
            Span::styled(" ↵ submit ", Style::default().fg(t.bg).bg(t.accent)),
            Span::raw(" "),
            Span::styled(" esc close ", Style::default().fg(t.text).bg(t.border)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, footer_area);
    }
}
