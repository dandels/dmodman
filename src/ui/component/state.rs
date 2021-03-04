use super::Stateful;
use tui::widgets::ListState;
use tui::widgets::TableState;

pub struct State {
    pub state: Stateful,
}

impl State {
    pub fn new(state_type: Stateful) -> Self {
        State { state: state_type }
    }

    pub fn new_list() -> Self {
        Self::new(Stateful::List(ListState::default()))
    }

    pub fn new_table() -> Self {
        Self::new(Stateful::Table(TableState::default()))
    }

    pub fn next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
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
