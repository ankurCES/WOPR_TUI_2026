use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

use crate::game::events::GameEvent;

#[derive(Debug)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    Resize(u16, u16),
    GameEvent(GameEvent),
    Quit,
}

pub fn create_channel() -> (mpsc::Sender<AppEvent>, mpsc::Receiver<AppEvent>) {
    mpsc::channel(256)
}
