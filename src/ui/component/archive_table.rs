use crate::util;
use crate::Archives;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;
use tokio_stream::StreamExt;

pub struct ArchiveTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 2],
    archives: Archives,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl ArchiveTable<'_> {
    pub async fn new(archives: Archives) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Archives");
        let headers = Row::new(["Name", "Size"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))));
        let widths = [Constraint::Ratio(4, 5), Constraint::Ratio(1, 5)];

        archives.update_list().await;

        Self {
            block,
            headers,
            widths,
            archives,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::default().widths(widths),
            len: 0,
        }
    }

    // TODO use inotify to refresh the directory state when needed
    pub async fn refresh(&mut self) -> bool {
        if self.archives.has_changed.swap(false, Ordering::Relaxed) {
            let files = self.archives.files.read().await;
            let mut stream = tokio_stream::iter(files.iter());
            let mut rows: Vec<Row> = vec![];
            while let Some(direntry) = stream.next().await {
                rows.push(Row::new(vec![
                    direntry.file_name().into_string().unwrap(),
                    util::format::human_readable(direntry.metadata().await.unwrap().len()).0,
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
