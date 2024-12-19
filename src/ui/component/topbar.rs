use crate::api::RequestCounter;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Tabs};
use std::sync::atomic::Ordering;

pub struct TopBar<'a> {
    tab_titles: Vec<Line<'a>>,
    pub tabs_widget: Tabs<'a>,
    request_counter: RequestCounter,
    pub counter_widget: Paragraph<'a>,
    needs_refresh: bool,
}

impl<'a> TopBar<'a> {
    pub async fn new(request_counter: RequestCounter) -> Self {
        let highlight_style = Style::new().bg(Color::White).fg(Color::Black);

        let tab_titles: Vec<Line<'a>> = vec!["Archives", "Installed", "Log"].into_iter().map(Line::from).collect();
        let tabs_widget = Tabs::new(tab_titles.clone()).highlight_style(highlight_style);
        let counter_widget = Self::create_widget(&request_counter).await;

        Self {
            tab_titles,
            tabs_widget,
            request_counter: request_counter.clone(),
            counter_widget,
            needs_refresh: true,
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

    /* This is a bit fragile since urgency highlight conflicts with regular highlight and the state is tracked outside
     * this component */
    pub fn add_urgency(&mut self, index: usize) {
        let title = self.tab_titles.get_mut(index).unwrap();
        *title = title.clone().style(Style::new().fg(Color::LightYellow));
        self.tabs_widget = self.tabs_widget.clone().titles(self.tab_titles.clone());
        self.needs_refresh = true;
    }

    pub fn remove_urgency(&mut self, index: usize) {
        let title = self.tab_titles.get_mut(index).unwrap();
        *title = title.clone().style(Style::new());
        self.tabs_widget = self.tabs_widget.clone().titles(self.tab_titles.clone());
        self.needs_refresh = true;
    }

    pub fn focus_tab(&mut self, index: usize) {
        self.tabs_widget = self.tabs_widget.clone().select(index);
    }

    pub async fn refresh(&mut self) -> bool {
        let mut ret = self.needs_refresh;
        if self.request_counter.has_changed.swap(false, Ordering::Relaxed) {
            self.counter_widget = Self::create_widget(&self.request_counter).await;
            ret = true;
        }
        ret
    }
}
