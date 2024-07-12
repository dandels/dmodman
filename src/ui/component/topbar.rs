use crate::api::RequestCounter;
use crate::ui::navigation::Nav as TabNavigation;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Tabs};
use std::sync::atomic::Ordering;

pub struct TopBar<'a> {
    pub tabs_widget: Tabs<'a>,
    request_counter: RequestCounter,
    pub counter_widget: Paragraph<'a>,
    prev_selected_tab: usize,
}

impl<'a> TopBar<'a> {
    pub async fn new(request_counter: RequestCounter) -> Self {
        let highlight_style = Style::new().bg(Color::White).fg(Color::Black);

        let tabnames = vec!["Archives", "Installed", "Log"];
        let prev_selected_tab = 0;
        let tabs_widget = Tabs::new(tabnames).select(prev_selected_tab).highlight_style(highlight_style);
        let counter_widget = Self::create_widget(&request_counter).await;

        Self {
            tabs_widget,
            request_counter: request_counter.clone(),
            counter_widget,
            prev_selected_tab,
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

    // TODO do this when changing tab instead of checking every draw tick?
    pub async fn refresh(&mut self, tabs: &TabNavigation) -> bool {
        let mut ret = false;
        if self.prev_selected_tab != tabs.active_tab {
            self.prev_selected_tab = tabs.active_tab;
            self.tabs_widget = self.tabs_widget.clone().select(tabs.active_tab);
            ret = true;
        }
        if self.request_counter.has_changed.swap(false, Ordering::Relaxed) {
            self.counter_widget = Self::create_widget(&self.request_counter).await;
            ret = true;
        }
        ret
    }
}
