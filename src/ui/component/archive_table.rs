use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::Archives;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use tokio_stream::StreamExt;

use crate::util;

pub struct ArchiveTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 2],
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub needs_redraw: AtomicBool,
    redraw_terminal: Arc<AtomicBool>,
    pub len: usize,
}

impl<'a> ArchiveTable<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Archives");
        let headers = Row::new(["Name", "Size"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))));
        let widths = [Constraint::Ratio(4, 5), Constraint::Ratio(1, 5)];

        Self {
            block,
            headers,
            widths,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget: Table::default().widths(widths),
            needs_redraw: AtomicBool::new(true),
            redraw_terminal,
            len: 0,
        }
    }

    // TODO use inotify to refresh the directory state only when needed
    pub async fn refresh(&mut self, archives: &mut Archives) {
        if archives.swap_has_changed() {
            let arch_list = archives.list().await;
            let mut stream = tokio_stream::iter(arch_list.iter());
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
            self.needs_redraw.store(false, Ordering::Relaxed);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        } else if self.needs_redraw.swap(false, Ordering::Relaxed) {
            self.widget =
                self.widget.clone().block(self.block.to_owned()).highlight_style(self.highlight_style.to_owned());
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}