use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Borders};
use tui_textarea::{CursorMove, TextArea};

pub struct PopupDialog<'a> {
    pub textarea: TextArea<'a>,
    pub needs_redraw: bool,
}

impl PopupDialog<'_> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Target directory"));
        Self {
            textarea,
            needs_redraw: true,
        }
    }

    pub fn widget(&self) -> impl Widget + '_ {
        self.textarea.widget()
    }

    pub fn get_contents(&self) -> String {
        self.textarea.lines()[0].clone()
    }

    pub fn show(&mut self, suggested_value: &str, title: String) {
        let input_style = Style::default().fg(Color::Black).bg(Color::White);
        let border_style = Style::default().fg(Color::Yellow).bg(Color::Black);
        self.textarea = TextArea::from([suggested_value]);
        self.textarea.set_block(Block::default().borders(Borders::ALL).title(title).border_style(border_style));
        self.textarea.set_cursor_line_style(input_style);
        self.textarea.move_cursor(CursorMove::End);
        self.textarea.set_placeholder_text(suggested_value);
    }
}
