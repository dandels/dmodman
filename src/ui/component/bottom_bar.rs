use crate::api::RequestCounter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;

pub struct BottomBar<'a> {
    request_counter: RequestCounter,
    pub widget: Paragraph<'a>,
    pub needs_redraw: AtomicBool,
    has_data_changed: Arc<AtomicBool>,
    redraw_terminal: Arc<AtomicBool>,
}

impl<'a> BottomBar<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>, request_counter: RequestCounter) -> Self {
        // Duplicating this text here isn't nice, but making this function async causes lifetime issues
        let widget = Paragraph::new("Remaining | hourly: NA | daily: NA").alignment(Alignment::Right);
        request_counter.has_changed.store(true, Ordering::Relaxed);
        Self {
            widget,
            request_counter: request_counter.clone(),
            needs_redraw: AtomicBool::new(true),
            has_data_changed: request_counter.has_changed,
            redraw_terminal,
        }
    }

    pub async fn refresh(&mut self) {
        if self.has_data_changed.swap(false, Ordering::Relaxed) {
            self.widget = Paragraph::new(self.request_counter.format().await).alignment(Alignment::Right);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}
