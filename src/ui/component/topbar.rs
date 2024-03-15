use crate::api::RequestCounter;
use crate::ui::navigation::Tabs as TabNavigation;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Tabs};
use std::sync::atomic::Ordering;

pub struct TopBar<'a> {
    pub tabs_widget: Tabs<'a>,
    request_counter: RequestCounter,
    pub counter_widget: Paragraph<'a>,
    pub highlight_style: Style,
    prev_selected_tab: usize,
    pub len: usize,
}

impl<'a> TopBar<'a> {
    pub async fn new(request_counter: RequestCounter) -> Self {
        let highlight_style = Style::new().bg(Color::White).fg(Color::Black);

        let tabnames = vec!["Main", "Archives"];
        let len = tabnames.len();
        let prev_selected_tab = 0;
        let tabs_widget = Tabs::new(tabnames).select(prev_selected_tab).highlight_style(highlight_style);
        let counter_widget = Self::create_widget(&request_counter).await;

        Self {
            tabs_widget,
            request_counter: request_counter.clone(),
            counter_widget,
            highlight_style,
            prev_selected_tab,
            len,
        }
    }

    pub async fn create_widget(request_counter: &RequestCounter) -> Paragraph<'a> {
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

    pub async fn refresh(&mut self, tabs: &TabNavigation) -> bool {
        let mut ret = false;
        if self.prev_selected_tab != tabs.active_index {
            self.prev_selected_tab = tabs.active_index;
            self.tabs_widget = self.tabs_widget.clone().select(tabs.active_index);
            ret = true;
        }
        if self.request_counter.has_changed.swap(false, Ordering::Relaxed) {
            self.counter_widget = Self::create_widget(&self.request_counter).await;
            ret = true;
        }
        ret
    }
}