use crate::api::Downloads;
use tokio_stream::StreamExt;
use tui::layout::Constraint;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Cell, Row, Table, TableState};

pub struct DownloadTable<'a> {
    pub state: TableState,
    pub downloads: Downloads,
    pub block: Block<'a>,
    headers: Row<'a>,
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
            state: TableState::default(),
            downloads: downloads.clone(),
            block,
            headers,
            highlight_style,
        }
    }

    pub async fn create<'b>(&self) -> Table<'b>
    where
        'a: 'b, {
        let ds = self.downloads.statuses.read().await;
        let mut stream = tokio_stream::iter(ds.values());
        let mut rows: Vec<Row> = vec![];
        while let Some(val) = stream.next().await {
            rows.push(Row::new(vec![val.file_name.clone(), val.progress()]))
        }

        let table = Table::new(rows)
            .header(self.headers.to_owned())
            .block(self.block.to_owned())
            .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)])
            .highlight_style(self.highlight_style);

        table
    }
}
