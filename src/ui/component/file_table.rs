use crate::api::UpdateChecker;
use crate::cache::{FileIndex, UpdateStatus};

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
        }
    }

    pub async fn refresh<'b>(&mut self)
    where
        'b: 'a,
    {
        let files = self.files.values_cloned().await;
        let mut stream = tokio_stream::iter(files);
        let mut rows: Vec<Row> = vec![];
        while let Some((local_file, file_details)) = stream.next().await {
            rows.push(Row::new(vec![
                file_details.name.clone(),
                match local_file.update_status {
                    Some(status) => match status {
                        UpdateStatus::OutOfDate => "U".to_string(),
                        UpdateStatus::UpToDate(_) | UpdateStatus::IgnoredUntil(_) => "K".to_string(),
                        UpdateStatus::HasNewFile(_) => "n".to_string(),
                    },
                    None => "?".to_string(),
                },
                file_details.version.as_ref().unwrap_or(&"".to_string()).to_string(),
            ]))
        }

        self.widget = Table::new(rows)
            .header(self.headers.to_owned())
            .block(self.block.to_owned())
            .widths(&[
                Constraint::Ratio(5, 7),
                Constraint::Ratio(1, 7),
                Constraint::Ratio(1, 7),
            ])
            .highlight_style(self.highlight_style.to_owned());
    }
}
