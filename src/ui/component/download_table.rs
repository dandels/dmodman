use crate::api::Downloads;

use tui::layout::Constraint;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Cell, Row, Table, TableState};

pub struct DownloadTable<'a> {
    pub widget: Table<'a>,
    pub block: Block<'a>,
    pub downloads: Downloads,
    headers: Row<'a>,
    pub state: TableState,
    pub highlight_style: Style,
}

impl<'a> DownloadTable<'a> {
    pub fn new(downloads: Downloads) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Downloads");

        let headers = Row::new(
            vec!["Filename", "Progress"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        let highlight_style = Style::default();

        Self {
            widget: Self::create(block.clone(), headers.clone(), &downloads, highlight_style),
            block,
            downloads,
            headers,
            state: TableState::default(),
            highlight_style,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(
            self.block.clone(),
            self.headers.clone(),
            &self.downloads,
            self.highlight_style,
        );
    }

    fn create(block: Block<'a>, headers: Row<'a>, downloads: &Downloads, highlight_style: Style) -> Table<'a> {
        let rows: Vec<Row> = downloads
            .statuses
            .read()
            .unwrap()
            .iter()
            .map(|x| {
                let x = x.read().unwrap();
                Row::new(vec![x.file_name.clone(), x.progress()])
            })
            .collect();

        let table = Table::new(rows)
            .header(headers)
            .block(block)
            .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)])
            .highlight_style(highlight_style);

        table
    }
}
