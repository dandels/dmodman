use crate::api::RequestCounter;
use ratatui::layout::Alignment;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct RequestCounterWidget<'a> {
    request_counter: RequestCounter,
    pub widget: Paragraph<'a>,
}

impl<'a> RequestCounterWidget<'a> {
    pub async fn new(request_counter: RequestCounter) -> Self {
        Self {
            request_counter: request_counter.clone(),
            widget: create_widget(&request_counter).await,
        }
    }

    pub async fn refresh(&mut self) {
        self.widget = create_widget(&self.request_counter).await;
    }
}

async fn create_widget<'a>(request_counter: &RequestCounter) -> Paragraph<'a> {
    let counter = request_counter.counter.read().await;
    let cells = vec![
        Span::from("API requests remaining: "),
        Span::from(format!(
            " Hourly: {} | Daily: {}",
            counter.hourly_remaining.map_or_else(|| "NA".to_string(), |i| i.to_string()),
            counter.daily_remaining.map_or_else(|| "NA".to_string(), |i| i.to_string())
        )),
    ];
    Paragraph::new(Line::from(cells)).alignment(Alignment::Right)
}
