use super::Stateful;
use std::sync::{Arc, RwLock};
use tui::widgets::ListState;
use tui::widgets::TableState;

pub struct StatefulCollection<T> {
    pub state: Stateful,
    pub items: Arc<RwLock<Vec<T>>>,
}

impl<T> StatefulCollection<T> {
    pub fn new(coll: Stateful) -> Self {
        StatefulCollection {
            state: coll,
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn new_list() -> Self {
        Self::new(Stateful::List(ListState::default()))
    }

    pub fn new_table() -> Self {
        Self::new(Stateful::Table(TableState::default()))
    }

    pub fn with_items(coll: Stateful, items: Vec<T>) -> Self {
        StatefulCollection {
            state: coll,
            items: Arc::new(RwLock::new(items)),
        }
    }

    pub fn list_with_items(items: Vec<T>) -> Self {
        StatefulCollection {
            state: Stateful::List(ListState::default()),
            items: Arc::new(RwLock::new(items)),
        }
    }

    pub fn table_with_items(items: Vec<T>) -> Self {
        StatefulCollection {
            state: Stateful::Table(TableState::default()),
            items: Arc::new(RwLock::new(items)),
        }
    }

    pub fn next(&mut self) {
        if self.items.read().unwrap().is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.read().unwrap().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.read().unwrap().is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.read().unwrap().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn deselect(&mut self) {
        self.state.select(None);
    }
}
