use super::common::*;
use crate::cache::ArchiveEntry;
use crate::ui::navigation::*;
use crate::util;
use crate::Cache;
use indexmap::IndexMap;
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;

pub struct ArchiveTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 4],
    pub cache: Cache,
    pub currently_shown: IndexMap<String, ArchiveEntry>,
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
            Cell::from(header_text("Flags").centered()),
            Cell::from(header_text("Size")),
        ]);
        let widths = [
            Constraint::Ratio(9, 12),
            Constraint::Ratio(1, 12),
            Constraint::Ratio(1, 12),
            Constraint::Ratio(1, 12),
        ];

        let mut neighbors = NeighboringWidgets::new();
        neighbors.map.insert(Tab::Archives, Neighbors::default().right(Focused::DownloadTable));

        Self {
            headers,
            widths,
            currently_shown: IndexMap::new(),
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
            let archives_lock = self.cache.archives.files.read().await;
            self.currently_shown = archives_lock.clone();
            for (i, (archive_name, entry)) in archives_lock.iter().enumerate() {
                let install_status = match &entry {
                    ArchiveEntry::File(archive) => archive.install_state.read().await.to_string(),
                    ArchiveEntry::MetadataOnly(_) => "Deleted".to_string(),
                };
                let update_status = match entry.metadata() {
                    Some(metadata) => {
                        let mfd = self.cache.metadata_index.get_by_file_id(&metadata.file_id).await.unwrap();
                        format_update_status_flags(&mfd.update_status)
                    }
                    None => "".into(),
                };
                rows.push(
                    Row::new(vec![
                        Cell::new(archive_name.clone()),
                        Cell::new(install_status),
                        Cell::new(update_status),
                        Cell::new(match entry {
                            ArchiveEntry::File(archive) => util::format::human_readable(archive.size).0,
                            _ => "".to_string(),
                        }),
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

    pub fn get_by_index(&self, index: usize) -> (&String, &ArchiveEntry) {
        self.currently_shown.get_index(index).unwrap()
    }

    pub async fn delete_by_index(&mut self, index: usize) {
        let (name, _) = self.get_by_index(index);
        self.cache.archives.delete(name).await;
        self.len = self.len.saturating_sub(1);
    }
}
