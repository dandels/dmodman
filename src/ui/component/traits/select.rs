use std::sync::atomic::Ordering;

use crate::ui::component::{ArchiveTable, DownloadTable, FileTable, LogList, TabBar};

impl Select for TabBar<'_> {
    fn len(&self) -> usize {
        self.len
    }

    fn select(&mut self, index: Option<usize>) {
        let i = index.unwrap();
        self.widget = self.widget.clone().select(i);
        self.selected_tab = i;
        self.needs_redraw.store(true, Ordering::Relaxed);
    }

    fn selected(&self) -> Option<usize> {
        Some(self.selected_tab)
    }
}

macro_rules! impl_stateful {
    ($T:ty) => {
        impl Select for $T {
            fn len(&self) -> usize {
                self.len
            }

            fn select(&mut self, index: Option<usize>) {
                self.state.select(index)
            }

            fn selected(&self) -> Option<usize> {
                self.state.selected()
            }
        }
    };
}

impl_stateful!(ArchiveTable<'_>);
impl_stateful!(DownloadTable<'_>);
impl_stateful!(FileTable<'_>);
impl_stateful!(LogList<'_>);

pub trait Select {
    fn len(&self) -> usize;

    fn select(&mut self, index: Option<usize>);

    fn selected(&self) -> Option<usize>;

    fn deselect(&mut self) {
        self.select(None);
    }

    fn next(&mut self) {
        let len = self.len();
        if len == 0 {
            return;
        }
        let i = match self.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }

    fn previous(&mut self) {
        let len = self.len();
        if len == 0 {
            return;
        }
        let i = match self.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }
}
