use super::FocusedWidget;
use crate::ui::hotkeys::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct HotkeyBar<'a> {
    pub widget: Paragraph<'a>,
    focused: FocusedWidget,
    pub needs_redraw: AtomicBool,
}

impl<'a> HotkeyBar<'a> {
    pub fn new(focused: FocusedWidget) -> Self {
        let widget = Paragraph::new(Line::from(vec![]));
        Self {
            widget,
            focused,
            needs_redraw: AtomicBool::new(true),
        }
    }

    pub async fn refresh(&mut self, focused: &FocusedWidget) {
        if self.needs_redraw.swap(false, Ordering::Relaxed) || !self.focused.eq(focused) {
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

            self.widget = Paragraph::new(Line::from(text));
        }
    }
}
