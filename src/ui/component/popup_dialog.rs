use super::common::*;
use super::traits::Select;
use crate::ui::rectangles::ExtractDialogRects;
use crate::Config;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, List, ListState, Paragraph};
use ratatui::Frame;
use ratatui_textarea::{CursorMove, TextArea};
use std::path::Path;
use termion::event::{Event, Key, MouseButton, MouseEvent};

pub struct ExtractDialog<'a> {
    config: Config,
    pub textbox: TextArea<'a>,
    pub description: Paragraph<'a>,
    pub prompt: Paragraph<'a>,
    pub suggestions: List<'a>,
    pub state: ListState,
    pub len: usize,
    pub suggested_values: Vec<String>,
    // layout_horizontal: Layout,
    // layout_vertical: Layout,
    // layout_label_and_input: Layout,
}

impl ExtractDialog<'_> {
    pub fn init(config: Config) -> Self {
        Self {
            config,
            description: Default::default(),
            state: ListState::default().with_selected(Some(0)),
            textbox: Default::default(),
            prompt: Default::default(),
            suggestions: Default::default(),
            len: Default::default(),
            suggested_values: Default::default(),
        }
    }

    pub fn create_widget(&mut self, suggested_values: Vec<String>, label: String) {
        let description = Paragraph::new(format!("Extracting to {:?}", self.config.install_dir()));

        let txt_default = "".to_string();
        let txt = suggested_values.first().unwrap_or(&txt_default);
        let mut textarea = TextArea::from([txt]);
        textarea.set_placeholder_text(txt);
        set_text(&mut textarea, txt);
        textarea.set_block(DEFAULT_BLOCK.title(" Ok "));

        let cursor_style = Style::default().fg(Color::Black).bg(Color::Gray);
        let cursor_line_style = Style::default();

        let text_label = Paragraph::new(format!("\n{}:", label)).style(Style::default().add_modifier(Modifier::BOLD));
        textarea.set_cursor_line_style(cursor_line_style);
        textarea.set_cursor_style(cursor_style);

        let list_style = Style::default().fg(Color::Gray).bg(Color::Black);
        let list_row_highlight_style = list_style.add_modifier(Modifier::REVERSED);

        let list = List::from_iter(suggested_values.clone())
            //.style(list_style)
            .block(Block::new().title("Suggested values:"))
            .highlight_style(list_row_highlight_style)
            .highlight_symbol(">> ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
        let len = list.len();

        *self = Self {
            config: self.config.clone(), // silly borrow checker
            description,
            textbox: textarea,
            prompt: text_label,
            suggestions: list,
            state: ListState::default().with_selected(Some(0)),
            len,
            suggested_values,
        };

        self.validate();
    }

    fn validate(&mut self) -> bool {
        let mut content = self.get_content();
        if content.is_empty() {
            // If textarea is empty the placeholder text is used instead
            content = self.textbox.placeholder_text();
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
                        self.textbox.set_block(DEFAULT_BLOCK.title(" Already exists ").style(textarea_warn_style));
                    } else {
                        self.textbox.set_block(DEFAULT_BLOCK.title(" Ok "));
                    }
                    true
                }
                Err(_e) => {
                    self.textbox.set_block(DEFAULT_BLOCK.title("  ").style(textarea_err_style));
                    false
                }
            }
        } else {
            self.textbox.set_block(DEFAULT_BLOCK.title(" Invalid directory name ").style(textarea_err_style));
            false
        }
    }

    pub fn input(&mut self, event: Event) {
        match event {
            Event::Key(Key::Down) | Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.next();
                let selected = self.selected().unwrap();
                set_text(&mut self.textbox, self.suggested_values.get(selected).unwrap());
            }
            Event::Key(Key::Up) | Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.previous();
                let selected = self.selected().unwrap();
                set_text(&mut self.textbox, self.suggested_values.get(selected).unwrap());
            }
            Event::Key(Key::Ctrl('z')) => {
                self.textbox.undo();
            }
            Event::Key(Key::Ctrl('y')) => {
                self.textbox.redo();
            }
            Event::Key(key) => {
                match key {
                    // disable tab character
                    Key::Char('\t') => {}
                    _ => {
                        self.textbox.input(key);
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

    pub fn get_content(&self) -> &str {
        &self.textbox.lines()[0]
    }

    pub fn render_widgets(&mut self, rects: &ExtractDialogRects, frame: &mut Frame) {
        frame.render_widget(&self.description, rects.description);
        frame.render_widget(&self.prompt, rects.prompt);
        frame.render_widget(&self.textbox, rects.textbox);
        frame.render_widget(&self.suggestions, rects.suggestions);
    }
}

fn set_text(textarea: &mut TextArea, text: &String) {
    textarea.move_cursor(CursorMove::End);
    textarea.delete_line_by_head();
    textarea.insert_str(text);
}
