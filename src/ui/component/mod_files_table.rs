use super::common::*;
use crate::api::UpdateStatus;
use crate::cache::Installed;
use crate::ui::navigation::*;
use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Cell, Row, Table, TableState};
use std::sync::atomic::Ordering;
use crate::install::ModDirectory;

pub struct ModFilesTable<'a> {
    headers: Row<'a>,
    widths: [Constraint; 3],
    pub installed: Installed,
    pub neighbors: NeighboringWidgets,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> ModFilesTable<'a> {
    pub fn new(installed: Installed) -> Self {
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
        neighbors
            .map
            .insert(Tab::Main, Neighbors::default().down(Focused::LogList).right(Focused::DownloadTable));

        Self {
            headers,
            widths,
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
            for (i, (dir_name, dir_type)) in lock.iter().enumerate() {
                let row =
                    match dir_type.as_ref() {
                        ModDirectory::Nexus(im) => {
                            Row::new(vec![
                                Cell::new(Span::raw(dir_name.clone())),
                                //Cell::from(fdata.md5results.as_ref().and_then(|res| res.r#mod.name.clone()).unwrap_or("".to_string())),
                                Cell::from(
                                    Text::from(
                                        match im.update_status.to_enum() {
                                            UpdateStatus::OutOfDate(_) => Span::from("!").red(),
                                            UpdateStatus::UpToDate(_) => Span::from(""),
                                            UpdateStatus::IgnoredUntil(_) => Span::from(""),
                                            UpdateStatus::HasNewFile(_) => Span::from("+").yellow(),
                                            UpdateStatus::Invalid(_) => Span::from("?").yellow(),
                                        }
                                    )
                                    .centered(),
                                ),
                                Cell::from(Text::from(im.version.as_ref().map(|v| v.to_string()).unwrap_or("".to_string()))),
                            ])
                        }
                        ModDirectory::Unknown => Row::new(vec![Span::raw(format!("unknown {}", &dir_name))]),
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
}
