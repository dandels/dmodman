use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

pub struct ListPicker<'a> {
    pub widget: List<'a>,
    pub block: Block<'a>,
    pub list: Vec<String>,
    pub state: ListState,
    pub highlight_style: Style,
}

impl<'a> ListPicker<'a> {
    pub fn new(title: String, list: Vec<String>) -> Self {
        let block = Block::default().borders(Borders::ALL).title(title);
        let highlight_style = Style::default();
        Self {
            widget: Self::create(block.clone(), &list, highlight_style),
            block,
            list,
            state: ListState::default(),
            highlight_style,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(self.block.clone(), &self.list, self.highlight_style);
    }

    fn create(block: Block<'a>, list: &Vec<String>, highlight_style: Style) -> List<'a> {
        let list_items: Vec<ListItem> = list
            .iter()
            .map(|i| {
                let lines = vec![Line::from(i.to_string())];
                ListItem::new(lines)
            })
            .collect();

        List::new(list_items).block(block).highlight_style(highlight_style)
    }
}
