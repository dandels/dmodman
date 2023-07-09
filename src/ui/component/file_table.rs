use crate::cache::{FileIndex, UpdateStatus};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use tokio_stream::StreamExt;

pub struct FileTable<'a> {
    pub file_index: FileIndex,
    headers: Row<'a>,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub needs_redraw: AtomicBool,
    has_data_changed: Arc<AtomicBool>,
    redraw_terminal: Arc<AtomicBool>,
}

impl<'a> FileTable<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>, file_index: FileIndex) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Files");
        let headers = Row::new(
            vec!["Name", "Category", "Mod id", "Flags", "Version"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        let has_data_changed = file_index.has_changed.clone();
        has_data_changed.store(true, Ordering::Relaxed);

        Self {
            file_index: file_index.clone(),
            block,
            headers,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::new(vec![]),
            needs_redraw: AtomicBool::new(true),
            has_data_changed: file_index.has_changed,
            redraw_terminal,
        }
    }

    pub async fn refresh<'b>(&mut self)
    where
        'b: 'a,
    {
        if self.has_data_changed.swap(false, Ordering::Relaxed) {
            let files = self.file_index.files_sorted.read().await;
            let mut stream = tokio_stream::iter(files.iter());
            let mut rows: Vec<Row> = vec![];
            while let Some(fdata) = stream.next().await {
                let lf = &fdata.local_file.read().await;
                let fd = &fdata.file_details;
                rows.push(Row::new(vec![
                    fd.name.to_string(),
                    match &fd.category_name {
                        Some(cat) => cat.to_string(),
                        None => fd.category_id.to_string(),
                    },
                    lf.mod_id.to_string(),
                    match &lf.update_status {
                        UpdateStatus::OutOfDate(_) => "!".to_string(),
                        UpdateStatus::UpToDate(_) => "".to_string(),
                        UpdateStatus::IgnoredUntil(_) => "".to_string(),
                        UpdateStatus::HasNewFile(_) => "?".to_string(),
                    },
                    fd.version.clone().map_or("".to_string(), |v| v),
                ]))
            }

            self.widget = Table::new(rows)
                .header(self.headers.to_owned())
                .block(self.block.to_owned())
                .widths(&[
                    Constraint::Ratio(6, 12),
                    Constraint::Ratio(2, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(1, 12),
                    Constraint::Ratio(2, 12),
                ])
                .highlight_style(self.highlight_style.to_owned());
            self.needs_redraw.store(false, Ordering::Relaxed);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        } else if self.needs_redraw.swap(false, Ordering::Relaxed) {
            self.widget =
                self.widget.clone().block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned());
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}
