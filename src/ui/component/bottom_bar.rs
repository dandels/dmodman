use crate::api::RequestCounter;
use tui::layout::Alignment;
use tui::widgets::Paragraph;

pub struct BottomBar<'a> {
    request_counter: RequestCounter,
    pub widget: Paragraph<'a>,
}

impl<'a> BottomBar<'a> {
    pub fn new(request_counter: RequestCounter) -> Self {
        // Duplicating this text here isn't nice, but making this function async causes lifetime issues
        let widget = Paragraph::new("Remaining | hourly: NA | daily: NA").alignment(Alignment::Right);
        Self {
            widget,
            request_counter,
        }
    }

    pub async fn refresh(&mut self) {
        self.widget = Paragraph::new(self.request_counter.format().await).alignment(Alignment::Right);
    }
}
