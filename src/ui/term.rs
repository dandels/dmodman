use ratatui::Terminal;
use ratatui::backend::TermionBackend;
use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::{AlternateScreen, IntoAlternateScreen};

use std::error::Error;
use std::io::Stdout;

type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;

pub struct TermUI {
    terminal: Terminal<Backend>,
}

impl TermUI {
    pub fn new() -> Result<TermUI, Box<dyn Error>> {
        let stdout = std::io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        /* The alternate screen restores terminal state when dropped.
         * Disable it if you need to see rust backtraces */
        let stdout = stdout.into_alternate_screen()?;
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;
        Ok(Self { terminal })
    }

    pub fn render(&self) {}
}
