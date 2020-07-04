use tui::widgets::TableState;

pub struct StatefulTable {
    pub state: TableState,
    pub items: Vec<Vec<String>>,
    pub headers: Vec<String>,
}

impl StatefulTable {
    pub fn new(headers: Vec<String>, data: Vec<Vec<String>>) -> StatefulTable {
        let mut state = TableState::default();
        if data.len() > 0 {
            state.select(Some(0));
        }
        let ret = StatefulTable {
            state: state,
            items: data,
            headers: headers,
        };
        return ret;
    }

    pub fn select_next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
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
}
