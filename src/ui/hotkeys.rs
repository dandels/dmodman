use super::component::traits::Select;
use super::component::{ConfirmDialog, PopupDialog};
use super::main_ui::*;
use super::navigation::*;
use crate::archives::InstallError;
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
            if self.archives.extract_jobs.read().unwrap().is_empty() {
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
                self.change_focus_to(self.focused_widget().neighbor_left(&self.tabs.active()));
            }
            Event::Key(Key::Right) | Event::Key(Key::Char('l')) => {
                self.change_focus_to(self.focused_widget().neighbor_right(&self.tabs.active()));
            }
            Event::Key(Key::Alt(ch)) => {
                if let Some(nr) = ch.to_digit(10) {
                    (nr as usize).checked_sub(1).and_then(|nr| Some(self.select_tab(nr)));
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
            Key::Char('i') => {
                if let Focused::FileTable = self.tabs.focused() {
                    if let Some(i) = self.focused_widget().selected() {
                        self.updater.ignore_file(i).await;
                    }
                }
            }
            Key::Char('U') => {
                if let Some(i) = self.focused_widget().selected() {
                    let (game, mod_id, files) = self.cache.file_index.get_game_mod_files_by_index(i).await;
                    self.updater.update_mod(game, mod_id, files).await;
                }
            }
            Key::Char('u') => {
                self.updater.update_all().await;
            }
            Key::Char('v') => {
                if let Some(i) = self.focused_widget().selected() {
                    let fd = self.cache.file_index.get_by_index(i).await;
                    let url = format!("https://www.nexusmods.com/{}/mods/{}", fd.game, fd.mod_id);
                    if Command::new("xdg-open").arg(url).status().is_err() {
                        self.logger.log("xdg-open is needed to open URLs in browser.".to_string());
                    }
                }
            }
            Key::Delete => {
                if let Some(i) = self.focused_widget().selected() {
                    if let Err(e) = self.cache.file_index.delete_by_index(i).await {
                        self.logger.log(format!("Unable to delete file: {}", e));
                    } else {
                        if i == 0 {
                            self.focused_widget_mut().select(None);
                        }
                        self.focused_widget_mut().previous();
                    }
                }
            }
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
                if let Some(i) = self.focused_widget().selected() {
                    self.downloads_view.downloads.delete(i).await;
                    if i == 0 {
                        self.focused_widget_mut().select(None);
                    }
                    self.focused_widget_mut().previous();
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
                    let path = self.archives.files.read().await.get(i).unwrap().path();
                    match self.archives.list_content(path.clone()).await {
                        Ok(_) => {}
                        Err(e) => {
                            self.logger.log(format!("{:?}", e));
                        }
                    }
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let dialog_title = "Directory name".to_string();
                    let mut suggested_values = vec![];
                    if let Some(fdata) = self.cache.file_index.get_by_filename(&file_name).await {
                        if let Some(fd) = &fdata.file_details {
                            suggested_values.push(fd.name.clone());
                        }
                        if let Some(modname) =
                            self.cache.md5result.get(&fdata.game, fdata.file_id).await.and_then(|res| res.r#mod.name)
                        {
                            suggested_values.push(modname);
                        }
                    } else {
                        self.logger.log(format!("Warn: mod for {file_name} doesn't exist in db"));
                        suggested_values.push(file_name);
                    }
                    self.popup_dialog = PopupDialog::new(self.config.clone(), suggested_values, dialog_title);
                    self.input_mode = InputMode::ReadLine;
                    self.redraw_terminal = true;
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
                    self.logger.remove(i).await;
                    if i == 0 {
                        self.focused_widget_mut().select(None);
                    }
                    self.focused_widget_mut().previous();
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
                        if let Err(e) = self.archives.extract(index, dest_dir.to_string(), true).await {
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
                    match self.archives.extract(index, dest_dir.to_string(), false).await {
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
                }
                _ => {
                    self.popup_dialog.input(Event::Key(key));
                }
            }
            self.redraw_terminal = true;
        }
    }
}
