mod component;
mod event;
mod main_ui;
mod rectangles;
pub mod sso;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use main_ui::MainUI;
use ratatui::backend::{Backend, TermionBackend};
use ratatui::Terminal;
use signal_hook_tokio::Signals;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tokio_stream::StreamExt;

pub fn term_setup() -> Result<Terminal<impl Backend>, Box<dyn Error>> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub async fn handle_sigwinch(mut signals: Signals, is_window_resized: Arc<AtomicBool>) {
    while signals.next().await.is_some() {
        is_window_resized.store(true, Ordering::Relaxed);
    }
}