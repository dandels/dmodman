use super::FocusedWidget;
use crate::ui::hotkeys::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct HotkeyBar<'a> {
    pub widget: Paragraph<'a>,
    focused: FocusedWidget,
}

impl<'a> HotkeyBar<'a> {
    pub fn new(focused: FocusedWidget) -> Self {
        let widget = create_widget(&focused);
        Self { widget, focused }
    }

    pub async fn refresh(&mut self, focused: &FocusedWidget) -> bool {
        if !self.focused.eq(focused) {
            self.widget = create_widget(focused);
            self.focused = focused.clone();
            return true;
        }
        false
    }
}
fn create_widget<'a>(focused: &FocusedWidget) -> Paragraph<'a> {
    let keys = match focused {
        FocusedWidget::ArchiveTable => ARCHIVES_KEYS,
        FocusedWidget::FileTable => FILES_KEYS,
        FocusedWidget::LogList => LOG_KEYS,
        FocusedWidget::DownloadTable => DOWNLOADS_KEYS,
    };

    let mut text = vec![];
    for (key, action) in keys {
        text.push(Span::styled(*key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        text.push(Span::raw(*action));
    }

    Paragraph::new(Line::from(text))
}
