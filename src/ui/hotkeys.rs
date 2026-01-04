use super::component::traits::Select;
use super::component::ConfirmDialog;
use super::main_ui::*;
use super::tabs::FocusedWidget;
use crate::db::ArchiveEntry;
use crate::extract::{InstallError, ModDirectory};
use crate::LOGGER;
use std::process::Command;
use termion::event::{Event, Key, MouseButton, MouseEvent};

pub const ARCHIVES_KEYS: &[(&str, &str)] = &[
    ("<Return>", "install "),
    ("<u>", "update all "),
    ("<U>", "update selected "),
    ("<i>", "ignore update "),
    ("<v>", "visit on Nexus "),
    ("<Del>", "delete "),
    ("<q>", "quit "),
];
pub const DOWNLOADS_KEYS: &[(&str, &str)] = &[("<p>", "pause/resume "), ("<Del>", "delete "), ("<q>", "quit ")];
pub const FILES_KEYS: &[(&str, &str)] = &[
    ("<u>", "update all "),
    ("<U>", "update selected "),
    ("<i>", "ignore update "),
    ("<v>", "visit on Nexus "),
    ("<Del>", "delete "),
    ("<q>", "quit "),
];
pub const LOG_KEYS: &[(&str, &str)] = &[("<Del>", "delete "), ("<q>", "quit ")];
pub const INPUT_DIALOG_KEYS: &[(&str, &str)] = &[
    ("<Return>", "confirm "),
    ("<Esc|C-c>", "cancel "),
    ("<Up|Down>", "cycle suggestions "),
    ("<C-z>", "undo "),
    ("<C-y>", "redo "),
];

