use crate::api::Downloads;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;
use tokio_stream::StreamExt;

pub struct DownloadTable<'a> {
    pub state: TableState,
    pub downloads: Downloads,
    pub block: Block<'a>,
    headers: Row<'a>,
    widths: [Constraint; 3],
    pub highlight_style: Style,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> DownloadTable<'a> {
    pub fn new(downloads: Downloads) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Downloads");

        let headers = Row::new(
            ["Filename", "Progress", "Status"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        downloads.has_changed.store(true, Ordering::Relaxed);
        let widths = [
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ];

        Self {
            state: TableState::default(),
            downloads,
            block,
            headers,
            widths,
            highlight_style: Style::default(),
            widget: Table::default(),
            len: 0,
        }
    }

    pub async fn refresh(&mut self) -> bool {
        if self.downloads.has_changed.swap(false, Ordering::Relaxed) {
            let tasks = self.downloads.tasks.read().await;
            let mut stream = tokio_stream::iter(tasks.values());
            let mut rows: Vec<Row> = vec![];
            while let Some(task) = stream.next().await {
                rows.push(Row::new(vec![
                    task.dl_info.file_info.file_name.to_owned(),
                    task.dl_info.progress.to_string(),
                    task.dl_info.get_state().to_string(),
                ]))
            }

            self.len = rows.len();
            self.widget = Table::new(rows, self.widths)
                .header(self.headers.to_owned())
                .block(self.block.to_owned())
                .highlight_style(self.highlight_style);
            return true
        }
        false
    }
}
