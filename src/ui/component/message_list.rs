use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use tokio_stream::StreamExt;

use crate::Messages;

pub struct MessageList<'a> {
    pub block: Block<'a>,
    pub msgs: Messages,
    pub state: ListState,
    pub highlight_style: Style,
    pub widget: List<'a>,
    pub needs_redraw: AtomicBool,
    has_data_changed: Arc<AtomicBool>,
    redraw_terminal: Arc<AtomicBool>,
    prev_len: usize,
}

impl<'a> MessageList<'a> {
    pub async fn new(redraw_terminal: Arc<AtomicBool>, msgs: Messages) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Messages");
        let highlight_style = Style::default();

        msgs.has_changed.store(true, Ordering::Relaxed);

        Self {
            block,
            msgs: msgs.clone(),
            state: ListState::default(),
            highlight_style,
            widget: List::default(),
            needs_redraw: AtomicBool::new(false),
            has_data_changed: msgs.has_changed,
            redraw_terminal,
            prev_len: 0,
        }
    }

    pub async fn refresh<'b>(&mut self)
    where
        'b: 'a,
    {
        if self.has_data_changed.swap(false, Ordering::Relaxed) {
            let mut items: Vec<ListItem<'b>> = vec![];
            let msgs = self.msgs.messages.read().await;

            let scroll_downwards =
                (self.state.selected() == Some(self.prev_len) || self.state.selected() == None) && msgs.len() != 0;

            let mut stream = tokio_stream::iter(msgs.iter());

            // TODO append new items to the list instead of constantly recreating it?
            /* TODO there is an open issue for ratatui for word wrapping list items. Until then we can't properly show
             * long error messages: https://github.com/ratatui-org/ratatui/issues/128 */
            while let Some(val) = stream.next().await {
                let lines = vec![Line::from(val.to_string())];
                items.push(ListItem::new(lines))
            }

            self.widget =
                List::new(items).block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned());

            if scroll_downwards {
                self.state.select(Some(msgs.len()));
            }
            self.prev_len = msgs.len();

            self.needs_redraw.store(false, Ordering::Relaxed);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        } else if self.needs_redraw.swap(false, Ordering::Relaxed) {
            self.widget =
                self.widget.clone().block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned());
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}
