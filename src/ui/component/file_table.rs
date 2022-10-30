use crate::cache::FileIndex;

use tui::layout::Constraint;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Cell, Row, Table, TableState};

pub struct FileTable<'a> {
    headers: Row<'a>,
    pub block: Block<'a>,
    pub files: FileIndex,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
}

impl<'a> FileTable<'a> {
    pub fn new(files: &FileIndex) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Files");

        let highlight_style = Style::default();

        let headers =
            Row::new(vec!["Name", "Version"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))));

        let widget = Self::create(block.clone(), headers.clone(), &files, highlight_style);

        Self {
            block,
            files: files.clone(),
            headers,
            highlight_style,
            state: TableState::default(),
            widget,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(
            self.block.clone(),
            self.headers.clone(),
            &self.files,
            self.highlight_style,
        )
    }

    fn create(block: Block<'a>, headers: Row<'a>, files: &FileIndex, highlight_style: Style) -> Table<'a> {
        let rows: Vec<Row> = files
            .items()
            .iter()
            .map(|x| {
                Row::new(vec![
                    x.name.clone(),
                    x.version.as_ref().unwrap_or(&"".to_string()).to_string(),
                ])
            })
            .collect();

        let table = Table::new(rows)
            .header(headers)
            .block(block)
            .widths(&[Constraint::Percentage(85), Constraint::Percentage(15)])
            .highlight_style(highlight_style);

        table
    }
}
