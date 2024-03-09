use super::component::traits::Select;
use super::component::{ConfirmDialog, PopupDialog};
use super::main_ui::*;
use super::navigation::*;
use crate::install::InstallError;
use std::process::Command;
use termion::event::{Event, Key, MouseButton, MouseEvent};

pub const ARCHIVES_KEYS: &[(&str, &str)] = &[("<Return>", "install "), ("<Del>", "delete "), ("<q>", "quit ")];
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

        if let Event::Key(Key::Char('q')) | Event::Key(Key::Ctrl('c')) = event {
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
            _ => {
                // Uncomment to log keypresses
                //self.logger.log(format!("{:?}", key));
            }
        }
        match self.tabs.focused() {
            Focused::FileTable => {
                self.handle_files_keys(event).await;
            }
            Focused::DownloadTable => {
                self.handle_downloads_keys(event).await;
            }
            Focused::ArchiveTable => {
                self.handle_archives_keys(event).await;
            }
            Focused::LogList => {
                self.handle_log_keys(event).await;
            }
        }
    }

    async fn handle_files_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        match key {
            //Key::Char('i') => {
            //    if let Focused::FileTable = self.tabs.focused() {
            //        if let Some(i) = self.focused_widget().selected() {
            //            self.updater.ignore_file(i).await;
            //        }
            //    }
            //}
            //Key::Char('U') => {
            //    if let Some(i) = self.focused_widget().selected() {
            //        let (game, mod_id, files) = self.cache.file_index.get_game_mod_files_by_index(i).await;
            //        self.updater.update_mod(game, mod_id, files).await;
            //    }
            //}
            Key::Char('u') => {
                self.updater.update_all().await;
            }
            //Key::Char('v') => {
            //    if let Some(i) = self.focused_widget().selected() {
            //        let fd = self.cache.file_index.get_by_index(i).await;
            //        let url = format!("https://www.nexusmods.com/{}/mods/{}", fd.game, fd.mod_id);
            //        if Command::new("xdg-open").arg(url).status().is_err() {
            //            self.logger.log("xdg-open is needed to open URLs in browser.".to_string());
            //        }
            //    }
            //}
            // Disabled during rewrite
            //Key::Delete => {
            //    // TODO handle deletion inside widget or something
            //    if let Some(i) = self.focused_widget().selected() {
            //        if let Err(e) = self.cache.file_index.delete_by_index(i).await {
            //            self.logger.log(format!("Unable to delete file: {}", e));
            //        } else {
            //            self.focused_widget_mut().next();
            //            self.files_view.len = self.files_view.len.saturating_sub(1);
            //        }
            //    }
            //}
            _ => {}
        }
    }

    async fn handle_downloads_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        match key {
            Key::Char('p') => {
                if let Focused::DownloadTable = self.tabs.focused() {
                    if let Some(i) = self.focused_widget().selected() {
                        self.downloads.toggle_pause_for(i).await;
                    }
                }
            }
            Key::Delete => {
                // TODO handle deletion somewhere cleaner
                if let Some(i) = self.focused_widget().selected() {
                    self.downloads_view.downloads.delete(i).await;
                    self.downloads_view.len = self.downloads_view.len.saturating_sub(1);
                    self.focused_widget_mut().next();
                }
            }
            _ => {}
        }
    }

    async fn handle_archives_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        match key {
            Key::Char('\n') => {
                if let Some(i) = self.focused_widget().selected() {
                    //let mfi = self.cache.file_index.get_by_index(i).await;
                    let archive = self.cache.archives.get_by_index(i).await.unwrap();
                    let dialog_title = "Directory name".to_string();
                    let mut suggested_values = vec![];
                    if let Some(mfd) = self.cache.file_index.get_by_archive_name(&archive.file_name).await {
                        let lock = mfd.file_details.read().await;
                        if let Some(fd) = lock.as_ref() {
                            suggested_values.push(fd.name.clone());
                        }
                        let lock = mfd.md5results.read().await;
                        if let Some(md5res) = lock.as_ref() {
                            if let Some(modname) = &md5res.r#mod.name {
                                suggested_values.push(modname.clone());
                            }
                        }
                    } else {
                        self.logger.log(format!("Warn: mod for {} doesn't exist in db", &archive.file_name));
                        suggested_values.push(archive.file_name.to_string());
                    }
                    self.popup_dialog = PopupDialog::new(self.config.clone(), suggested_values, dialog_title);
                    self.input_mode = InputMode::ReadLine;
                    self.redraw_terminal = true;
                }
            }
            Key::Char('L') => {
                if let Some(i) = self.focused_widget().selected() {
                    let archive = self.cache.archives.get_by_index(i).await.unwrap();
                    match self.installer.list_content(&archive.file_name).await {
                        Ok(content) => {
                            for c in content {
                                self.logger.log(format!("{}", c));
                            }
                        }
                        Err(e) => {
                            self.logger.log(format!("{:?}", e));
                        }
                    }
                }
            }
            Key::Delete => {
                self.logger.log("Not implemented.");
            }
            _ => {}
        }
    }

    async fn handle_log_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        #[allow(clippy::single_match)]
        match key {
            Key::Delete => {
                if let Some(i) = self.focused_widget().selected() {
                    self.log_view.remove(i);
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
                        let archive = self.cache.archives.get_by_index(index).await.unwrap();
                        if let Err(e) =
                            self.installer.extract(archive.file_name.clone(), dest_dir.to_string(), true).await
                        {
                            self.logger.log(format!("Error when extracting: {e}"));
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
                    if let Some(archive) = self.cache.archives.get_by_index(index).await {
                        match self.installer.extract(archive.file_name.clone(), dest_dir.to_string(), false).await {
                            Ok(()) => self.input_mode = InputMode::Normal,
                            Err(InstallError::AlreadyExists) => {
                                self.confirm_dialog =
                                    // This should be handled somewhere else
                                    ConfirmDialog::new(" Target directory already exists. Overwrite? ".to_string());
                                self.input_mode = InputMode::Confirm;
                            }
                            Err(e) => {
                                self.logger.log(format!("Failed to extract to {dest_dir}: {}", e));
                            }
                        }
                    } else {
                        self.logger.log(format!("Archive to extract no longer exists."));
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
