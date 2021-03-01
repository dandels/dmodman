use tui::widgets::ListState;
use tui::widgets::TableState;

// This is a lot of trouble just to not copypaste the implementations of ListState and TableState.

#[derive(Clone)]
pub enum Stateful {
    List(ListState),
    Table(TableState),
}

impl Stateful {
    pub fn select(&mut self, index: Option<usize>) {
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

    pub fn as_table_state(&mut self) -> &mut TableState {
        if let Stateful::Table(s) = self {
            s
        } else {
            panic!("Attempted to get table state of non-table collection");
        }
    }

    pub fn as_list_state(&mut self) -> &mut ListState {
        if let Stateful::List(s) = self {
            s
        } else {
            panic!("Attempted to get list state of non-list collection");
        }
    }
}
