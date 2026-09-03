mod component;
mod hotkeys;
mod main_ui;
mod rectangles;
pub mod sso;
mod tab;
mod tabs;

pub use main_ui::*;

use ratatui::Terminal;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::error::Error;
use std::io;
use std::thread;
use std::time::Duration;
use termion::event::Event;
use termion::input::TermRead;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

#[derive(Debug)]
pub enum TickEvent {
    Input(Event),
    Tick,
}

pub struct InputEvents {
    pub rx: mpsc::UnboundedReceiver<TickEvent>,
}

impl InputEvents {
    pub fn new() -> InputEvents {
        let tick_rate = Duration::from_millis(250);

        let (tx, rx) = mpsc::unbounded_channel();
        let _input_handle = {
            let tx = tx.clone();
            // TODO what are the tradeoffs between tokio/std threads here?
            thread::spawn(move || {
                let stdin = io::stdin();
                for event in stdin.events().flatten() {
                    if tx.send(TickEvent::Input(event)).is_err() {
                        return;
                    }
                }
            })
        };
        let _tick_handle = {
            thread::spawn(move || {
                loop {
                    if tx.send(TickEvent::Tick).is_err() {
                        break;
                    }
                    thread::sleep(tick_rate);
                }
            })
        };
        InputEvents { rx }
    }
}
