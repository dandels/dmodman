use super::traits::Select;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::{List, ListState, Widget};
use termion::event::{Event, Key, MouseButton, MouseEvent};
use tui_textarea::{CursorMove, TextArea};

#[derive(Default)]
pub struct PopupDialog<'a> {
    pub textarea: TextArea<'a>,
    pub list: List<'a>,
    pub state: ListState,
    pub needs_redraw: bool,
    pub len: usize,
    suggested_values: Vec<String>,
}

impl PopupDialog<'_> {
    pub fn new(suggested_values: Vec<String>, title: String) -> Self {
        let txt_default = "".to_string();
        let txt = suggested_values.first().unwrap_or(&txt_default);
        let mut textarea = TextArea::from([txt]);
        textarea.set_placeholder_text(txt);
        Self::set_text(&mut textarea, txt);

        let border_style = Style::default().fg(Color::Yellow).bg(Color::Black);
        let cursor_line_style = Style::default().fg(Color::Black).bg(Color::White);
        textarea.set_block(Block::default().borders(Borders::ALL).title(title).border_style(border_style));
        textarea.set_cursor_line_style(cursor_line_style);

        let list = List::from_iter(suggested_values.clone())
            .style(border_style)
            .block(Block::default().borders(Borders::ALL).title("Suggested values"))
            .highlight_style(cursor_line_style);
        let len = list.len();

        Self {
            textarea,
            needs_redraw: true,
            list,
            state: ListState::default().with_selected(Some(0)),
            len,
            suggested_values,
        }
    }

    fn set_text(textarea: &mut TextArea, text: &String) {
        textarea.move_cursor(CursorMove::End);
        textarea.delete_line_by_head();
        textarea.insert_str(text);
        //textarea.move_cursor(CursorMove::End);
    }

    pub fn input(&mut self, event: Event) {
        match event {
            Event::Key(Key::Down) | Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.next();
                let selected = self.selected().unwrap();
                Self::set_text(&mut self.textarea, self.suggested_values.get(selected).unwrap());
            }
            Event::Key(Key::Up) | Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.previous();
                let selected = self.selected().unwrap();
                Self::set_text(&mut self.textarea, self.suggested_values.get(selected).unwrap());
            }
            Event::Key(key) => {
                match key {
                    // disable tab character
                    Key::Char('\t') => {}
                    _ => {
                        self.textarea.input(key);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn widget(&self) -> impl Widget + '_ {
        self.textarea.widget()
    }

    pub fn get_contents(&self) -> String {
        self.textarea.lines()[0].clone()
    }
}
