use tui::widgets::ListState;
use tui::widgets::TableState;

// This is a lot of trouble just to not copypaste the implementations of ListState and TableState.

#[derive(Clone)]
pub enum Stateful {
    List(ListState),
    Table(TableState),
}

impl Stateful {
    pub fn select(&self, index: Option<usize>) {
        match self {
            Stateful::List(s) => s.select(index),
            Stateful::Table(s) => s.select(index),
        }
    }

    pub fn selected(&self) -> Option<usize> {
        match self {
            Stateful::List(s) => s.selected(),
            Stateful::Table(s) => s.selected(),
        }
    }

    pub fn as_table_state(&self) -> &TableState {
        if let Stateful::Table(s) = self {
            s
        } else {
            panic!("Attempted to get table state of non-table collection");
        }
    }

    pub fn as_list_state(&self) -> &ListState {
        if let Stateful::List(s) = self {
            s
        } else {
            panic!("Attempted to get list state of non-list collection");
        }
    }
}

#[derive(Clone)]
pub struct StatefulCollection<T> {
    pub state: Stateful,
    pub items: Vec<T>,
}

impl<T> StatefulCollection<T> {
    pub fn new(coll: Stateful) -> Self {
        StatefulCollection {
            state: coll,
            items: Vec::new(),
        }
    }

    pub fn new_list() -> Self {
        Self::new(Stateful::List(ListState::default()))
    }

    pub fn new_table() -> Self {
        Self::new(Stateful::Table(TableState::default()))
    }

    pub fn with_items(coll: Stateful, items: Vec<T>) -> Self {
        StatefulCollection { state: coll, items }
    }

    pub fn list_with_items(items: Vec<T>) -> Self {
        StatefulCollection {
            state: Stateful::List(ListState::default()),
            items,
        }
    }

    pub fn table_with_items(items: Vec<T>) -> Self {
        StatefulCollection {
            state: Stateful::Table(TableState::default()),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
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
