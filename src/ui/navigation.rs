use super::component::traits::{Focus, Select};
use super::main_ui::MainUI;
use std::collections::HashMap;

impl MainUI<'_> {
    pub fn select_tab(&mut self, index: usize) {
        if index < self.nav.focused_per_tab.len() {
            self.focused_widget_mut().remove_highlight();
            self.nav.select(Some(index));
            self.focused_widget_mut().add_highlight();
            self.redraw_terminal = true;
        }
    }

    pub fn next_tab(&mut self) {
        self.focused_widget_mut().remove_highlight();
        self.nav.next();
        self.focused_widget_mut().add_highlight();
        self.redraw_terminal = true;
    }

    pub fn previous_tab(&mut self) {
        self.focused_widget_mut().remove_highlight();
        self.nav.previous();
        self.focused_widget_mut().add_highlight();
        self.redraw_terminal = true;
    }

    pub fn focused_widget(&self) -> &dyn Focus {
        match self.nav.focused() {
            Focused::ArchiveTable => &self.archives_table,
            Focused::DownloadTable => &self.downloads_table,
            Focused::InstalledMods => &self.installed_mods_table,
            Focused::LogList => &self.log_view,
        }
    }

    pub fn focused_widget_mut(&mut self) -> &mut dyn Focus {
        match &self.nav.focused() {
            Focused::ArchiveTable => &mut self.archives_table,
            Focused::DownloadTable => &mut self.downloads_table,
            Focused::InstalledMods => &mut self.installed_mods_table,
            Focused::LogList => &mut self.log_view,
        }
    }

    pub fn change_focus_to(&mut self, selected: Option<Focused>) {
        if let Some(selected) = selected {
            self.focused_widget_mut().remove_highlight();
            self.nav.focus(selected);
            self.focused_widget_mut().add_highlight();
            self.redraw_terminal = true;
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Focused {
    ArchiveTable,
    DownloadTable,
    InstalledMods,
    LogList,
}

#[derive(Eq, Hash, PartialEq)]
pub enum Tab {
    Installed,
    Archives,
    Log,
}

#[derive(Eq, Hash, PartialEq)]
pub struct Nav {
    pub active_tab: usize,
    pub focused_per_tab: Vec<Focused>,
}

impl Nav {
    pub fn new() -> Self {
        Self {
            active_tab: 0,
            // Default focused element for each tab
            focused_per_tab: vec![Focused::ArchiveTable, Focused::InstalledMods, Focused::LogList],
        }
    }

    pub fn active(&self) -> Tab {
        self.active_tab.into()
    }

    pub fn focused(&self) -> &Focused {
        self.focused_per_tab.get(self.active_tab).unwrap()
    }

    pub fn focus(&mut self, to_focus: Focused) {
        self.focused_per_tab[self.active_tab] = to_focus;
    }
}

pub struct NeighboringWidgets {
    pub map: HashMap<Tab, Neighbors>,
}

impl NeighboringWidgets {
    pub fn new() -> Self {
        Self {
            map: HashMap::from([Tab::Archives, Tab::Installed, Tab::Log].map(|tab| (tab, Neighbors::default()))),
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

// These shouldn't be callable outside this file
mod private_impl {
    use super::Nav;
    use crate::ui::component::traits::Select;

    impl Select for Nav {
        fn len(&self) -> usize {
            self.focused_per_tab.len()
        }

        fn select(&mut self, index: Option<usize>) {
            if let Some(index) = index {
                if index < self.len() {
                    self.active_tab = index;
                }
            }
        }

        fn selected(&self) -> Option<usize> {
            Some(self.active_tab)
        }
    }
}

// Defines the order of the tabs
pub const TAB_ARCHIVES: usize = 0;
pub const TAB_INSTALLED: usize = 1;
pub const TAB_LOG: usize = 2;

impl From<Tab> for usize {
    fn from(value: Tab) -> Self {
        match value {
            Tab::Archives => TAB_ARCHIVES,
            Tab::Installed => TAB_INSTALLED,
            Tab::Log => TAB_LOG,
        }
    }
}

impl From<usize> for Tab {
    fn from(val: usize) -> Self {
        match val {
            TAB_ARCHIVES => Tab::Archives,
            TAB_INSTALLED => Tab::Installed,
            TAB_LOG => Tab::Log,
            _ => panic!("Undefined tab index."),
        }
    }
}
