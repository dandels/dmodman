use crate::ui::tabs::FocusedWidget;

use super::component::traits::Select;
use super::main_ui::MainUI;
use std::collections::HashMap;

impl MainUI<'_> {
    pub fn select_tab(&mut self, index: usize) {
        if index < TABS_LEN {
            self.tabs.focused_widget_mut().remove_highlight();
            self.tabs.select(Some(index));
            self.tabs.focused_widget_mut().add_highlight();
            self.tabs.tab_display.remove_urgency(index);
            self.tabs.tab_display.focus_tab(index);
            self.render_active_widget();
        }
    }

    // pub fn next_tab(&mut self) {
    //     if let Some(i) = self.tabs.next() {
    //         self.select_tab(i);
    //     }
    // }

    // pub fn previous_tab(&mut self) {
    //     if let Some(i) = self.tabs.previous() {
    //         self.select_tab(i);
    //     }
    // }

    // pub fn focused_widget(&self) -> &Box<dyn Focus> {
    //     self.tabs.fo
    //     // match self.nav.focused_widget_type() {
    //     //     Focused::ArchiveTable => &self.archives_table,
    //     //     Focused::DownloadTable => &self.downloads_table,
    //     //     Focused::InstalledMods => &self.installed_mods_table,
    //     //     Focused::LogList => &self.log_list,
    //     // }
    // }

    // pub fn focused_widget_mut(&mut self) -> &mut Box<dyn Focus> {
    //     match &self.nav.focused_widget() {
    //         Focused::ArchiveTable => &mut self.archives_table,
    //         Focused::DownloadTable => &mut self.downloads_table,
    //         Focused::InstalledMods => &mut self.installed_mods_table,
    //         Focused::LogList => &mut self.log_list,
    //     }
    // }

    // pub fn focus_left(&mut self) {
    //     self.change_focus_to(self.tabs.focused_widget().neighbor_left(self.tabs.focused_tab()));
    // }

    // pub fn change_focus_to(&mut self, selected: Option<Focused>) {
    //     if let Some(selected) = selected {
    //         self.focused_widget_mut().remove_highlight();
    //         self.nav.set_focused_widget_type(selected);
    //         self.focused_widget_mut().add_highlight();
    //         self.render_active_widget();
    //     }
    // }
}

// #[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
// pub enum Focused {
//     #[default]
//     ArchiveTable,
//     DownloadTable,
//     InstalledMods,
//     LogList,
// }

pub const TABS_LEN: usize = 3;
impl TabType {
    pub fn index(self) -> usize {
        self as usize
    }
}

// #[derive(Eq, Hash, PartialEq)]
pub struct Nav {
    widget_layout: Vec<Vec<FocusedWidget>>,
    focused_tab: TabType,
    focused_widget_per_tab: Vec<FocusedWidget>,
}

impl Nav {
    pub fn new(widget_layout: Vec<Vec<FocusedWidget>>) -> Self {
        Self {
            widget_layout,
            focused_tab: TabType::default(),
            // Default focused element for each tab
            focused_widget_per_tab: vec![
                FocusedWidget::ArchiveTable,
                FocusedWidget::InstalledMods,
                FocusedWidget::LogList,
            ],
        }
    }

    pub fn focused_tab(&self) -> TabType {
        self.focused_tab
    }

    pub fn focused_tab_as_index(&self) -> usize {
        self.focused_tab as usize
    }

    //     pub fn focused_widget(&self) -> &Box<dyn Focus> {
    //         match self.focused_widget_per_tab[self.focused_tab as usize] {
    //             Focused::ArchiveTable => self.archive,
    //             Focused::DownloadTable => todo!(),
    //             Focused::InstalledMods => todo!(),
    //             Focused::LogList => todo!(),
    //         }
    //     }

    pub fn focused_widget_type(&self) -> FocusedWidget {
        self.focused_widget_per_tab[self.focused_tab as usize]
    }

    //     pub fn set_focused_widget_type(&mut self, to_focus: Focused) {
    //         self.focused_widget_per_tab[self.focused_tab] = to_focus;
    //     }
}

pub struct NeighboringWidgets {
    pub map: HashMap<TabType, Neighbors>,
}

impl NeighboringWidgets {
    pub fn new() -> Self {
        Self {
            map: HashMap::from(
                [TabType::Archives, TabType::Installed, TabType::Log].map(|tab| (tab, Neighbors::default())),
            ),
        }
    }
}

#[derive(Default, Eq, PartialEq)]
pub struct Neighbors {
    pub up: Option<FocusedWidget>,
    pub down: Option<FocusedWidget>,
    pub left: Option<FocusedWidget>,
    pub right: Option<FocusedWidget>,
}

#[allow(dead_code)]
impl Neighbors {
    pub fn up(mut self, neighbor: FocusedWidget) -> Self {
        self.up = Some(neighbor);
        self
    }

    pub fn down(mut self, neighbor: FocusedWidget) -> Self {
        self.down = Some(neighbor);
        self
    }

    pub fn left(mut self, neighbor: FocusedWidget) -> Self {
        self.left = Some(neighbor);
        self
    }

    pub fn right(mut self, neighbor: FocusedWidget) -> Self {
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
            Some(self.focused_tab as usize)
        }
    }
}

// impl From<usize> for Tab {
//     fn from(val: usize) -> Self {
//         match val {
//             Self::TAB_ARCHIVES => Tab::Archives,
//             Self::TAB_INSTALLED => Tab::Installed,
//             Self::TAB_LOG => Tab::Log,
//             _ => unreachable!("Undefined tab index."),
//         }
//     }
// }
