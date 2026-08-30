use crate::LOGGER;

use super::common::*;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, ListState};

pub struct LogWidget<'a> {
    list_items: Vec<ListItem<'a>>,
    pub block: Block<'a>,
    pub state: ListState,
    pub highlight_style: Style,
    pub widget: List<'a>,
    pub len: usize,
}

impl<'a> LogWidget<'a> {
    pub async fn new() -> Self {
        let block = DEFAULT_BLOCK.title(" Log ");
        let widget = List::default().block(block.clone());
        let mut ret = Self {
            list_items: vec![],
            block,
            state: ListState::default(),
            highlight_style: Style::default(),
            widget,
            len: 0,
        };
        ret.refresh().await;
        ret
    }

    /* TODO there is an open issue for ratatui for word wrapping list items. Until then we can't properly show
     * long error messages: https://github.com/ratatui-org/ratatui/issues/128 */
    pub async fn refresh(&mut self) {
        let mut msgs_lock = LOGGER.messages.write().unwrap();
        self.list_items
            .append(&mut msgs_lock.drain(..).map(|msg| ListItem::new(Line::from(msg.to_owned()))).collect());
        let old_last_index = self.len.checked_sub(1);
        self.len = self.list_items.len();

        if self.state.selected().is_none() && self.len > 0 || self.state.selected() == old_last_index {
            self.state.select(self.len.checked_sub(1));
        }

        // TODO Ratatui's API forces needless copying. Upstream seems to be slowly working on this.
        self.widget = self.widget.to_owned().items(self.list_items.to_owned());
    }

    pub fn delete_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            self.list_items.remove(index);
            self.len = self.len.saturating_sub(1);
            self.widget = self.widget.clone().items(self.list_items.clone());
        }
    }
}
