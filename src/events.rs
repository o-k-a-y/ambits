use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};

use crate::ingest::AgentToolCall;

/// Unified application event.
#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    FileChanged(PathBuf),
    AgentEvent(AgentToolCall),
    Tick,
}

/// Spawn a thread that polls crossterm key events and sends them to the channel.
pub fn spawn_key_reader(tx: mpsc::Sender<AppEvent>) {
    std::thread::spawn(move || loop {
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if tx.send(AppEvent::Key(key)).is_err() {
                    break;
                }
            }
        }
    });
}

/// Spawn a tick timer that sends Tick events at the given interval.
pub fn spawn_tick_timer(tx: mpsc::Sender<AppEvent>, interval: Duration) {
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        if tx.send(AppEvent::Tick).is_err() {
            break;
        }
    });
}
