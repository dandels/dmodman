use super::component::traits::{Focus, Select};
use super::main_ui::MainUI;
use std::collections::HashMap;

impl MainUI<'_> {
    pub fn select_tab(&mut self, index: usize) {
        if index < self.tabs.focused_per_tab.len() {
            self.focused_widget_mut().remove_highlight();
            self.tabs.select(Some(index));
            self.focused_widget_mut().add_highlight();
            self.redraw_terminal = true;
        }
    }

    pub fn next_tab(&mut self) {
        self.focused_widget_mut().remove_highlight();
        self.tabs.next();
        self.focused_widget_mut().add_highlight();
        self.redraw_terminal = true;
    }

    pub fn previous_tab(&mut self) {
        self.focused_widget_mut().remove_highlight();
        self.tabs.previous();
        self.focused_widget_mut().add_highlight();
        self.redraw_terminal = true;
    }

    pub fn focused_widget(&self) -> &dyn Focus {
        match self.tabs.focused() {
            Focused::ArchiveTable => &self.archives_view,
            Focused::DownloadTable => &self.downloads_view,
            Focused::FileTable => &self.files_view,
            Focused::LogList => &self.log_view,
        }
    }

    pub fn focused_widget_mut(&mut self) -> &mut dyn Focus {
        match &self.tabs.focused() {
            Focused::ArchiveTable => &mut self.archives_view,
            Focused::DownloadTable => &mut self.downloads_view,
            Focused::FileTable => &mut self.files_view,
            Focused::LogList => &mut self.log_view,
        }
    }

    pub fn change_focus_to(&mut self, selected: Option<Focused>) {
        if let Some(selected) = selected {
            self.focused_widget_mut().remove_highlight();
            self.tabs.focus(selected);
            self.focused_widget_mut().add_highlight();
            self.redraw_terminal = true;
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Focused {
    ArchiveTable,
    DownloadTable,
    FileTable,
    LogList,
}

#[derive(Eq, Hash, PartialEq)]
pub enum Tab {
    Main,
    Archives,
}

#[derive(Eq, Hash, PartialEq)]
pub struct Tabs {
    pub active_index: usize,
    pub focused_per_tab: Vec<Focused>,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            active_index: 0,
            focused_per_tab: vec![Focused::FileTable, Focused::ArchiveTable],
        }
    }

    pub fn active(&self) -> Tab {
        self.active_index.into()
    }

    pub fn focused(&self) -> &Focused {
        self.focused_per_tab.get(self.active_index).unwrap()
    }

    pub fn focus(&mut self, to_focus: Focused) {
        self.focused_per_tab[self.active_index] = to_focus;
    }
}

pub struct NeighboringWidgets {
    pub map: HashMap<Tab, Neighbors>,
}

impl NeighboringWidgets {
    pub fn new() -> Self {
        Self {
            map: HashMap::from([Tab::Main, Tab::Archives].map(|tab| (tab, Neighbors::default()))),
        }
    }
}

#[derive(Default, Eq, PartialEq)]
pub struct Neighbors {
    pub up: Option<Focused>,
    pub down: Option<Focused>,
    pub left: Option<Focused>,
    pub right: Option<Focused>,
}

impl Neighbors {
    pub fn up(mut self, neighbor: Focused) -> Self {
        self.up = Some(neighbor);
        self
    }

    pub fn down(mut self, neighbor: Focused) -> Self {
        self.down = Some(neighbor);
        self
    }

    pub fn left(mut self, neighbor: Focused) -> Self {
        self.left = Some(neighbor);
        self
    }

    pub fn right(mut self, neighbor: Focused) -> Self {
        self.right = Some(neighbor);
        self
    }
}

// TODO Select trait items shouldn't be exposed outside this module, but Tabs should
impl Select for Tabs {
    fn len(&self) -> usize {
        self.focused_per_tab.len()
    }

    fn select(&mut self, index: Option<usize>) {
        if let Some(index) = index {
            if index < self.len() {
                self.active_index = index;
            }
        }
    }

    fn selected(&self) -> Option<usize> {
        Some(self.active_index)
    }
}

impl From<Tab> for usize {
    fn from(value: Tab) -> Self {
        match value {
            Tab::Main => 0,
            Tab::Archives => 1,
        }
    }
}

impl From<usize> for Tab {
    fn from(val: usize) -> Self {
        match val {
            0 => Tab::Main,
            1 => Tab::Archives,
            _ => panic!("Undefined tab index."),
        }
    }
}
