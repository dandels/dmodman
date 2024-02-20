use crate::ui::component::{ArchiveTable, DownloadTable, FileTable, LogList};
use ratatui::style::{Color, Modifier, Style};

macro_rules! impl_highlight {
    ($T:ty) => {
        impl Highlight for $T {
            fn focus(&mut self) {
                self.highlight_style = Style::default().fg(Color::Black).bg(Color::White);
                self.block =
                    self.block.clone().border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                self.widget = self.widget.clone().highlight_style(self.highlight_style).block(self.block.clone());
            }

            fn unfocus(&mut self) {
                self.widget = self
                    .widget
                    .clone()
                    .highlight_style(Style::reset())
                    .block(self.block.clone().border_style(Style::reset()));
            }
        }
    };
}

impl_highlight!(ArchiveTable<'_>);
impl_highlight!(DownloadTable<'_>);
impl_highlight!(FileTable<'_>);
impl_highlight!(LogList<'_>);

pub trait Highlight {
    fn focus(&mut self);
    fn unfocus(&mut self);
}
