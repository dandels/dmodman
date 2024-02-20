use super::traits::{Highlight, Select};
use super::*;
use crate::ui::main_ui::MainUI;

#[derive(Clone, Debug, PartialEq)]
pub enum FocusedWidget {
    DownloadTable,
    FileTable,
    LogList,
    ArchiveTable,
}

pub trait FocusableWidget: Highlight + Select {}
impl FocusableWidget for ArchiveTable<'_> {}
impl FocusableWidget for DownloadTable<'_> {}
impl FocusableWidget for FileTable<'_> {}
impl FocusableWidget for LogList<'_> {}

impl MainUI<'_> {
    fn inner(&mut self, focused: &FocusedWidget) -> &mut dyn FocusableWidget {
        match focused {
            FocusedWidget::ArchiveTable => &mut self.archives_view,
            FocusedWidget::DownloadTable => &mut self.downloads_view,
            FocusedWidget::FileTable => &mut self.files_view,
            FocusedWidget::LogList => &mut self.log_view,
        }
    }

    pub fn focused_widget(&mut self) -> &mut dyn FocusableWidget {
        self.inner(&self.focused.clone())
    }

    pub fn change_focus_to(&mut self, selected: FocusedWidget) {
        self.focused_widget().unfocus();
        self.inner(&selected).focus();
        self.focused = selected;
        self.redraw_terminal = true;
    }

    pub fn select_next(&mut self) {
        self.focused_widget().next();
        self.redraw_terminal = true;
    }

    pub fn select_previous(&mut self) {
        self.focused_widget().previous();
        self.redraw_terminal = true;
    }

    pub fn selected_index(&mut self) -> Option<usize> {
        self.focused_widget().selected()
    }

    pub fn select_widget_index(&mut self, index: Option<usize>) {
        self.focused_widget().select(index);
        self.redraw_terminal = true;
    }
}
