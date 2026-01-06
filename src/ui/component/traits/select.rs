use crate::ui::component::{
    ArchivesWidget, ConfirmDialog, DownloadsWidget, InstalledModsWidget, LogWidget, PopupDialog,
};

macro_rules! impl_select {
    ($T:ty) => {
        impl Select for $T {
            fn len(&self) -> usize {
                self.len
            }

            fn select(&mut self, index: Option<usize>) {
                if index < Some(self.len()) {
                    self.state.select(index)
                } else {
                    self.state.select(self.len().checked_sub(1))
                }
            }

            fn selected(&self) -> Option<usize> {
                self.state.selected()
            }
        }
    };
}

impl_select!(ArchivesWidget<'_>);
impl_select!(ConfirmDialog<'_>);
impl_select!(DownloadsWidget<'_>);
impl_select!(LogWidget<'_>);
impl_select!(InstalledModsWidget<'_>);
impl_select!(PopupDialog<'_>);

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
        let i = match self.selected() {
            Some(i) => {
                if i == 0 {
                    self.len().checked_sub(1)
                } else {
                    i.checked_sub(1)
                }
            }
            None => Some(0),
        };
        self.select(i);
    }
}