impl MainUI<'_> {
    pub async fn handle_input(&mut self, event: Event) {
        //MouseEvent::Press(mouse_event, x, y) => {
        //LOGGER.log(format!("click! {mouse_event:?}, x: {x}, y: {y}"));
        //Event::Unsupported(u) => {
        //LOGGER.log(format!("Unsupported: {u:?}"));
        if let InputMode::Confirm = self.input_mode {
            self.handle_confirm_dialog(event).await;
            return;
        }
        if let InputMode::Extract = self.input_mode {
            self.handle_popup_dialog(event).await;
            return;
        }
        if let Event::Key(Key::Ctrl('c')) = event {
            self.should_run = false;
        }
        if let Event::Key(Key::Char('q')) = event {
            if self.installer.extract_jobs.read().await.is_empty() {
                self.should_run = false;
            } else {
                LOGGER.log("Refusing to quit, archive extraction is still in progress.");
                LOGGER.log("Press 'Ctrl + C' to force quit.");
            }
            return;
        }

        match event {
            Event::Key(Key::Down)
            | Event::Key(Key::Char('j'))
            | Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.tabs.focused_widget_mut().next();
            }
            Event::Key(Key::Up)
            | Event::Key(Key::Char('k'))
            | Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.tabs.focused_widget_mut().previous();
            }
            Event::Key(Key::Char('H')) => {
                self.tabs.focused_tab_mut().previous();
            }
            Event::Key(Key::Char('J')) => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    if let FocusedWidget::InstalledMods = self.tabs.focused_widget_type() {
                        self.db.installed.move_to_index(i, i.saturating_add(1)).await;
                        self.tabs.focused_widget_mut().next();
                    }
                }
            }
            Event::Key(Key::Char('K')) => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    if let FocusedWidget::InstalledMods = self.tabs.focused_widget_type() {
                        if i == 0 {
                            self.db
                                .installed
                                .move_to_index(i, self.tabs.focused_widget().len().saturating_sub(1))
                                .await;
                        } else {
                            self.db.installed.move_to_index(i, i.saturating_sub(1)).await;
                        }
                        self.tabs.focused_widget_mut().previous();
                    }
                }
            }
            // Event::Key(Key::Char('L')) => {
            //     self.change_focus_to(self.tabs.focused_widget().neighbor_right(&self.tabs.focused_tab_type()));
            // }
            Event::Key(Key::Left) | Event::Key(Key::Char('h')) => {
                self.tabs.focused_tab_mut().previous();
                // self.change_focus_to(self.tabs.focused_widget().neighbor_left(&self.tabs.focused_tab_type()));
            }
            Event::Key(Key::Right) | Event::Key(Key::Char('l')) => {
                self.tabs.focused_tab_mut().next();
                // self.change_focus_to(self.tabs.focused_widget().neighbor_right(&self.tabs.focused_tab_type()));
            }
            Event::Key(Key::Alt(ch)) => {
                let nr = ch.to_digit(10).map(|d| d as usize);
                if nr.is_some() {
                    self.tabs.select(nr);
                }
            }
            Event::Key(Key::Char('\t')) => {
                self.tabs.next();
            }
            Event::Key(Key::BackTab) => {
                self.tabs.previous();
            }
            Event::Key(Key::Char('v')) => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    let mut args: Option<(String, u32)> = None;
                    match self.tabs.focused_widget_type() {
                        FocusedWidget::ArchiveTable => {
                            if let Some(metadata) = &self.tabs.archive_table.get_by_index(i).1.metadata() {
                                args = Some((metadata.game.clone(), metadata.mod_id));
                            }
                        }
                        FocusedWidget::InstalledMods => {
                            let (_, md) = self.tabs.installed_mods_table.get_by_index(i);
                            if let ModDirectory::Nexus(im) = md {
                                args = Some((im.game.clone(), im.mod_id))
                            }
                        }
                        _ => {
                            return;
                        }
                    }
                    if let Some((game, mod_id)) = args {
                        let url = format!("https://www.nexusmods.com/{}/mods/{}", game, mod_id);
                        if Command::new("xdg-open").arg(url).status().is_err() {
                            LOGGER.log("xdg-open is needed to open URLs in browser.".to_string());
                        }
                    }
                }
            }
            Event::Key(Key::Char('f')) => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    match self.tabs.focused_widget_type() {
                        FocusedWidget::ArchiveTable => {
                            let (archive_name, _) = self.tabs.archive_table.get_by_index(i);
                            if let Some(mfd) = self.db.metadata_index.get_by_archive_name(archive_name).await {
                                let query = self.query.clone();
                                // let refresh_bottom_bar = self.bottom_bar.selected_has_changed.clone();
                                let events_tx = self.ui_events_tx.clone();
                                tokio::task::spawn(async move {
                                    query.verify_metadata(mfd).await;
                                    // refresh_bottom_bar.store(true, Ordering::Relaxed);
                                    // self.bottom_bar.refresh_for_archive_table(&self.tabs.archives_table);
                                    events_tx.send(NeedsRefresh::BottomBar).unwrap();
                                });
                            }
                        }
                        FocusedWidget::InstalledMods => {
                            let (_, mod_dir) = self.tabs.installed_mods_table.get_by_index(i);
                            if let ModDirectory::Nexus(im) = mod_dir {
                                if let Some(mfd) = self.db.metadata_index.get_by_file_id(&im.file_id).await {
                                    let query = self.query.clone();
                                    // let refresh_bottom_bar = self.bottom_bar.selected_has_changed.clone();
                                    let events_tx = self.ui_events_tx.clone();
                                    tokio::task::spawn(async move {
                                        query.verify_metadata(mfd).await;
                                        // refresh_bottom_bar.store(true, Ordering::Relaxed);
                                        events_tx.send(NeedsRefresh::BottomBar).unwrap();
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(Key::Delete) => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    match self.tabs.focused_widget_type() {
                        FocusedWidget::ArchiveTable => {
                            self.tabs.archive_table.delete_by_index(i).await;
                        }
                        FocusedWidget::InstalledMods => {
                            self.tabs.installed_mods_table.delete_by_index(i).await;
                        }
                        FocusedWidget::LogList => {
                            self.tabs.log_list.delete_selected();
                        }
                        FocusedWidget::DownloadTable => {
                            self.tabs.downloads_table.delete_by_index(i).await;
                        }
                    }
                    // Ensure selected index isn't out of bounds after deletion
                    self.tabs.focused_widget_mut().select(Some(i));
                }
            }
            Event::Key(Key::Char('i')) => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    match self.tabs.focused_widget_type() {
                        FocusedWidget::ArchiveTable => {
                            let (_, archive) = self.tabs.archive_table.get_by_index(i);
                            if let Some(metadata) = archive.metadata() {
                                self.updater.ignore_file(metadata.file_id).await;
                            }
                        }
                        FocusedWidget::InstalledMods => {
                            let (_, mod_dir) = self.tabs.installed_mods_table.get_by_index(i);
                            if let ModDirectory::Nexus(im) = mod_dir {
                                self.updater.ignore_file(im.file_id).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(Key::Char('U')) => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    match self.tabs.focused_widget_type() {
                        FocusedWidget::ArchiveTable => {
                            let (_, archive) = self.tabs.archive_table.get_by_index(i);
                            if let Some(metadata) = archive.metadata() {
                                if let Some(files) =
                                    self.db.metadata_index.get_modfiles(&metadata.game, &metadata.mod_id).await
                                {
                                    self.updater.update_mod(metadata.game.clone(), metadata.mod_id, files).await;
                                }
                            }
                        }
                        FocusedWidget::InstalledMods => {
                            let (_, mod_dir) = self.tabs.installed_mods_table.get_by_index(i);
                            if let ModDirectory::Nexus(im) = mod_dir {
                                if let Some(files) = self.db.metadata_index.get_modfiles(&im.game, &im.mod_id).await {
                                    self.updater.update_mod(im.game.clone(), im.mod_id, files).await;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(Key::Char('u')) => {
                self.updater.update_all().await;
            }

            _ => {
                // Uncomment to log keypresses
                //LOGGER.log(format!("{:?}", event));
            }
        }
        match self.tabs.focused_widget_type() {
            FocusedWidget::InstalledMods => {
                // no keys to handle
            }
            FocusedWidget::DownloadTable => {
                self.handle_downloads_keys(event).await;
            }
            FocusedWidget::ArchiveTable => {
                self.handle_archives_keys(event).await;
            }
            FocusedWidget::LogList => {
                // no keys to handle
            }
        }
    }

    async fn handle_downloads_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        if let Key::Char('p') = key {
            if let FocusedWidget::DownloadTable = self.tabs.focused_widget_type() {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    self.downloads.toggle_pause_for(i).await;
                }
            }
        }
    }

    async fn handle_archives_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        match key {
            Key::Char('\n') => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    //let mfi = self.db.file_index.get_by_index(i).await;
                    let (file_name, archive) = self.tabs.archive_table.get_by_index(i);
                    let dialog_title = "Directory name".to_string();
                    let mut suggested_values = vec![];
                    if let Some(mfd) = self.db.metadata_index.get_by_archive_name(file_name).await {
                        if let Some(name) = mfd.name().await {
                            suggested_values.push(name);
                        }
                        if let Some(modname) = mfd.mod_name().await {
                            // Sometimes the mod and mod file name are the same
                            if !suggested_values.contains(&modname) {
                                suggested_values.push(modname.clone());
                            }
                        }
                    } else {
                        LOGGER.log(format!("Warn: mod for {} doesn't exist in db", &file_name));
                    }
                    if suggested_values.is_empty() {
                        suggested_values.push(archive.file_name().clone());
                    }
                    self.rectangles.extract_dialog.layouts.set_suggestions_len(suggested_values.len());
                    self.popup_dialog.create_widget(suggested_values, dialog_title);
                    self.input_mode = InputMode::Extract;
                    self.render_active_widget();
                }
            }
            Key::Char('L') => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    let (_file_name, archive) = self.tabs.archive_table.get_by_index(i);
                    if let Some(res) = self.installer.list_content(archive.file_name()).await {
                        match res {
                            Ok(content) => {
                                for c in content {
                                    LOGGER.log(c.to_string());
                                }
                            }
                            Err(e) => {
                                LOGGER.log(format!("{:?}", e));
                            }
                        }
                    }
                }
            }
            Key::Char('p') => {
                if let Some(i) = self.tabs.focused_widget().selected() {
                    let (_, archive) = self.tabs.archive_table.get_by_index(i);
                    if let ArchiveEntry::File(archive) = archive {
                        self.installer.cancel(&archive).await;
                    }
                }
            }
            _ => {}
        }
    }

    async fn handle_confirm_dialog(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key {
                Key::Up | Key::Left => {
                    self.confirm_dialog.previous();
                    self.render_active_widget();
                }
                Key::Down | Key::Right => {
                    self.confirm_dialog.next();
                    self.render_active_widget();
                }
                Key::Char('\n') => {
                    if let 0 = self.confirm_dialog.selected().unwrap() {
                        let dest_dir = self.popup_dialog.get_content();
                        let index = self.tabs.archive_table.selected().unwrap();
                        let (file_name, _archive) = self.tabs.archive_table.get_by_index(index);
                        if let Err(e) = self.installer.extract(file_name.to_string(), dest_dir.to_string(), true).await
                        {
                            LOGGER.log(format!("Error when extracting {file_name}: {e}"));
                        }
                        self.input_mode = InputMode::Normal;
                    } else {
                        self.input_mode = InputMode::Extract;
                    }
                    self.render_active_widget();
                }
                Key::Ctrl('c') | Key::Esc => {
                    self.input_mode = InputMode::Extract;
                    self.render_active_widget();
                }
                _ => {}
            }
        }
    }

    async fn handle_popup_dialog(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key {
                Key::Ctrl('c') | Key::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                Key::Char('\n') => {
                    let dest_dir = self.popup_dialog.get_content();
                    let index = self.tabs.archive_table.selected().unwrap();
                    let (file_name, _archive) = self.tabs.archive_table.get_by_index(index);
                    match self.installer.extract(file_name.to_string(), dest_dir.to_string(), false).await {
                        Ok(()) => self.input_mode = InputMode::Normal,
                        Err(InstallError::AlreadyExists) => {
                            self.confirm_dialog =
                                // This should be handled somewhere else
                                ConfirmDialog::new(" Target directory already exists. Overwrite? ".to_string());
                            self.input_mode = InputMode::Confirm;
                        }
                        Err(e) => {
                            LOGGER.log(format!("Failed to extract to {dest_dir}: {}", e));
                            self.input_mode = InputMode::Normal;
                        }
                    }
                }
                _ => {
                    self.popup_dialog.input(Event::Key(key));
                }
            }
            self.render_active_widget();
        }
    }
}
