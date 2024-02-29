use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;
use tokio_stream::StreamExt;

use crate::cache::{FileIndex, UpdateStatus};

pub struct FileTable<'a> {
    pub file_index: FileIndex,
    headers: Row<'a>,
    widths: [Constraint; 5],
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> FileTable<'a> {
    pub fn new(file_index: FileIndex) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Files");
        let headers = Row::new(
            ["Name", "Category", "Mod", "Flags", "Version"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );
        let widths = [
            Constraint::Ratio(6, 14),
            Constraint::Ratio(2, 14),
            Constraint::Ratio(3, 14),
            Constraint::Ratio(1, 14),
            Constraint::Ratio(2, 14),
        ];

        Self {
            file_index: file_index.clone(),
            block,
            headers,
            widths,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::default().widths(widths),
            len: 0,
        }
    }

    pub async fn refresh(&mut self) -> bool {
        if self.file_index.has_changed.swap(false, Ordering::Relaxed) {
            let files = self.file_index.files_sorted.read().await;
            let mut stream = tokio_stream::iter(files.iter());
            let mut rows: Vec<Row> = vec![];
            while let Some(fdata) = stream.next().await {
                let fd = &fdata.file_details;
                rows.push(Row::new(vec![
                    fd.as_ref().and_then(|fd| Some(fd.name.to_string())).unwrap_or(fdata.local_file.file_name.clone()),
                    fd.as_ref().and_then(|fd|
                                Some(match &fd.category_name {
                                    Some(cat) => cat.to_string(),
                                    None => fd.category_id.to_string(),
                                })
                    ).unwrap_or("".to_string()),
                    fdata.md5results.as_ref().and_then(|res| res.r#mod.name.clone()).unwrap_or("".to_string()),
                    //lf.mod_id.to_string(),
                    match fdata.local_file.update_status() {
                        UpdateStatus::OutOfDate(_) => "!".to_string(),
                        UpdateStatus::UpToDate(_) => "".to_string(),
                        UpdateStatus::IgnoredUntil(_) => "".to_string(),
                        UpdateStatus::HasNewFile(_) => "?".to_string(),
                    },
                    fd.as_ref().and_then(|fd| fd.version.clone()).map_or("".to_string(), |v| v),
                ]))
            }

            self.len = rows.len();

            self.widget = Table::new(rows, self.widths)
                .header(self.headers.to_owned())
                .block(self.block.to_owned())
                .highlight_style(self.highlight_style.to_owned());
            return true;
        }
        false
    }
}
