use super::common::*;
use crate::api::{Downloads, FileInfo};
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::widgets::{Block, Cell, Row, Table, TableState};

pub struct DownloadsWidget<'a> {
    headers: Row<'a>,
    widths: [Constraint; 3],
    pub last_known_state: Vec<FileInfo>,
    pub downloads: Downloads,
    pub block: Block<'a>,
    pub state: TableState,
    pub highlight_style: Style,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> DownloadsWidget<'a> {
    pub async fn new(downloads: Downloads) -> Self {
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

        let mut ret = Self {
            headers,
            widths,
            last_known_state: Vec::new(),
            downloads,
            block,
            state: TableState::default(),
            highlight_style: Style::default(),
            widget: Table::default(),
            len: 0,
        };
        ret.refresh().await;
        ret
    }

    pub async fn refresh(&mut self) {
        let mut rows: Vec<Row> = vec![];
        let lock = self.downloads.tasks.read().await;
        let mut current_values = Vec::with_capacity(lock.len());
        for (i, task) in lock.values().enumerate() {
            current_values.push(task.dl_info.file_info.clone());
            rows.push(
                Row::new(vec![
                    task.dl_info.file_info.file_name.to_owned(),
                    task.dl_info.progress.to_string(),
                    task.dl_info.get_state().to_string(),
                ])
                .style(LIST_STYLES[i % 2]),
            )
        }
        self.last_known_state = current_values;

        self.len = rows.len();
        self.widget = Table::new(rows, self.widths)
            .header(self.headers.to_owned())
            .block(self.block.to_owned())
            .row_highlight_style(self.highlight_style);
    }

    pub fn get_by_index(&self, index: usize) -> &FileInfo {
        self.last_known_state.get(index).unwrap()
    }

    pub async fn delete_by_index(&mut self, index: usize) {
        let fi = self.get_by_index(index);
        self.downloads.delete(fi.file_id).await;
        self.len = self.len.saturating_sub(1);
    }
}
