use crate::ui::component::common::*;
use crate::ui::component::{ArchiveTable, DownloadsTable, InstalledModsTable, LogList};
use ratatui::style::Style;

macro_rules! impl_table_highlight {
    ($T:ty) => {
        impl Highlight for $T {
            fn add_highlight(&mut self) {
                self.highlight_style = HIGHLIGHT_LINE_STYLE;
                self.block = self.block.clone().border_style(BLOCK_HIGHLIGHT_STYLE);
                self.widget = self.widget.clone().row_highlight_style(self.highlight_style).block(self.block.clone());
            }

            fn remove_highlight(&mut self) {
                self.widget = self
                    .widget
                    .clone()
                    .row_highlight_style(Style::default())
                    .block(self.block.clone().border_style(BLOCK_STYLE));
            }
        }
    };
}

macro_rules! impl_highlight {
    ($T:ty) => {
        impl Highlight for $T {
            fn add_highlight(&mut self) {
                self.highlight_style = HIGHLIGHT_LINE_STYLE;
                self.block = self.block.clone().border_style(BLOCK_HIGHLIGHT_STYLE);
                self.widget = self.widget.clone().highlight_style(self.highlight_style).block(self.block.clone());
            }

            fn remove_highlight(&mut self) {
                self.widget = self
                    .widget
                    .clone()
                    .highlight_style(Style::default())
                    .block(self.block.clone().border_style(BLOCK_STYLE));
            }
        }
    };
}

impl_table_highlight!(ArchiveTable<'_>);
impl_table_highlight!(DownloadsTable<'_>);
impl_table_highlight!(InstalledModsTable<'_>);
impl_highlight!(LogList<'_>);

pub trait Highlight {
    fn add_highlight(&mut self);
    fn remove_highlight(&mut self);
}
