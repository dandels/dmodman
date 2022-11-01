use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;

pub struct TopBar<'a> {
    pub widget: Paragraph<'a>,
    text: Vec<Spans<'a>>,
}

impl<'a> TopBar<'a> {
    pub fn new() -> Self {
        let text = vec![Spans::from(vec![
            Span::styled("<q>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("quit,"),
            Span::styled(" <u>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("update all"),
            Span::styled(" <U>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("update selected,"),
        ])];

        let widget = Paragraph::new(vec![]);
        Self { widget, text }
    }

    pub async fn refresh(&mut self) {
        self.widget = Paragraph::new(self.text.clone());
    }
}
