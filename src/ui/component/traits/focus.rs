use super::{Highlight, Select};
use crate::ui::component::*;
use crate::ui::navigation::*;
use std::collections::HashMap;

macro_rules! impl_focus {
    ($T:ty) => {
        impl Focus for $T {
            fn neighbor_map(&self) -> &HashMap<Tab, Neighbors> {
                &self.neighbors.map
            }
        }
    };
}

impl_focus!(ArchiveTable<'_>);
impl_focus!(DownloadsTable<'_>);
impl_focus!(InstalledModsTable<'_>);
impl_focus!(LogList<'_>);

pub trait Focus: Highlight + Select {
    fn neighbor_map(&self) -> &HashMap<Tab, Neighbors>;

    fn neighbor_up(&self, tab: &Tab) -> Option<Focused> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.up.clone())
    }
    fn neighbor_down(&self, tab: &Tab) -> Option<Focused> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.down.clone())
    }
    fn neighbor_left(&self, tab: &Tab) -> Option<Focused> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.left.clone())
    }
    fn neighbor_right(&self, tab: &Tab) -> Option<Focused> {
        self.neighbor_map().get(tab).and_then(|neighbors| neighbors.right.clone())
    }
}
