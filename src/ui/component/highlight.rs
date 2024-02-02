use crate::ui::component::{DownloadTable, FileTable, MessageList, Select};
use async_trait::async_trait;
use ratatui::style::{Color, Modifier, Style};
use std::sync::atomic::Ordering;

macro_rules! impl_highlight {
    ($T:ty) => {
        #[async_trait]
        impl Highlight for $T {
            async fn highlight_block(&mut self, block_style: Style) {
                self.block = self.block.clone().border_style(block_style);
            }

            async fn highlight_item(&mut self, highlight_style: Style) {
                self.highlight_style = highlight_style;
                self.needs_redraw.store(true, Ordering::Relaxed);
            }
        }
    };
}

impl_highlight!(DownloadTable<'_>);
impl_highlight!(FileTable<'_>);
impl_highlight!(MessageList<'_>);

#[async_trait]
pub trait Highlight: Select + Send {
    async fn highlight_block(&mut self, block_style: Style);
    async fn highlight_item(&mut self, highlight_style: Style);

    async fn focus(&mut self) {
        self.highlight_block(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)).await;
        self.highlight_item(Style::default().fg(Color::Black).bg(Color::White)).await;
    }

    async fn unfocus(&mut self) {
        self.highlight_block(Style::reset()).await;
        self.highlight_item(Style::reset()).await;
    }
}
