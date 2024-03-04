use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListState};

#[derive(Default)]
pub struct ConfirmDialog<'a> {
    pub widget: List<'a>,
    pub state: ListState,
    pub len: usize,
}

impl<'a> ConfirmDialog<'a> {
    pub fn new(title: String) -> Self {
        let border_style = Style::default().fg(Color::Gray).bg(Color::Black);
        let block = Block::default().borders(Borders::ALL).title(title).border_style(border_style);

        let widget = List::new([Span::raw("Ok"), Span::raw("Cancel")])
            .block(block)
            .style(Style::default())
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Gray))
            .highlight_symbol(">> ");

        Self {
            state: ListState::default().with_selected(Some(0)),
            widget,
            len: 2,
        }
    }
}
