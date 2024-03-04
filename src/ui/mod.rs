mod component;
mod hotkeys;
mod main_ui;
mod navigation;
mod rectangles;
pub mod sso;

pub use main_ui::*;

use ratatui::backend::{Backend, TermionBackend};
use ratatui::Terminal;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::error::Error;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use termion::event::Event;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

pub fn term_setup() -> Result<Terminal<impl Backend>, Box<dyn Error>> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    /* The alternate screen restores terminal state when dropped.
     * Disable it if you need to see rust backtraces */
    let stdout = stdout.into_alternate_screen()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub async fn handle_sigwinch(is_window_resized: Arc<AtomicBool>) {
    let mut signals = Signals::new([SIGWINCH]).unwrap();
    while signals.next().await.is_some() {
        is_window_resized.store(true, Ordering::Relaxed);
    }
}

pub enum TickEvent {
    Input(Event),
    Tick,
}

pub struct Events {
    rx: mpsc::UnboundedReceiver<TickEvent>,
}

impl Events {
    pub fn new() -> Events {
        let tick_rate = Duration::from_millis(250);

        let (tx, rx) = mpsc::unbounded_channel();
        let _input_handle = {
            let tx = tx.clone();
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
            thread::spawn(move || loop {
                if tx.send(TickEvent::Tick).is_err() {
                    break;
                }
                thread::sleep(tick_rate);
            })
        };
        Events { rx }
    }

    pub async fn next(&mut self) -> Option<TickEvent> {
        self.rx.recv().await
    }
}
