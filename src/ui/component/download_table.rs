use super::common::*;
use crate::api::{Downloads, FileInfo};
use crate::ui::navigation::*;
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;

pub struct DownloadsTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 3],
    pub currently_shown: Vec<FileInfo>,
    pub downloads: Downloads,
    pub block: Block<'a>,
    pub state: TableState,
    pub neighbors: NeighboringWidgets,
    pub highlight_style: Style,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> DownloadsTable<'a> {
    pub fn new(downloads: Downloads) -> Self {
        let block = DEFAULT_BLOCK.title(" Downloads ").border_style(BLOCK_STYLE);

        let headers = Row::new(vec![
            Cell::from(header_text("Filename")),
            Cell::from(header_text("Progress")),
            Cell::from(header_text("Status")),
        ]);

        let widths = [
            Constraint::Percentage(65),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ];

        downloads.has_changed.store(true, Ordering::Relaxed);

        let mut neighbors = NeighboringWidgets::new();
        neighbors
            .map
            .insert(Tab::Main, Neighbors::default().left(Focused::InstalledMods).down(Focused::LogList));

        Self {
            headers,
            widths,
            currently_shown: Vec::new(),
            downloads,
            block,
            neighbors,
            state: TableState::default(),
            highlight_style: Style::default(),
            widget: Table::default(),
            len: 0,
        }
    }

    pub async fn refresh(&mut self) -> bool {
        if self.downloads.has_changed.swap(false, Ordering::Relaxed) {
            let mut rows: Vec<Row> = vec![];
            let lock = self.downloads.tasks.read().await;
            let mut shown = Vec::with_capacity(lock.len());
            for (i, task) in lock.values().enumerate() {
                shown.push(task.dl_info.file_info.clone());
                rows.push(
                    Row::new(vec![
                        task.dl_info.file_info.file_name.to_owned(),
                        task.dl_info.progress.to_string(),
                        task.dl_info.get_state().to_string(),
                    ])
                    .style(LIST_STYLES[i % 2]),
                )
            }
            self.currently_shown = shown;

            self.len = rows.len();
            self.widget = Table::new(rows, self.widths)
                .header(self.headers.to_owned())
                .block(self.block.to_owned())
                .highlight_style(self.highlight_style);
            return true;
        }
        false
    }

    pub fn get_by_index(&self, index: usize) -> &FileInfo {
        self.currently_shown.get(index).unwrap()
    }

    pub async fn delete_by_index(&mut self, index: usize) {
        let fi = self.get_by_index(index);
        self.downloads.delete(fi.file_id).await;
        self.len = self.len.saturating_sub(1);
    }
}
