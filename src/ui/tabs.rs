use super::component::*;
use crate::ui::component::traits::{FocusableComponent, Select};
use crate::{
    api::Downloads,
    db::Db,
    ui::{
        component::{ArchivesWidget, DownloadsWidget},
        tab::Tab,
    },
};

#[derive(Clone, Copy, Debug, Default)]
#[repr(usize)]
pub enum FocusedWidget {
    #[default]
    ArchiveTable,
    DownloadTable,
    InstalledMods,
    LogList,
}

// std::mem::variant_count() is not yet stabilized
const WIDGET_COUNT: usize = 4;

#[derive(Copy, Clone, Default)]
pub struct IndexMapping {
    pub tab: usize,
    pub widget: usize,
    // pub needs_update: bool,
}

// Hardcoded since this can seemingly not be done in a const context and not inlinining this is silly
pub const INDEX_MAPPING: [IndexMapping; WIDGET_COUNT] = {
    [
        IndexMapping { tab: 0, widget: 0 },
        IndexMapping { tab: 0, widget: 1 },
        IndexMapping { tab: 1, widget: 0 },
        IndexMapping { tab: 2, widget: 0 },
    ]
};

pub struct TabWidgets<'a> {
    pub tab_display: TabDisplay<'a>,
    pub archive_table: ArchivesWidget<'a>,
    pub downloads_table: DownloadsWidget<'a>,
    pub installed_mods_table: InstalledModsWidget<'a>,
    pub log_list: LogWidget<'a>,
    pub focused_index: usize,
    tabs: Vec<Tab>, // bool tracks whether widget has changed since tab was last active
    len: usize,
}

impl<'a> TabWidgets<'a> {
    pub async fn new(db: Db, downloads: Downloads) -> Self {
        const WIDGET_TYPES: [&[FocusedWidget]; 3] = [
            (&[FocusedWidget::ArchiveTable, FocusedWidget::DownloadTable]),
            (&[FocusedWidget::InstalledMods]),
            (&[FocusedWidget::LogList]),
        ];

        let tabs: Vec<Tab> = WIDGET_TYPES.into_iter().map(Tab::new).collect();
        let tab_display = TabDisplay::new();
        let installed_mods_table = InstalledModsWidget::new(db.installed.clone()).await;
        let archives_table = ArchivesWidget::new(db).await;
        let downloads_table = DownloadsWidget::new(downloads).await;
        let log_list = LogWidget::new().await;

        let mut ret = Self {
            tab_display,
            archive_table: archives_table,
            downloads_table,
            installed_mods_table,
            log_list,
            len: tabs.len(),
            tabs,
            focused_index: 0,
            // index_mapping,
        };
        // Add highlight to initally focused widget
        ret.widget_for_type_mut(ret.focused_tab().focused_widget_type()).add_highlight();
        // ret.widget_for_type_mut(TABS[ret.focused_index]).add_highlight();
        ret
    }

    pub fn focused_tab(&self) -> &Tab {
        &self.tabs[self.focused_index]
    }

    pub fn focused_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.focused_index]
    }

    pub fn focused_widget(&self) -> &dyn FocusableComponent {
        self.widget_for_type(self.focused_tab().focused_widget_type())
    }

    pub fn focused_widget_mut(&mut self) -> &mut dyn FocusableComponent {
        self.widget_for_type_mut(self.focused_tab().focused_widget_type())
    }

    pub fn focused_widget_mut_and_index(&mut self) -> (&mut dyn FocusableComponent, usize) {
        let tab = self.focused_tab();
        let i = tab.focused_widget_index;
        (self.widget_for_type_mut(tab.widget_types[i]), i)
    }

    pub fn focused_widget_type(&self) -> FocusedWidget {
        self.focused_tab().focused_widget_type()
    }

    pub fn widget_for_type(&self, t: FocusedWidget) -> &dyn FocusableComponent {
        match t {
            FocusedWidget::ArchiveTable => &self.archive_table as &dyn FocusableComponent,
            FocusedWidget::DownloadTable => &self.downloads_table as &dyn FocusableComponent,
            FocusedWidget::InstalledMods => &self.installed_mods_table as &dyn FocusableComponent,
            FocusedWidget::LogList => &self.log_list as &dyn FocusableComponent,
        }
    }

    pub fn widget_for_type_mut(&mut self, t: FocusedWidget) -> &mut dyn FocusableComponent {
        match t {
            FocusedWidget::ArchiveTable => &mut self.archive_table,
            FocusedWidget::DownloadTable => &mut self.downloads_table,
            FocusedWidget::InstalledMods => &mut self.installed_mods_table,
            FocusedWidget::LogList => &mut self.log_list,
        }
    }
}

impl<'a> Select for TabWidgets<'a> {
    fn len(&self) -> usize {
        self.len
    }

    fn select(&mut self, index: Option<usize>) {
        if let Some(index) = index {
            if index < self.len() {
                self.focused_widget_mut().remove_highlight();
                self.focused_index = index;
                self.focused_widget_mut().add_highlight();
                self.tab_display.remove_urgency(index);
                self.tab_display.focus_tab(index);
            }
        }
    }

    fn selected(&self) -> Option<usize> {
        Some(self.focused_index)
    }
}

impl std::ops::Index<FocusedWidget> for [IndexMapping; WIDGET_COUNT] {
    type Output = IndexMapping;

    fn index(&self, index: FocusedWidget) -> &Self::Output {
        &self[index as usize]
    }
}
