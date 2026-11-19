use crate::ui::InputMode;
use crate::ui::hotkeys::*;
use crate::ui::tabs::Focused;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct HotkeyBar<'a> {
    pub widget: Paragraph<'a>,
    focused: Focused,
    input_mode: InputMode,
}

impl<'a> HotkeyBar<'a> {
    pub fn new() -> Self {
        let mut ret = Self {
            widget: Paragraph::default(),
            focused: Focused::default(),
            input_mode: InputMode::default(),
        };
        ret.create_text();
        ret
    }

    pub fn change_focus(&mut self, focused: Focused) {
        self.focused = focused;
        self.create_text();
    }

    pub fn change_mode(&mut self, input_mode: InputMode) {
        self.input_mode = input_mode;
        self.create_text();
    }

    fn create_text(&mut self) {
        let keys = {
            match self.input_mode {
                InputMode::Normal => match self.focused {
                    Focused::ArchiveTable => ARCHIVES_KEYS,
                    Focused::InstalledMods => FILES_KEYS,
                    Focused::LogList => LOG_KEYS,
                    Focused::DownloadTable => DOWNLOADS_KEYS,
                },
                InputMode::Extract => INPUT_DIALOG_KEYS,
                _ => &[],
            }
        };

        let mut text = vec![];
        for (key, action) in keys {
            text.push(Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            text.push(Span::raw(*action));
        }

        self.widget = Paragraph::new(Line::from(text))
    }
}
