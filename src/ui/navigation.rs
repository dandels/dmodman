use super::component::traits::{Focus, Select};
use super::main_ui::MainUI;
use std::collections::HashMap;

impl MainUI<'_> {
    pub fn select_tab(&mut self, index: usize) {
        if index < self.nav.focused_widget_per_tab.len() {
            self.focused_widget_mut().remove_highlight();
            self.nav.select(Some(index));
            self.focused_widget_mut().add_highlight();
            self.top_bar.remove_urgency(index);
            self.top_bar.focus_tab(index);
            self.redraw_terminal = true;
        }
    }

    pub fn next_tab(&mut self) {
        if let Some(i) = self.nav.next() {
            self.select_tab(i);
        }
    }

    pub fn previous_tab(&mut self) {
        if let Some(i) = self.nav.previous() {
            self.select_tab(i);
        }
    }

    pub fn focused_widget(&self) -> &dyn Focus {
        match self.nav.focused_widget() {
            Focused::ArchiveTable => &self.archives_table,
            Focused::DownloadTable => &self.downloads_table,
            Focused::InstalledMods => &self.installed_mods_table,
            Focused::LogList => &self.log_view,
        }
    }

    pub fn focused_widget_mut(&mut self) -> &mut dyn Focus {
        match &self.nav.focused_widget() {
            Focused::ArchiveTable => &mut self.archives_table,
            Focused::DownloadTable => &mut self.downloads_table,
            Focused::InstalledMods => &mut self.installed_mods_table,
            Focused::LogList => &mut self.log_view,
        }
    }

    pub fn change_focus_to(&mut self, selected: Option<Focused>) {
        if let Some(selected) = selected {
            self.focused_widget_mut().remove_highlight();
            self.nav.set_focused_widget(selected);
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
    Archives,
    Installed,
    Log,
}

impl Tab {
    // Defines the order of the tabs
    const TAB_ARCHIVES: usize = 0;
    const TAB_INSTALLED: usize = 1;
    const TAB_LOG: usize = 2;

    pub fn index(&self) -> usize {
        match self {
            Tab::Archives => Self::TAB_ARCHIVES,
            Tab::Installed => Self::TAB_INSTALLED,
            Tab::Log => Self::TAB_LOG,
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct Nav {
    focused_tab: usize,
    focused_widget_per_tab: Vec<Focused>,
}

impl Nav {
    pub fn new() -> Self {
        Self {
            focused_tab: 0,
            // Default focused element for each tab
            focused_widget_per_tab: vec![Focused::ArchiveTable, Focused::InstalledMods, Focused::LogList],
        }
    }

    pub fn focused_tab(&self) -> Tab {
        self.focused_tab.into()
    }

    pub fn focused_widget(&self) -> &Focused {
        self.focused_widget_per_tab.get(self.focused_tab).unwrap()
    }

    pub fn set_focused_widget(&mut self, to_focus: Focused) {
        self.focused_widget_per_tab[self.focused_tab] = to_focus;
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

#[allow(dead_code)]
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

// These shouldn't be callable outside this file, but encapsulating it into this mod isn't enough
mod private_impl {
    use super::Nav;
    use crate::ui::component::traits::Select;

    impl Select for Nav {
        fn len(&self) -> usize {
            self.focused_widget_per_tab.len()
        }

        fn select(&mut self, index: Option<usize>) {
            if let Some(index) = index {
                if index < self.len() {
                    self.focused_tab = index;
                }
            }
        }

        fn selected(&self) -> Option<usize> {
            Some(self.focused_tab)
        }
    }
}

impl From<usize> for Tab {
    fn from(val: usize) -> Self {
        match val {
            Self::TAB_ARCHIVES => Tab::Archives,
            Self::TAB_INSTALLED => Tab::Installed,
            Self::TAB_LOG => Tab::Log,
            _ => unreachable!("Undefined tab index."),
        }
    }
}
