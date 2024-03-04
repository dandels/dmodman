use crate::ui::component::{ArchiveTable, ConfirmDialog, DownloadTable, FileTable, LogList, PopupDialog};

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
impl_stateful!(ConfirmDialog<'_>);
impl_stateful!(DownloadTable<'_>);
impl_stateful!(FileTable<'_>);
impl_stateful!(LogList<'_>);
impl_stateful!(PopupDialog<'_>);

pub trait Select {
    fn len(&self) -> usize;

    fn select(&mut self, index: Option<usize>);

    fn selected(&self) -> Option<usize>;

    #[allow(dead_code)]
    fn deselect(&mut self) {
        self.select(None);
    }

    fn next(&mut self) {
        if self.len() == 0 {
            self.select(None);
            return;
        }
        let i = match self.selected() {
            Some(i) => {
                if i + 1 >= self.len() {
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
        self.select(match self.selected() {
            Some(i) => {
                if i == 0 {
                    self.len().checked_sub(1)
                } else {
                    i.checked_sub(1)
                }
            }
            None => Some(0),
        });
    }
}
