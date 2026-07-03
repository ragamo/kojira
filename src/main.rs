mod app;
mod config;
mod event;
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
use event::event_loop;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cfg = config::load_config()?;
    let mut app = App::new(cfg);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let _ = event_loop(tx).await;
    });

    while app.running {
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        if let Some(event) = rx.recv().await {
            app.handle_event(event)?;
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
