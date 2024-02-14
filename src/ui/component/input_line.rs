use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Borders};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tui_textarea::{CursorMove, TextArea};

pub struct InputLine<'a> {
    pub textarea: TextArea<'a>,
    pub needs_redraw: AtomicBool,
    redraw_terminal: Arc<AtomicBool>,
}

impl InputLine<'_> {
    pub fn new(redraw_terminal: Arc<AtomicBool>) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Target directory"));
        Self {
            textarea,
            needs_redraw: AtomicBool::new(false),
            redraw_terminal,
        }
    }

    pub fn widget(&self) -> impl Widget + '_ {
        self.textarea.widget()
    }

    pub fn get_contents(&self) -> String {
        self.textarea.lines()[0].clone()
    }

    pub fn ask_extract_destination(&mut self, suggested_name: &str) {
        let input_style = Style::default().fg(Color::Black).bg(Color::White);
        let border_style = Style::default().fg(Color::Yellow).bg(Color::Black);
        self.textarea = TextArea::from([suggested_name]);
        self.textarea
            .set_block(Block::default().borders(Borders::ALL).title("Target directory").border_style(border_style));
        self.textarea.set_cursor_line_style(input_style);
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.set_placeholder_text("Target directory");
    }
}
