use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::ui::component::traits::Select;
use ratatui::style::{Color, Style};
use ratatui::widgets::Tabs;

pub struct TabBar<'a> {
    pub widget: Tabs<'a>,
    pub highlight_style: Style,
    pub selected_tab: usize,
    pub needs_redraw: AtomicBool,
    redraw_terminal: Arc<AtomicBool>,
    pub len: usize,
}

impl<'a> TabBar<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>) -> Self {
        let highlight_style = Style::new().bg(Color::White).fg(Color::Black);

        let tabnames = vec!["Main", "Archives"];
        let len = tabnames.len();
        let selected_tab = 0;
        let widget = Tabs::new(tabnames).select(selected_tab).highlight_style(highlight_style);

        Self {
            widget,
            highlight_style,
            selected_tab,
            len,
            needs_redraw: AtomicBool::new(false),
            redraw_terminal,
        }
    }

    pub async fn refresh(&mut self) {
        if self.needs_redraw.swap(false, Ordering::Relaxed) {
            self.widget = self.widget.clone().select(self.selected_tab);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }

    pub fn next_tab(&mut self) {
        self.next();
        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    pub fn prev_tab(&mut self) {
        self.previous();
        self.needs_redraw.store(true, Ordering::Relaxed);
    }
}
