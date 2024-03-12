use crate::ui::hotkeys::*;
use crate::ui::navigation::Focused;
use crate::ui::InputMode;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct HotkeyBar<'a> {
    pub widget: Paragraph<'a>,
    focused: Focused,
    input_mode: InputMode,
}

impl<'a> HotkeyBar<'a> {
    pub fn new(focused: Focused) -> Self {
        let input_mode = InputMode::Normal;
        let widget = create_widget(&input_mode, &focused);
        Self {
            widget,
            focused,
            input_mode,
        }
    }

    pub async fn refresh(&mut self, input_mode: &InputMode, focused: &Focused) -> bool {
        if !self.focused.eq(focused) || !self.input_mode.eq(input_mode) {
            self.widget = create_widget(input_mode, focused);
            self.focused = focused.clone();
            self.input_mode = input_mode.clone();
            return true;
        }
        false
    }
}
fn create_widget<'a>(input_mode: &InputMode, focused: &Focused) -> Paragraph<'a> {
    let keys = {
        match input_mode {
            InputMode::Normal => match focused {
                Focused::ArchiveTable => ARCHIVES_KEYS,
                Focused::InstalledMods => FILES_KEYS,
                Focused::LogList => LOG_KEYS,
                Focused::DownloadTable => DOWNLOADS_KEYS,
            },
            InputMode::ReadLine => INPUT_DIALOG_KEYS,
            _ => &[],
        }
    };

    let mut text = vec![];
    for (key, action) in keys {
        text.push(Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        text.push(Span::raw(*action));
    }

    Paragraph::new(Line::from(text))
}
