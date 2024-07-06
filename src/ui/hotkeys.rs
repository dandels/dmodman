use super::component::traits::Select;
use super::component::{ConfirmDialog, PopupDialog};
use super::main_ui::*;
use super::navigation::*;
use crate::cache::ArchiveEntry;
use crate::extract::{InstallError, ModDirectory};
use std::process::Command;
use std::sync::atomic::Ordering;
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
    pub async fn handle_events(&mut self, event: Event) {
        //MouseEvent::Press(mouse_event, x, y) => {
        //self.logger.log(format!("click! {mouse_event:?}, x: {x}, y: {y}"));
        //Event::Unsupported(u) => {
        //self.logger.log(format!("Unsupported: {u:?}"));
        if let InputMode::Confirm = self.input_mode {
            self.handle_confirm_dialog(event).await;
            return;
        }
        if let InputMode::ReadLine = self.input_mode {
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
                self.logger.log("Refusing to quit, archive extraction is still in progress.");
            }
            return;
        }

        match event {
            Event::Key(Key::Down)
            | Event::Key(Key::Char('j'))
            | Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.focused_widget_mut().next();
            }
            Event::Key(Key::Up)
            | Event::Key(Key::Char('k'))
            | Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.focused_widget_mut().previous();
            }
            Event::Key(Key::Char('H')) => {
                self.change_focus_to(self.focused_widget().neighbor_left(&self.tabs.active()));
            }
            Event::Key(Key::Char('J')) => {
                self.change_focus_to(self.focused_widget().neighbor_down(&self.tabs.active()));
            }
            Event::Key(Key::Char('K')) => {
                self.change_focus_to(self.focused_widget().neighbor_up(&self.tabs.active()));
            }
            Event::Key(Key::Char('L')) => {
                self.change_focus_to(self.focused_widget().neighbor_right(&self.tabs.active()));
            }
            Event::Key(Key::Left) | Event::Key(Key::Char('h')) => {
                let focused = self.focused_widget();
                if let Some(left) = focused.neighbor_left(&self.tabs.active()) {
                    self.change_focus_to(Some(left));
                } else if let Some(up) = focused.neighbor_up(&self.tabs.active()) {
                    self.change_focus_to(Some(up));
                } else if let Some(down) = focused.neighbor_down(&self.tabs.active()) {
                    self.change_focus_to(Some(down));
                }
            }
            Event::Key(Key::Right) | Event::Key(Key::Char('l')) => {
                let focused = self.focused_widget();
                if let Some(right) = focused.neighbor_right(&self.tabs.active()) {
                    self.change_focus_to(Some(right));
                } else if let Some(up) = focused.neighbor_up(&self.tabs.active()) {
                    self.change_focus_to(Some(up));
                } else if let Some(down) = focused.neighbor_down(&self.tabs.active()) {
                    self.change_focus_to(Some(down));
                }
            }
            Event::Key(Key::Alt(ch)) => {
                if let Some(nr) = ch.to_digit(10) {
                    if let Some(nr) = (nr as usize).checked_sub(1) {
                        self.select_tab(nr);
                    }
                }
            }
            Event::Key(Key::Char('\t')) => {
                self.next_tab();
            }
            Event::Key(Key::BackTab) => {
                self.previous_tab();
            }
            Event::Key(Key::Char('v')) => {
                if let Some(i) = self.focused_widget().selected() {
                    let mut args: Option<(String, u32)> = None;
                    match self.tabs.focused() {
                        Focused::ArchiveTable => {
                            if let Some(metadata) = &self.archives_view.get_by_index(i).1.metadata() {
                                args = Some((metadata.game.clone(), metadata.mod_id));
                            }
                        }
                        Focused::InstalledMods => {
                            let (_, md) = self.installed_mods_table.get_by_index(i);
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
                            self.logger.log("xdg-open is needed to open URLs in browser.".to_string());
                        }
                    }
                }
            }
            Event::Key(Key::Char('f')) => {
                if let Some(i) = self.focused_widget().selected() {
                    match self.tabs.focused() {
                        Focused::ArchiveTable => {
                            let (archive_name, _) = self.archives_view.get_by_index(i);
                            if let Some(mfd) = self.cache.metadata_index.get_by_archive_name(archive_name).await {
                                let query = self.query.clone();
                                let refresh_bottom_bar = self.bottom_bar.selected_has_changed.clone();
                                tokio::task::spawn(async move {
                                    query.verify_metadata(mfd).await;
                                    refresh_bottom_bar.store(true, Ordering::Relaxed);
                                });
                            }
                        }
                        Focused::InstalledMods => {
                            let (_, mod_dir) = self.installed_mods_table.get_by_index(i);
                            if let ModDirectory::Nexus(im) = mod_dir {
                                if let Some(mfd) = self.cache.metadata_index.get_by_file_id(&im.file_id).await {
                                    let query = self.query.clone();
                                    let refresh_bottom_bar = self.bottom_bar.selected_has_changed.clone();
                                    tokio::task::spawn(async move {
                                        query.verify_metadata(mfd).await;
                                        refresh_bottom_bar.store(true, Ordering::Relaxed);
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(Key::Delete) => {
                if let Some(i) = self.focused_widget().selected() {
                    match self.tabs.focused() {
                        Focused::ArchiveTable => {
                            self.archives_view.delete_by_index(i).await;
                        }
                        Focused::InstalledMods => {
                            self.installed_mods_table.delete_by_index(i).await;
                        }
                        Focused::LogList => {
                            self.log_view.delete_selected();
                        }
                        Focused::DownloadTable => {
                            self.downloads_view.delete_by_index(i).await;
                            self.downloads_view.len = self.downloads_view.len.saturating_sub(1);
                        }
                    }
                    // Ensure selected index isn't out of bounds after deletion
                    self.focused_widget_mut().select(Some(i));
                }
            }
            Event::Key(Key::Char('i')) => {
                if let Some(i) = self.focused_widget().selected() {
                    match self.tabs.focused() {
                        Focused::ArchiveTable => {
                            let (_, archive) = self.archives_view.get_by_index(i);
                            if let Some(metadata) = archive.metadata() {
                                self.updater.ignore_file(metadata.file_id).await;
                            }
                        }
                        Focused::InstalledMods => {
                            let (_, mod_dir) = self.installed_mods_table.get_by_index(i);
                            if let ModDirectory::Nexus(im) = mod_dir {
                                self.updater.ignore_file(im.file_id).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(Key::Char('U')) => {
                if let Some(i) = self.focused_widget().selected() {
                    match self.tabs.focused() {
                        Focused::ArchiveTable => {
                            let (_, archive) = self.archives_view.get_by_index(i);
                            if let Some(metadata) = archive.metadata() {
                                if let Some(files) =
                                    self.cache.metadata_index.get_modfiles(&metadata.game, &metadata.mod_id).await
                                {
                                    self.updater.update_mod(metadata.game.clone(), metadata.mod_id, files).await;
                                }
                            }
                        }
                        Focused::InstalledMods => {
                            let (_, mod_dir) = self.installed_mods_table.get_by_index(i);
                            if let ModDirectory::Nexus(im) = mod_dir {
                                if let Some(files) = self.cache.metadata_index.get_modfiles(&im.game, &im.mod_id).await
                                {
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
                //self.logger.log(format!("{:?}", key));
            }
        }
        match self.tabs.focused() {
            Focused::InstalledMods => {
                // no keys to handle
            }
            Focused::DownloadTable => {
                self.handle_downloads_keys(event).await;
            }
            Focused::ArchiveTable => {
                self.handle_archives_keys(event).await;
            }
            Focused::LogList => {
                // no keys to handle
            }
        }
    }

    async fn handle_downloads_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        if let Key::Char('p') = key {
            if let Focused::DownloadTable = self.tabs.focused() {
                if let Some(i) = self.focused_widget().selected() {
                    self.downloads.toggle_pause_for(i).await;
                }
            }
        }
    }

    async fn handle_archives_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        match key {
            Key::Char('\n') => {
                if let Some(i) = self.focused_widget().selected() {
                    //let mfi = self.cache.file_index.get_by_index(i).await;
                    let (file_name, archive) = self.archives_view.get_by_index(i);
                    let dialog_title = "Directory name".to_string();
                    let mut suggested_values = vec![];
                    if let Some(mfd) = self.cache.metadata_index.get_by_archive_name(file_name).await {
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
                        self.logger.log(format!("Warn: mod for {} doesn't exist in db", &file_name));
                    }
                    if suggested_values.is_empty() {
                        suggested_values.push(archive.file_name().clone());
                    }
                    self.popup_dialog = PopupDialog::new(self.config.clone(), suggested_values, dialog_title);
                    self.input_mode = InputMode::ReadLine;
                    self.redraw_terminal = true;
                }
            }
            Key::Char('L') => {
                if let Some(i) = self.focused_widget().selected() {
                    let (_file_name, archive) = self.archives_view.get_by_index(i);
                    if let Some(res) = self.installer.list_content(archive.file_name()).await {
                        match res {
                            Ok(content) => {
                                for c in content {
                                    self.logger.log(c.to_string());
                                }
                            }
                            Err(e) => {
                                self.logger.log(format!("{:?}", e));
                            }
                        }
                    }
                }
            }
            Key::Char('p') => {
                if let Some(i) = self.focused_widget().selected() {
                    let (_, archive) = self.archives_view.get_by_index(i);
                    if let ArchiveEntry::File(archive) = archive {
                        self.installer.cancel(archive).await;
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
                    self.redraw_terminal = true;
                }
                Key::Down | Key::Right => {
                    self.confirm_dialog.next();
                    self.redraw_terminal = true;
                }
                Key::Char('\n') => {
                    if let 0 = self.confirm_dialog.selected().unwrap() {
                        let dest_dir = self.popup_dialog.get_content();
                        let index = self.archives_view.selected().unwrap();
                        let (file_name, _archive) = self.archives_view.get_by_index(index);
                        if let Err(e) = self.installer.extract(file_name.to_string(), dest_dir.to_string(), true).await
                        {
                            self.logger.log(format!("Error when extracting {file_name}: {e}"));
                        }
                        self.input_mode = InputMode::Normal;
                    } else {
                        self.input_mode = InputMode::ReadLine;
                    }
                    self.redraw_terminal = true;
                }
                Key::Ctrl('c') | Key::Esc => {
                    self.input_mode = InputMode::ReadLine;
                    self.redraw_terminal = true;
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
                    let index = self.archives_view.selected().unwrap();
                    let (file_name, _archive) = self.archives_view.get_by_index(index);
                    match self.installer.extract(file_name.to_string(), dest_dir.to_string(), false).await {
                        Ok(()) => self.input_mode = InputMode::Normal,
                        Err(InstallError::AlreadyExists) => {
                            self.confirm_dialog =
                                // This should be handled somewhere else
                                ConfirmDialog::new(" Target directory already exists. Overwrite? ".to_string());
                            self.input_mode = InputMode::Confirm;
                        }
                        Err(e) => {
                            self.logger.log(format!("Failed to extract to {dest_dir}: {}", e));
                            self.input_mode = InputMode::Normal;
                        }
                    }
                }
                _ => {
                    self.popup_dialog.input(Event::Key(key));
                }
            }
            self.redraw_terminal = true;
        }
    }
}
