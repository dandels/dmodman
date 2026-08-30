use crate::ui::component::common::*;
use crate::ui::component::{ArchivesWidget, DownloadsWidget, InstalledModsWidget, LogWidget};
use ratatui::style::Style;

macro_rules! impl_table_highlight {
    ($T:ty) => {
        impl Highlight for $T {
            fn add_highlight(&mut self) {
                self.highlight_style = HIGHLIGHT_LINE_STYLE;
                // TODO not being able to modify the style of the existing block seems like a pointless API limitation
                self.block = self.block.to_owned().border_style(BLOCK_HIGHLIGHT_STYLE);
                self.widget =
                    self.widget.to_owned().row_highlight_style(self.highlight_style).block(self.block.to_owned());
            }

            fn remove_highlight(&mut self) {
                self.highlight_style = Style::default();
                self.block = self.block.to_owned().border_style(BLOCK_STYLE);
                self.widget =
                    self.widget.to_owned().row_highlight_style(self.highlight_style).block(self.block.to_owned());
            }
        }
    };
}

macro_rules! impl_highlight {
    ($T:ty) => {
        impl Highlight for $T {
            fn add_highlight(&mut self) {
                self.highlight_style = HIGHLIGHT_LINE_STYLE;
                self.block = self.block.to_owned().border_style(BLOCK_HIGHLIGHT_STYLE);
                self.widget = self.widget.to_owned().highlight_style(self.highlight_style).block(self.block.to_owned());
            }

            fn remove_highlight(&mut self) {
                self.highlight_style = Style::default();
                self.block = self.block.to_owned().border_style(BLOCK_STYLE);
                self.widget = self.widget.to_owned().highlight_style(self.highlight_style).block(self.block.to_owned());
            }
        }
    };
}

impl_table_highlight!(ArchivesWidget<'_>);
impl_table_highlight!(DownloadsWidget<'_>);
impl_table_highlight!(InstalledModsWidget<'_>);
impl_highlight!(LogWidget<'_>);

pub trait Highlight {
    fn add_highlight(&mut self);
    fn remove_highlight(&mut self);
}
