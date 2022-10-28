use crate::cache::FileDetailsCache;

use tui::layout::Constraint;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Cell, Row, Table, TableState};

pub struct FileTable<'a> {
    pub widget: Table<'a>,
    pub block: Block<'a>,
    pub files: FileDetailsCache,
    headers: Row<'a>,
    pub state: TableState,
    pub highlight_style: Style,
}

impl<'a> FileTable<'a> {
    pub fn new(files:  FileDetailsCache) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Files");

        let highlight_style = Style::default();

        let headers = Row::new(
            vec!["Name", "Version"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        Self {
            widget: Self::create(block.clone(), headers.clone(), &files, highlight_style),
            block,
            files,
            headers,
            state: TableState::default(),
            highlight_style,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(
            self.block.clone(),
            self.headers.clone(),
            &self.files,
            self.highlight_style,
        );
    }

    // TODO handle missing FileDetails and foreign (non-Nexusmods) mods
    fn create(block: Block<'a>, headers: Row<'a>, files: &FileDetailsCache, highlight_style: Style) -> Table<'a> {
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

    pub fn is_changed(&self) -> bool {
        self.files.is_changed()
    }
}
