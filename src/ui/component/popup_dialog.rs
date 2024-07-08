use super::common::*;
use super::traits::Select;
use crate::Config;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, List, ListState, Paragraph};
use std::path::Path;
use std::sync::Arc;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use tui_textarea::{CursorMove, TextArea};

#[derive(Default)]
pub struct PopupDialog<'a> {
    config: Arc<Config>,
    pub textarea: TextArea<'a>,
    pub text_label: Paragraph<'a>,
    pub list: List<'a>,
    pub state: ListState,
    pub len: usize,
    suggested_values: Vec<String>,
}

impl PopupDialog<'_> {
    pub fn new(config: Arc<Config>, suggested_values: Vec<String>, label: String) -> Self {
        let txt_default = "".to_string();
        let txt = suggested_values.first().unwrap_or(&txt_default);
        let mut textarea = TextArea::from([txt]);
        textarea.set_placeholder_text(txt);
        Self::set_text(&mut textarea, txt);
        textarea.set_block(DEFAULT_BLOCK.title(" Ok "));

        let cursor_style = Style::default().fg(Color::Black).bg(Color::Gray);
        let cursor_line_style = Style::default();

        let text_label = Paragraph::new(format!("\n{}:", label)).style(Style::default().add_modifier(Modifier::BOLD));
        textarea.set_cursor_line_style(cursor_line_style);
        textarea.set_cursor_style(cursor_style);

        let list_style = Style::default().fg(Color::Gray).bg(Color::Black);
        let list_highlight_style = list_style.add_modifier(Modifier::REVERSED);

        let list = List::from_iter(suggested_values.clone())
            //.style(list_style)
            .block(Block::new().title("Suggested values:"))
            .highlight_style(list_highlight_style)
            .highlight_symbol(">> ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
        let len = list.len();

        let mut ret = Self {
            config,
            textarea,
            text_label,
            list,
            state: ListState::default().with_selected(Some(0)),
            len,
            suggested_values,
        };
        ret.validate();
        ret
    }

    fn validate(&mut self) -> bool {
        let mut content = self.get_content();
        if content.is_empty() {
            // If textarea is empty the placeholder text is used instead
            content = self.textarea.placeholder_text();
        }
        // Don't consider slashes at the end of the name in validation
        while content.ends_with('/') {
            content = &content[0..content.len() - 1];
        }
        let path = Path::new(content);
        let textarea_err_style = Style::default().fg(Color::Red);
        let textarea_warn_style = Style::default().fg(Color::Yellow);

        // Set highlight and warning message if input matches an existing file
        if Some(content).eq(&path.file_name().and_then(|s| s.to_str())) {
            match self.config.install_dir().join(path).try_exists() {
                Ok(exists) => {
                    if exists {
                        self.textarea.set_block(DEFAULT_BLOCK.title(" Already exists ").style(textarea_warn_style));
                    } else {
                        self.textarea.set_block(DEFAULT_BLOCK.title(" Ok "));
                    }
                    true
                }
                Err(_e) => {
                    self.textarea.set_block(DEFAULT_BLOCK.title("  ").style(textarea_err_style));
                    false
                }
            }
        } else {
            self.textarea.set_block(DEFAULT_BLOCK.title(" Invalid directory name ").style(textarea_err_style));
            false
        }
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
            Event::Key(Key::Ctrl('z')) => {
                self.textarea.undo();
            }
            Event::Key(Key::Ctrl('y')) => {
                self.textarea.redo();
            }
            Event::Key(key) => {
                match key {
                    // disable tab character
                    Key::Char('\t') => {}
                    _ => {
                        self.textarea.input(key);
                        self.select(None);
                    }
                }
            }
            _ => {}
        }
        self.validate();
    }

    pub fn get_required_height(&self) -> usize {
        self.len + 4
    }

    fn set_text(textarea: &mut TextArea, text: &String) {
        textarea.move_cursor(CursorMove::End);
        textarea.delete_line_by_head();
        textarea.insert_str(text);
    }

    pub fn get_content(&self) -> &str {
        &self.textarea.lines()[0]
    }
}
