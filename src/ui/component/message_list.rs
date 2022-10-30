use crate::Messages;

use tui::style::Style;
use tui::text::Spans;
use tui::widgets::{Block, Borders, List, ListItem, ListState};

pub struct MessageList<'a> {
    pub widget: List<'a>,
    pub block: Block<'a>,
    pub msgs: Messages,
    pub state: ListState,
    pub highlight_style: Style,
}

impl<'a> MessageList<'a> {
    pub fn new(msgs: Messages) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Messages");
        let highlight_style = Style::default();
        Self {
            widget: Self::create(block.clone(), &msgs, highlight_style),
            block,
            msgs,
            state: ListState::default(),
            highlight_style,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(self.block.clone(), &self.msgs, self.highlight_style);
    }

    fn create(block: Block<'a>, msgs: &Messages, highlight_style: Style) -> List<'a> {
        let list_items: Vec<ListItem> = msgs
            .messages
            .read()
            .unwrap()
            .iter()
            .map(|i| {
                let lines = vec![Spans::from(i.to_string())];
                ListItem::new(lines)
            })
            .collect();

        List::new(list_items).block(block).highlight_style(highlight_style)
    }
}
