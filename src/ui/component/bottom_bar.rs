use crate::api::RequestCounter;
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;
use std::sync::atomic::Ordering;

pub struct BottomBar<'a> {
    request_counter: RequestCounter,
    pub widget: Paragraph<'a>,
    pub needs_redraw: bool,
}

impl<'a> BottomBar<'a> {
    pub fn new(request_counter: RequestCounter) -> Self {
        let widget = Paragraph::new("Remaining | hourly: NA | daily: NA").alignment(Alignment::Right);
        Self {
            widget,
            request_counter: request_counter.clone(),
            needs_redraw: true,
        }
    }

    pub async fn refresh(&mut self) -> bool {
        if self.request_counter.has_changed.swap(false, Ordering::Relaxed) {
            self.widget = Paragraph::new(self.request_counter.format().await).alignment(Alignment::Right);
            return true;
        }
        false
    }
}
