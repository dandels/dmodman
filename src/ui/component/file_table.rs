use crate::cache::{FileIndex, UpdateStatus};

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio_stream::StreamExt;
use tui::layout::Constraint;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Cell, Row, Table, TableState};

pub struct FileTable<'a> {
    pub files: FileIndex,
    headers: Row<'a>,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub needs_redraw: Arc<AtomicBool>,
}

impl<'a> FileTable<'a> {
    pub fn new(files: FileIndex) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Files");
        let headers = Row::new(
            vec!["Name", "Flags", "Version"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        Self {
            files,
            block,
            headers,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::new(vec![]),
            needs_redraw: files.has_changed.clone(),
        }
    }

    pub async fn refresh<'b>(&mut self)
    where
        'b: 'a,
    {
        let files = self.files.file_index.read().await;
        let mut stream = tokio_stream::iter(files.values());
        let mut rows: Vec<Row> = vec![];
        while let Some(fdata) = stream.next().await {
            let lf = fdata.local_file.read().await;
            let (name, version) = match (*fdata.file_details).clone() {
                Some(fd) => (fd.name, fd.version.map_or("".to_string(), |v| v.to_string())),
                None => (lf.file_name.clone(), "".to_string()),
            };
            rows.push(Row::new(vec![
                name,
                match &lf.update_status {
                    UpdateStatus::OutOfDate(_) => "!".to_string(),
                    UpdateStatus::UpToDate(_) => "".to_string(),
                    UpdateStatus::IgnoredUntil(_) => "".to_string(),
                    UpdateStatus::HasNewFile(_) => "?".to_string(),
                },
                version,
            ]))
        }

        self.widget = Table::new(rows)
            .header(self.headers.to_owned())
            .block(self.block.to_owned())
            .widths(&[
                Constraint::Ratio(7, 10),
                Constraint::Ratio(1, 10),
                Constraint::Ratio(2, 10),
            ])
            .highlight_style(self.highlight_style.to_owned());
    }
}
