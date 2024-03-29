mod component;
mod event;
mod hotkeys;
mod main_ui;
mod rectangles;
pub mod sso;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub use main_ui::*;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use ratatui::backend::{Backend, TermionBackend};
use ratatui::Terminal;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
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
