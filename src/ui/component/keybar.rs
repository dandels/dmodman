use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct KeyBar<'a> {
    pub widget: Paragraph<'a>,
    text: Vec<Line<'a>>,
}

impl<'a> KeyBar<'a> {
    pub fn new() -> Self {
        let text = vec![Line::from(vec![
            Span::styled("<u>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("update all,"),
            Span::styled(" <U>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("update selected,"),
            Span::styled(" <i>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("ignore update,"),
            Span::styled(" <p>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("pause/resume,"),
            Span::styled(" <v>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("visit on Nexus,"),
            Span::styled(" <Del>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("delete,"),
            Span::styled(" <q>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("quit"),
        ])];

        let widget = Paragraph::new(Line::from(vec![]));
        Self { widget, text }
    }

    pub async fn refresh(&mut self) {
        self.widget = Paragraph::new(self.text.clone());
    }
}
