use super::common::*;
use super::traits::Select;
use crate::ui::navigation::*;
use crate::Logger;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, ListState};
use std::sync::atomic::Ordering;

pub struct LogList<'a> {
    list_items: Vec<ListItem<'a>>,
    logger: Logger,
    pub neighbors: NeighboringWidgets,
    pub block: Block<'a>,
    pub state: ListState,
    pub highlight_style: Style,
    pub widget: List<'a>,
    pub needs_redraw: bool,
    pub len: usize,
}

impl<'a> LogList<'a> {
    pub fn new(logger: Logger) -> Self {
        let block = DEFAULT_BLOCK.title(" Log ").border_style(BLOCK_STYLE);
        let highlight_style = Style::default();

        let mut neighbors = NeighboringWidgets::new();
        neighbors.map.insert(
            Tab::Main,
            Neighbors::default().left(Focused::FileTable).right(Focused::DownloadTable).up(Focused::FileTable),
        );
        neighbors.map.insert(Tab::Archives, Neighbors::default().up(Focused::ArchiveTable));

        Self {
            list_items: vec![],
            logger: logger.clone(),
            neighbors,
            block,
            state: ListState::default(),
            highlight_style,
            widget: List::default(),
            needs_redraw: true,
            len: 0,
        }
    }

    /* TODO there is an open issue for ratatui for word wrapping list items. Until then we can't properly show
     * long error messages: https://github.com/ratatui-org/ratatui/issues/128 */
    pub async fn refresh(&mut self) -> bool {
        if self.logger.has_changed.swap(false, Ordering::Relaxed) {
            let mut msgs_lock = self.logger.messages.write().unwrap();
            self.len += msgs_lock.len();
            self.list_items
                .append(&mut msgs_lock.drain(..).map(|msg| ListItem::new(Line::from(msg.to_owned()))).collect());

            if self.state.selected().is_none() && self.len > 0 || self.state.selected() == self.len.checked_sub(1) {
                self.state.select(Some(self.len));
            }

            self.widget = List::new(self.list_items.clone())
                .block(self.block.to_owned())
                .highlight_style(self.highlight_style.to_owned());

            return true;
        }
        false
    }

    pub fn remove(&mut self, i: usize) {
        crate::logger::log_to_file(format!("selected before {:?} len {}", self.selected(), self.len));
        self.list_items.remove(i);
        self.len = self.len.checked_sub(1).unwrap_or(0);
        self.widget = self.widget.clone().items(self.list_items.clone());
        self.next();
        crate::logger::log_to_file(format!("selected after {:?} len {}", self.selected(), self.len));
    }
}
