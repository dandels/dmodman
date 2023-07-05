use crate::api::Downloads;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio_stream::StreamExt;

pub struct DownloadTable<'a> {
    pub state: TableState,
    pub downloads: Downloads,
    pub block: Block<'a>,
    headers: Row<'a>,
    pub highlight_style: Style,
    pub widget: Table<'a>,
    pub needs_redraw: AtomicBool,
    redraw_terminal: Arc<AtomicBool>,
}

impl<'a> DownloadTable<'a> {
    pub fn new(redraw_terminal: Arc<AtomicBool>, downloads: Downloads) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Downloads");

        let headers = Row::new(
            vec!["Filename", "Progress"].iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        downloads.has_changed.store(true, Ordering::Relaxed);

        Self {
            state: TableState::default(),
            downloads,
            block,
            headers,
            highlight_style: Style::default(),
            widget: Table::new(vec![]),
            needs_redraw: AtomicBool::new(false),
            redraw_terminal,
        }
    }

    pub async fn refresh<'b>(&mut self)
    where
        'b: 'a,
    {
        if self.downloads.has_changed.swap(false, Ordering::Relaxed) {
            let tasks = self.downloads.tasks.read().await;
            let mut stream = tokio_stream::iter(tasks.values());
            let mut rows: Vec<Row> = vec![];
            while let Some(task) = stream.next().await {
                rows.push(Row::new(vec![task.file_name.clone(), task.progress.to_string()]))
            }

            self.widget = Table::new(rows)
                .header(self.headers.to_owned())
                .block(self.block.to_owned())
                .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)])
                .highlight_style(self.highlight_style);

            self.needs_redraw.store(false, Ordering::Relaxed);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        } else if self.needs_redraw.swap(false, Ordering::Relaxed) {
            self.widget = self.widget.clone().block(self.block.to_owned()).highlight_style(self.highlight_style);
            self.redraw_terminal.store(true, Ordering::Relaxed);
        }
    }
}
