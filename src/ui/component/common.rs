use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Borders, Padding};

pub const DEFAULT_BLOCK: Block = Block::new().borders(Borders::ALL).padding(Padding::horizontal(1));

pub const BLOCK_STYLE: Style = Style::new();
pub const BLOCK_HIGHLIGHT_STYLE: Style = Style::new().fg(Color::Red).add_modifier(Modifier::BOLD);

pub const HEADER_STYLE: Style = Style::new().add_modifier(Modifier::UNDERLINED);
pub const HIGHLIGHT_LINE_STYLE: Style = Style::new().fg(Color::Black).bg(Color::White);

pub const LISTITEM_STYLE: Style = Style::new();
pub const LISTITEM_ALTERNATE_STYLE: Style = Style::new().fg(Color::White);
pub const LIST_STYLES: &[Style] = &[LISTITEM_ALTERNATE_STYLE, LISTITEM_STYLE];

pub fn header_text(name: &str) -> Text<'_> {
    Text::from(Span::from(name).style(HEADER_STYLE))
}
