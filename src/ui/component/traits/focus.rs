use super::{Component, Highlight, Select};
use crate::ui::component::*;
use crate::ui::navigation::Neighbors;
use crate::ui::navigation::TabType;
use crate::ui::tabs::FocusedWidget;
use std::collections::HashMap;

// Work around traits not being able to require fields
macro_rules! impl_focus {
    ($T:ty, $S:ty) => {
        impl Focus for $T {
            fn neighbor_map(&self) -> &HashMap<TabType, Neighbors> {
                &self.neighbors.map
            }
        }
    };
}

impl_focus!(ArchivesWidget<'_>, TableState);
impl_focus!(DownloadsWidget<'_>, TableState);
impl_focus!(InstalledModsWidget<'_>, TableState);
impl_focus!(LogWidget<'_>, ListState);

// impl_focus!(ArchiveTable<'_>);
// impl_focus!(DownloadsTable<'_>);
// impl_focus!(InstalledModsTable<'_>);
// impl_focus!(LogList<'_>);

pub trait Focus: Highlight + Select + Component {
    fn neighbor_map(&self) -> &HashMap<TabType, Neighbors>;

    #[allow(dead_code)]
    fn neighbor_up(&self, tab: &TabType) -> Option<FocusedWidget> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.up.clone())
    }
    #[allow(dead_code)]
    fn neighbor_down(&self, tab: &TabType) -> Option<FocusedWidget> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.down.clone())
    }
    fn neighbor_left(&self, tab: &TabType) -> Option<FocusedWidget> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.left.clone())
    }
    fn neighbor_right(&self, tab: &TabType) -> Option<FocusedWidget> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.right.clone())
    }
}
