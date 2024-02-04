use crate::ui::component::{DownloadTable, FileTable, MessageList, Select, TabBar};
use async_trait::async_trait;
use ratatui::style::{Color, Modifier, Style};
use std::sync::atomic::Ordering;

macro_rules! impl_highlight {
    ($T:ty) => {
        #[async_trait]
        impl Highlight for $T {
            fn focus(&mut self) {
                self.block = self.block.clone().border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                self.highlight_style = Style::default().fg(Color::Black).bg(Color::White);
                self.needs_redraw.store(true, Ordering::Relaxed);
            }

            fn unfocus(&mut self) {
                self.block =self.block.clone().border_style(Style::reset());
                self.highlight_style = Style::reset();
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
    fn focus(&mut self);
    fn unfocus(&mut self);
}