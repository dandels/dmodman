use crate::ui::{DownloadTable, FileTable, MessageList};
use tui::style::{Color, Modifier, Style};
use tui::widgets::Borders;

macro_rules! impl_highlight {
    ($T:ty, $title:expr) => {
        impl Highlight for $T {
            fn highlight_block(&mut self, block_style: Style) {
                self.block = self
                    .block
                    .clone()
                    .border_style(block_style)
                    .borders(Borders::ALL)
                    .title($title);
            }

            fn highlight_item(&mut self, highlight_style: Style) {
                self.highlight_style = highlight_style;
                self.refresh();
            }
        }
    };
}

impl_highlight!(DownloadTable<'_>, "Downloads");
impl_highlight!(FileTable<'_>, "Files");
impl_highlight!(MessageList<'_>, "Messages");

pub trait Highlight {
    fn highlight_item(&mut self, highlight_style: Style);

    fn highlight_block(&mut self, block_style: Style);

    fn focus(&mut self) {
        self.highlight_block(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        self.highlight_item(Style::default().fg(Color::Black).bg(Color::White));
    }

    fn unfocus(&mut self) {
        self.highlight_block(Style::reset());
        self.highlight_item(Style::reset());
    }
}
