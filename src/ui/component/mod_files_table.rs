use super::common::*;
use crate::cache::Installed;
use crate::install::ModDirectory;
use crate::ui::navigation::*;
use indexmap::IndexMap;
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;

pub struct InstalledModsTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 3],
    pub currently_shown: IndexMap<String, ModDirectory>,
    pub installed: Installed,
    pub neighbors: NeighboringWidgets,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> InstalledModsTable<'a> {
    pub fn new(installed: Installed) -> Self {
        let block = DEFAULT_BLOCK.title(" Installed ").border_style(BLOCK_STYLE);
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
        neighbors
            .map
            .insert(Tab::Main, Neighbors::default().down(Focused::LogList).right(Focused::DownloadTable));

        Self {
            headers,
            widths,
            currently_shown: IndexMap::new(),
            installed,
            neighbors,
            block,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::default().widths(widths),
            len: 0,
        }
    }

    pub async fn refresh(&mut self) -> bool {
        if self.installed.has_changed.swap(false, Ordering::Relaxed) {
            let mut rows: Vec<Row> = vec![];
            let lock = self.installed.mods.read().await;
            self.currently_shown = lock.clone();
            for (i, (dir_name, dir_type)) in lock.iter().enumerate() {
                let row = match dir_type {
                    ModDirectory::Nexus(im) => Row::new(vec![
                        Cell::new(Span::raw(dir_name.clone())),
                        Cell::from(format_update_status_flags(&im.update_status)),
                        Cell::from(Text::from(im.version.as_ref().map(|v| v.to_string()).unwrap_or("".to_string()))),
                    ]),
                    _ => Row::new(vec![Span::raw((&dir_name).to_string())]),
                }
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

    pub fn get_by_index(&self, index: usize) -> (&String, &ModDirectory) {
        self.currently_shown.get_index(index).unwrap()
    }

    pub async fn delete_by_index(&self, index: usize) {
        let (dir_name, _) = self.get_by_index(index);
        self.installed.delete(dir_name).await;
    }
}
