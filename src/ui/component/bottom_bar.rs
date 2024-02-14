use crate::api::RequestCounter;
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct BottomBar<'a> {
    request_counter: RequestCounter,
    pub widget: Paragraph<'a>,
    pub needs_redraw: AtomicBool,
    redraw_terminal: Arc<AtomicBool>,
}

impl<'a> BottomBar<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>, request_counter: RequestCounter) -> Self {
        let widget = Paragraph::new("Remaining | hourly: NA | daily: NA").alignment(Alignment::Right);
        request_counter.has_changed.store(true, Ordering::Relaxed);
        Self {
            widget,
            request_counter: request_counter.clone(),
            needs_redraw: AtomicBool::new(true),
            redraw_terminal,
        }
    }

    pub async fn refresh(&mut self) {
        if self.request_counter.has_changed.swap(false, Ordering::Relaxed) {
            self.widget = Paragraph::new(self.request_counter.format().await).alignment(Alignment::Right);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}
