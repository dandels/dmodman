use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::Widget;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tui_textarea::TextArea;

pub struct InputLine<'a> {
    pub textarea: TextArea<'a>,
    pub needs_redraw: AtomicBool,
    redraw_terminal: Arc<AtomicBool>,
}

impl InputLine<'_> {
    pub fn new(redraw_terminal: Arc<AtomicBool>) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Target directory"));
        textarea.set_cursor_line_style(Style::default());
        Self {
            textarea,
            needs_redraw: AtomicBool::new(false),
            redraw_terminal,
        }
    }

    pub fn delete_char(&mut self) -> bool {
        self.needs_redraw.store(true, Ordering::Relaxed);
        self.textarea.delete_char()
    }

    pub fn insert_char(&mut self, ch: char) {
        self.textarea.insert_char(ch);
        self.needs_redraw.store(true, Ordering::Relaxed);
    }


    pub fn widget(&self) -> impl Widget + '_ {
        self.textarea.widget()
    }

    pub fn clear(&mut self) -> bool {
        self.redraw_terminal.store(true, Ordering::Relaxed);
        self.textarea.delete_str(0)
    }

    pub fn get_contents(&self) -> String {
        self.textarea.lines()[0].clone()
    }

    pub fn get_file_name(&mut self, suggested_name: &str) {
        self.clear();
        self.textarea.set_placeholder_text("Target directory");
        self.textarea.insert_str(suggested_name);
    }

    pub fn validate(&self) {}
}