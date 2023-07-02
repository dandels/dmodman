use crate::Messages;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio_stream::StreamExt;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub struct MessageList<'a> {
    pub block: Block<'a>,
    pub msgs: Messages,
    pub state: ListState,
    pub highlight_style: Style,
    pub widget: List<'a>,
    pub needs_redraw: AtomicBool,
    has_data_changed: Arc<AtomicBool>,
    redraw_terminal: Arc<AtomicBool>,
}

impl<'a> MessageList<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>, msgs: Messages) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Messages");
        let highlight_style = Style::default();

        msgs.has_changed.store(true, Ordering::Relaxed);

        Self {
            block,
            msgs: msgs.clone(),
            state: ListState::default(),
            highlight_style,
            widget: List::new(vec![]),
            needs_redraw: AtomicBool::new(false),
            has_data_changed: msgs.has_changed,
            redraw_terminal,
        }
    }

    pub async fn refresh<'b>(&mut self)
    where
        'b: 'a,
    {

        if self.has_data_changed.swap(false, Ordering::Relaxed) {
            let mut items: Vec<ListItem<'b>> = vec![];
            let msgs = self.msgs.messages.read().await;
            let mut stream = tokio_stream::iter(msgs.iter());

            // TODO we could easily append new items to the list instead of constantly recreating it
            while let Some(val) = stream.next().await {
                let lines = vec![Line::from(val.to_string())];
                items.push(ListItem::new(lines))
            }

            self.widget =
                List::new(items).block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned());
            self.needs_redraw.store(false, Ordering::Relaxed);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        } else if self.needs_redraw.swap(false, Ordering::Relaxed) {
            self.widget =
                self.widget.clone().block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned());
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}
