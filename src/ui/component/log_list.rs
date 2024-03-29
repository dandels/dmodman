use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::Logger;

pub struct LogList<'a> {
    pub block: Block<'a>,
    pub logger: Logger,
    pub state: ListState,
    pub highlight_style: Style,
    pub widget: List<'a>,
    pub needs_redraw: AtomicBool,
    list_items: Vec<ListItem<'a>>,
    redraw_terminal: Arc<AtomicBool>,
    pub len: usize,
}

impl<'a> LogList<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>, logger: Logger) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Log");
        let highlight_style = Style::default();

        // Render on first loop
        logger.has_changed.store(true, Ordering::Relaxed);

        Self {
            block,
            logger: logger.clone(),
            state: ListState::default(),
            highlight_style,
            widget: List::default(),
            needs_redraw: AtomicBool::new(false),
            list_items: vec![],
            redraw_terminal,
            len: 0,
        }
    }

    /* TODO there is an open issue for ratatui for word wrapping list items. Until then we can't properly show
     * long error messages: https://github.com/ratatui-org/ratatui/issues/128 */
    pub async fn refresh<'b>(&mut self)
    where
        'b: 'a,
    {
        if self.logger.has_changed.swap(false, Ordering::Relaxed) {
            let new_len;
            let mut items: Vec<ListItem<'a>> = {
                let msgs_lock = self.logger.messages.read().unwrap();
                new_len = msgs_lock.len();
                if new_len > 0 {
                    let msgs: &[String] = &msgs_lock[self.len..msgs_lock.len()];
                    msgs.iter().map(|msg| ListItem::new(Line::from(msg.to_owned()))).collect()
                } else {
                    vec![]
                }
            };
            self.list_items.append(&mut items);

            if self.state.selected() == None && new_len != 0 || self.state.selected() == self.len.checked_sub(1) {
                self.state.select(Some(new_len));
            }
            self.len = new_len;

            self.widget = List::new(self.list_items.clone())
                .block(self.block.to_owned())
                .highlight_style(self.highlight_style.to_owned());

            self.needs_redraw.store(false, Ordering::Relaxed);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        } else if self.needs_redraw.swap(false, Ordering::Relaxed) {
            self.widget =
                self.widget.clone().block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned());
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}
