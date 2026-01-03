use crate::ui::hotkeys::*;
use crate::ui::tabs::FocusedWidget;
use crate::ui::InputMode;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct HotkeyBar<'a> {
    pub widget: Paragraph<'a>,
}

impl<'a> HotkeyBar<'a> {
    pub fn new() -> Self {
        let mut ret = Self {
            widget: Paragraph::default(),
        };
        ret.update_state(&InputMode::default(), &FocusedWidget::default());
        ret
    }

    pub fn update_state(&mut self, input_mode: &InputMode, focused: &FocusedWidget) {
        let keys = {
            match input_mode {
                InputMode::Normal => match focused {
                    FocusedWidget::ArchiveTable => ARCHIVES_KEYS,
                    FocusedWidget::InstalledMods => FILES_KEYS,
                    FocusedWidget::LogList => LOG_KEYS,
                    FocusedWidget::DownloadTable => DOWNLOADS_KEYS,
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
