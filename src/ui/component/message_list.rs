use crate::Messages;

use tokio_stream::StreamExt;
use tui::style::Style;
use tui::text::Spans;
use tui::widgets::{Block, Borders, List, ListItem, ListState};

pub struct MessageList<'a> {
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
            block,
            msgs,
            state: ListState::default(),
            highlight_style,
        }
    }

    // If the list gets long, it might be a good idea to create only the visible parts of the list
    pub async fn create<'b>(&self) -> List<'b>
    where
        'a: 'b, {
        let mut items: Vec<ListItem<'b>> = vec![];
        let msgs = self.msgs.messages.read().await;
        let mut stream = tokio_stream::iter(msgs.iter());

        while let Some(val) = stream.next().await {
            let lines = vec![Spans::from(val.to_string())];
            items.push(ListItem::new(lines))
        }

        List::new(items).block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned())
    }
}
