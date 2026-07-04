mod app;
mod config;
mod event;
mod provider;
mod table_nav;
mod theme;
mod ui;

use std::io;

use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use app::App;
use event::{AppEvent, event_loop};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cfg = config::load_config()?;

    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel();
    let mut app = App::new(cfg, msg_tx);

    if app.logged_in && !app.projects.is_empty() {
        app.load_column_order();
        app.load_all_list_tabs();
        let board_ids: Vec<u64> = app.board_tabs.iter().map(|b| b.board_id).collect();
        for id in board_ids {
            app.load_board_data(id);
        }
    }

    if app.logged_in && app.list_tabs.is_empty() && app.board_tabs.is_empty() {
        app.open_find();
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (event_tx, mut event_rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let _ = event_loop(event_tx).await;
    });

    while app.running {
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        tokio::select! {
            Some(event) = event_rx.recv() => {
                app.handle_event(event)?;
            }
            Some(msg) = msg_rx.recv() => {
                app.handle_event(AppEvent::Message(msg))?;
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
