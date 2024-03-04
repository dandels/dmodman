use super::common::*;
use crate::ui::navigation::*;
use crate::cache::{FileIndex, UpdateStatus};
use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;

pub struct FileTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 3],
    pub file_index: FileIndex,
    pub neighbors: NeighboringWidgets,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> FileTable<'a> {
    pub fn new(file_index: FileIndex) -> Self {
        let block = DEFAULT_BLOCK.title(" Files ").border_style(BLOCK_STYLE);
        let widths = [
            Constraint::Ratio(9, 12),
            Constraint::Ratio(1, 12),
            Constraint::Ratio(2, 12),
        ];

        let headers = Row::new(vec![
            Cell::from(header_text("Name")),
            Cell::from(header_text("Flags").centered()),
            Cell::from(header_text("Version")),
        ]);

        let mut neighbors = NeighboringWidgets::new();
        neighbors.map.insert(
            Tab::Main,
            Neighbors::default()
                .down(Focused::LogList)
                .right(Focused::DownloadTable)
        );

        Self {
            headers,
            widths,
            file_index: file_index.clone(),
            neighbors,
            block,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::default().widths(widths),
            len: 0,
        }
    }

    pub async fn refresh(&mut self) -> bool {
        if self.file_index.has_changed.swap(false, Ordering::Relaxed) {
            let mut rows: Vec<Row> = vec![];
            for (i, fdata) in self.file_index.files_sorted.read().await.iter().enumerate() {
                let fd = &fdata.file_details;
                let row = Row::new(vec![
                    Cell::from(fd.as_ref().map(|fd| fd.name.to_string()).unwrap_or(fdata.local_file.file_name.clone())),
                    //Cell::from(fdata.md5results.as_ref().and_then(|res| res.r#mod.name.clone()).unwrap_or("".to_string())),
                    Cell::from(Text::from(match fdata.local_file.update_status() {
                        UpdateStatus::OutOfDate(_) => Span::from("!").red(),
                        UpdateStatus::UpToDate(_) => Span::from(""),
                        UpdateStatus::IgnoredUntil(_) => Span::from(""),
                        UpdateStatus::HasNewFile(_) => Span::from("?").yellow(),
                    }).centered()),
                    Cell::from(
                        Text::from(fd.as_ref().and_then(|fd| fd.version.clone()).unwrap_or("".to_string())),
                    ),
                ])
                .style(LIST_STYLES[i % 2]);
                rows.push(row);
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
