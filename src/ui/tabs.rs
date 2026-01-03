use super::component::*;
use crate::events::EventSource;
use crate::ui::component::traits::{FocusableComponent, Select};
use crate::{
    api::{Client, Downloads},
    config::Config,
    db::Db,
    logger::Logger,
    ui::{
        component::{ArchivesWidget, DownloadsWidget},
        tab::Tab,
    },
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FocusedWidget {
    #[default]
    ArchiveTable,
    DownloadTable,
    InstalledMods,
    LogList,
}

#[derive(Copy, Clone, Default, Eq, Hash, PartialEq)]
#[repr(usize)]
// This defines the order of the tabs
pub enum TabType {
    #[default]
    Archives,
    Installed,
    Log,
}

pub struct WidgetContainer<'a> {
    pub tabs: TabWidgets<'a>,
    pub top_bar: RequestCounterWidget<'a>,
    pub hotkey_bar: HotkeyBar<'a>,
    pub bottom_bar: BottomBar<'a>,
    pub confirm_dialog: ConfirmDialog<'a>,
    pub popup_dialog: PopupDialog<'a>,
}

impl<'a> WidgetContainer<'a> {
    pub async fn create_tabs(
        config: Arc<Config>,
        client: Client,
        db: Db,
        downloads: Downloads,
        logger: Logger,
    ) -> Self {
        // let tabs = Tabs::new().await;
        // let bottom_bar = BottomBar::new(db, tabs.clone());
        // let confirm_dialog = ConfirmDialog::default();
        // let hotkey_bar = HotkeyBar::new();
        // let popup_dialog = PopupDialog::new(config.clone());
        // let top_bar = TopBar::new(client.request_counter).await;

        let tabs = TabWidgets::new(db.clone(), downloads.clone(), logger.clone()).await;
        let bottom_bar = BottomBar::new(db);
        let confirm_dialog = ConfirmDialog::default();
        let hotkey_bar = HotkeyBar::new();
        let popup_dialog = PopupDialog::init(config.clone());
        let top_bar = RequestCounterWidget::new(client.request_counter).await;

        Self {
            top_bar,
            hotkey_bar,
            bottom_bar,
            confirm_dialog,
            popup_dialog,
            tabs,
        }
    }

    pub fn focused_widget(&self) -> &dyn FocusableComponent {
        self.tabs.focused_widget()
    }

    pub fn focused_widget_mut(&mut self) -> &mut dyn FocusableComponent {
        self.tabs.focused_widget_mut()
    }
}

pub struct TabWidgets<'a> {
    pub tab_display: TabDisplay<'a>,
    pub archive_table: ArchivesWidget<'a>,
    pub downloads_table: DownloadsWidget<'a>,
    pub installed_mods_table: InstalledModsWidget<'a>,
    pub log_list: LogWidget<'a>,
    focused_index: usize,
    tabs: Vec<(Tab, bool)>, // bool tracks whether widget has changed since tab was last active
    len: usize,
}

impl<'a> TabWidgets<'a> {
    pub async fn new(db: Db, downloads: Downloads, logger: Logger) -> Self {
        let tabs: Vec<(Tab, bool)> = [
            Tab::new(&[FocusedWidget::ArchiveTable, FocusedWidget::DownloadTable]),
            Tab::new(&[FocusedWidget::InstalledMods]),
            Tab::new(&[FocusedWidget::LogList]),
        ]
        .into_iter()
        .map(|t| (t, false))
        .collect();

        let tab_display = TabDisplay::new();
        let installed_mods_table = InstalledModsWidget::new(db.installed.clone());
        let archives_table = ArchivesWidget::new(db);
        let downloads_table = DownloadsWidget::new(downloads);
        let log_list = LogWidget::new(logger);

        let mut ret = Self {
            tab_display,
            archive_table: archives_table,
            downloads_table,
            installed_mods_table,
            log_list,
            len: tabs.len(),
            tabs,
            focused_index: 0,
        };
        // Add highlight to initally focused widget
        ret.widget_for_type_mut(ret.focused_tab().focused_widget_type()).add_highlight();
        // ret.widget_for_type_mut(TABS[ret.focused_index]).add_highlight();
        ret
    }

    pub fn focused_tab(&self) -> &Tab {
        &self.tabs[self.focused_index].0
    }

    pub fn focused_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.focused_index].0
    }

    pub fn focused_widget(&self) -> &dyn FocusableComponent {
        self.widget_for_type(self.focused_tab().focused_widget_type())
    }

    pub fn focused_widget_mut(&mut self) -> &mut dyn FocusableComponent {
        self.widget_for_type_mut(self.focused_tab().focused_widget_type())
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
            FocusedWidget::ArchiveTable => &mut self.archive_table as &mut dyn FocusableComponent,
            FocusedWidget::DownloadTable => &mut self.downloads_table as &mut dyn FocusableComponent,
            FocusedWidget::InstalledMods => &mut self.installed_mods_table as &mut dyn FocusableComponent,
            FocusedWidget::LogList => &mut self.log_list as &mut dyn FocusableComponent,
        }
    }

    pub fn is_widget_visible(&self, t: FocusedWidget) {}

    pub fn try_focus_left_widget(&mut self) {}
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
