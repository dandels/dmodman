use super::common::*;
use crate::ui::navigation::*;
use crate::util;
use crate::Cache;
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;

pub struct ArchiveTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 3],
    cache: Cache,
    pub neighbors: NeighboringWidgets,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl ArchiveTable<'_> {
    pub async fn new(cache: Cache) -> Self {
        let block = DEFAULT_BLOCK.title(" Archives ");
        let headers = Row::new(vec![
            Cell::from(header_text("Filename")),
            Cell::from(header_text("Status")),
            Cell::from(header_text("Size")),
        ]);
        let widths = [
            Constraint::Ratio(3, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ];

        let mut neighbors = NeighboringWidgets::new();
        neighbors.map.insert(Tab::Archives, Neighbors::default().down(Focused::LogList));

        Self {
            headers,
            widths,
            cache,
            neighbors,
            block,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::default().widths(widths),
            len: 0,
        }
    }

    // TODO use inotify to refresh the directory state when needed
    pub async fn refresh(&mut self) -> bool {
        if self.cache.archives.has_changed.swap(false, Ordering::Relaxed) {
            let mut rows: Vec<Row> = vec![];
            for (i, (archive_name, archive)) in self.cache.archives.files.read().await.iter().enumerate() {
                let mfd = self.cache.file_index.get_by_archive_name(&archive_name).await;
                let install_status = match mfd {
                    Some(mfd) => mfd.install_status.read().await.to_string(),
                    None => "".to_string(),
                };
                rows.push(
                    Row::new(vec![
                        archive_name.clone(),
                        install_status,
                        util::format::human_readable(archive.size).0,
                    ])
                    .style(LIST_STYLES[i % 2]),
                )
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
