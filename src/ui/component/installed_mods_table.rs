use super::common::*;
use crate::db::Installed;
use crate::extract::ModDirectory;
use indexmap::IndexMap;
use ratatui::layout::Constraint;
use ratatui::style::Style;
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Cell, Row, Table, TableState};

pub struct InstalledModsWidget<'a> {
    pub currently_shown: IndexMap<String, ModDirectory>,
    pub installed: Installed,
    pub block: Block<'a>,
    pub highlight_style: Style,
    pub state: TableState,
    pub widget: Table<'a>,
    pub len: usize,
}

impl<'a> InstalledModsWidget<'a> {
    pub async fn new(installed: Installed) -> Self {
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

        let widget = Table::new(Vec::<Row>::new(), widths)
            .header(headers)
            .block(block.to_owned())
            .row_highlight_style(Style::default());

        let mut ret = Self {
            currently_shown: IndexMap::new(),
            installed,
            block,
            highlight_style: Style::default(),
            state: TableState::default(),
            widget,
            len: 0,
        };
        ret.refresh().await;
        ret
    }

    pub async fn refresh(&mut self) {
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
        self.widget = self.widget.to_owned().rows(rows);
    }

    pub fn get_by_index(&self, index: usize) -> (&String, &ModDirectory) {
        self.currently_shown.get_index(index).unwrap()
    }

    pub async fn delete_by_index(&mut self, index: usize) {
        let (dir_name, _) = self.get_by_index(index);
        self.installed.delete(dir_name).await;
        self.len = self.len.saturating_sub(1);
    }
}
