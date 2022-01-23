use super::{DownloadTable, FileTable, Highlight, MessageList};

macro_rules! impl_stateful {
    ($T:ty, $collection:ident) => {
        impl Select for $T {
            fn len(&self) -> usize {
                self.$collection.len()
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

impl_stateful!(DownloadTable<'_>, downloads);
impl_stateful!(FileTable<'_>, files);
impl_stateful!(MessageList<'_>, msgs);

pub trait Select: Highlight {
    fn select(&mut self, index: Option<usize>);

    fn selected(&self) -> Option<usize>;

    fn len(&self) -> usize;

    fn deselect(&mut self) {
        self.select(None);
    }

    fn next(&mut self) {
        if self.len() == 0 {
            return;
        }
        let i = match self.selected() {
            Some(i) => {
                if i >= self.len() - 1 {
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
        if self.len() == 0 {
            return;
        }
        let i = match self.selected() {
            Some(i) => {
                if i == 0 {
                    self.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }
}
