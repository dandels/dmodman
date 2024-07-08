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
    pub len: usize,
}

impl<'a> LogList<'a> {
    pub fn new(logger: Logger) -> Self {
        let block = DEFAULT_BLOCK.title(" Log ");

        let mut neighbors = NeighboringWidgets::new();
        neighbors.map.insert(
            Tab::Main,
            // Define nonexistent neighbors to left and right so hotkeys work as expected
            Neighbors::default()
                .left(Focused::InstalledMods)
                .right(Focused::DownloadTable)
                .up(Focused::InstalledMods),
        );
        neighbors.map.insert(Tab::Archives, Neighbors::default().up(Focused::ArchiveTable));
        let widget = List::default().block(block.clone());

        Self {
            list_items: vec![],
            logger: logger.clone(),
            neighbors,
            block,
            state: ListState::default(),
            highlight_style: Style::default(),
            widget,
            len: 0,
        }
    }

    /* TODO there is an open issue for ratatui for word wrapping list items. Until then we can't properly show
     * long error messages: https://github.com/ratatui-org/ratatui/issues/128 */
    pub async fn refresh(&mut self) -> bool {
        if self.logger.has_changed.swap(false, Ordering::Relaxed) {
            let mut msgs_lock = self.logger.messages.write().unwrap();
            self.list_items
                .append(&mut msgs_lock.drain(..).map(|msg| ListItem::new(Line::from(msg.to_owned()))).collect());
            let old_last_index = self.len.checked_sub(1);
            self.len = self.list_items.len();

            if self.state.selected().is_none() && self.len > 0 || self.state.selected() == old_last_index {
                self.state.select(self.len.checked_sub(1));
            }

            self.widget = self.widget.clone().items(self.list_items.clone());
            return true;
        }
        false
    }

    pub fn delete_selected(&mut self) {
        if let Some(index) = self.selected() {
            self.list_items.remove(index);
            self.len = self.len.saturating_sub(1);
            self.widget = self.widget.clone().items(self.list_items.clone());
        }
    }
}
