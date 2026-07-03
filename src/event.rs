use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyEvent, MouseEvent};
use futures::StreamExt;
use tokio::sync::mpsc;

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    Message(AppMessage),
}

pub enum AppMessage {
    Tick,
}

pub async fn event_loop(tx: mpsc::UnboundedSender<AppEvent>) -> Result<()> {
    let mut reader = EventStream::new();
    let tick_rate = Duration::from_secs(10);
    let mut tick_interval = tokio::time::interval(tick_rate);

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
            event = reader.next() => {
                match event {
                    Some(Ok(Event::Key(key))) => {
                        if tx.send(AppEvent::Key(key)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Event::Mouse(mouse))) => {
                        if tx.send(AppEvent::Mouse(mouse)).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Event::Resize(w, h))) => {
                        if tx.send(AppEvent::Resize(w, h)).is_err() {
                            break;
                        }
                    }
                    Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
