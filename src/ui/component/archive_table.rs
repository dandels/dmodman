use super::common::*;
use crate::ui::navigation::*;
use crate::util;
use crate::Archives;
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;

pub struct ArchiveTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 2],
    archives: Archives,
    pub neighbors: NeighboringWidgets,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl ArchiveTable<'_> {
    pub async fn new(archives: Archives) -> Self {
        let block = DEFAULT_BLOCK.title(" Archives ");
        let headers = Row::new(vec![Cell::from(header_text("Filename")), Cell::from(header_text("Size"))]);
        let widths = [Constraint::Ratio(4, 5), Constraint::Ratio(1, 5)];

        archives.update_list().await;

        let mut neighbors = NeighboringWidgets::new();
        neighbors.map.insert(Tab::Archives, Neighbors::default().down(Focused::LogList));

        Self {
            headers,
            widths,
            archives,
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
        if self.archives.has_changed.swap(false, Ordering::Relaxed) {
            let mut rows: Vec<Row> = vec![];
            for (i, direntry) in self.archives.files.read().await.iter().enumerate() {
                rows.push(
                    Row::new(vec![
                        direntry.file_name().into_string().unwrap(),
                        util::format::human_readable(direntry.metadata().await.unwrap().len()).0,
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
