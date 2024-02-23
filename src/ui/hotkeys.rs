use std::process::Command;

use std::sync::atomic::Ordering;
use termion::event::{Event, Key, MouseButton, MouseEvent};

//use tui_textarea::{Input, Key};
//use tui_textarea::{Input, Key, TextArea};
use super::component::traits::*;
use super::component::*;
use super::main_ui::*;

pub const ARCHIVES_KEYS: &[(&str, &str)] = &[("<i>", "install "), ("<Del>", "delete "), ("<q>", "quit ")];
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

impl MainUI<'_> {
    pub async fn handle_events(&mut self, event: Event) {
        //MouseEvent::Press(mouse_event, x, y) => {
        //self.logger.log(format!("click! {mouse_event:?}, x: {x}, y: {y}"));
        //Event::Unsupported(u) => {
        //self.logger.log(format!("Unsupported: {u:?}"));
        if let InputMode::ReadLine = self.input_mode {
            self.read_input_line(event).await;
            return;
        }

        if let Event::Key(Key::Char('q')) | Event::Key(Key::Ctrl('c')) = event {
            self.should_run = false;
            return;
        }

        match event {
            Event::Key(Key::Down)
            | Event::Key(Key::Char('j'))
            | Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.select_next();
            }
            Event::Key(Key::Up)
            | Event::Key(Key::Char('k'))
            | Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.select_previous();
            }
            Event::Key(Key::Left) | Event::Key(Key::Char('h')) => match self.focused {
                FocusedWidget::LogList | FocusedWidget::DownloadTable => {
                    self.change_focus_to(FocusedWidget::FileTable);
                }
                FocusedWidget::FileTable => {
                    self.change_focus_to(FocusedWidget::LogList);
                }
                _ => {}
            },
            Event::Key(Key::Right) | Event::Key(Key::Char('l')) => match self.focused {
                FocusedWidget::LogList | FocusedWidget::FileTable => {
                    self.change_focus_to(FocusedWidget::DownloadTable);
                }
                FocusedWidget::DownloadTable => {
                    self.change_focus_to(FocusedWidget::LogList);
                }
                _ => {}
            },
            Event::Key(Key::Char('\t')) => {
                self.tab_bar.next_tab();
                self.change_focused_tab().await;
            }
            Event::Key(Key::BackTab) => {
                self.tab_bar.prev_tab();
                self.change_focused_tab().await;
            }
            _ => {
                // Uncomment to log keypresses
                //self.logger.log(format!("{:?}", key));
            }
        }
        match self.focused {
            FocusedWidget::FileTable => {
                self.handle_files_keys(event).await;
            }
            FocusedWidget::DownloadTable => {
                self.handle_downloads_keys(event).await;
            }
            FocusedWidget::ArchiveTable => {
                self.handle_archives_keys(event).await;
            }
            FocusedWidget::LogList => {
                self.handle_log_keys(event).await;
            }
        }
    }

    async fn handle_files_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        match key {
            Key::Char('i') => {
                if let FocusedWidget::FileTable = self.focused {
                    if let Some(i) = self.selected_index() {
                        self.updater.ignore_file(i).await;
                    }
                }
            }
            Key::Char('U') => {
                if let Some(i) = self.selected_index() {
                    let game: String;
                    let mod_id: u32;
                    let files;
                    {
                        let files_lock = self.cache.file_index.files_sorted.read().await;
                        let fdata = files_lock.get(i).unwrap();
                        let lf_lock = fdata.local_file.read().await;
                        game = lf_lock.game.clone();
                        mod_id = lf_lock.mod_id;
                        let map_lock = self.cache.file_index.game_to_mods_map.read().await;
                        let mods = map_lock.get(&game).unwrap();
                        files = mods.get(&mod_id).unwrap().clone();
                    }
                    self.updater.update_mod(game, mod_id, files).await;
                }
            }
            Key::Char('u') => {
                self.updater.update_all().await;
            }
            Key::Char('v') => {
                if let Some(i) = self.selected_index() {
                    let files_lock = self.files_view.file_index.files_sorted.read().await;
                    let fdata = files_lock.get(i).unwrap();
                    let lf_lock = fdata.local_file.read().await;
                    let url = format!("https://www.nexusmods.com/{}/mods/{}", &lf_lock.game, &lf_lock.mod_id);
                    if Command::new("xdg-open").arg(url).status().is_err() {
                        self.logger.log("xdg-open is needed to open URLs in browser.".to_string());
                    }
                }
            }
            Key::Delete => {
                if let Some(i) = self.selected_index() {
                    if let Err(e) = self.cache.delete_by_index(i).await {
                        self.logger.log(format!("Unable to delete file: {}", e));
                    } else {
                        if i == 0 {
                            self.select_widget_index(None);
                        }
                        self.select_previous();
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
                if let FocusedWidget::DownloadTable = self.focused {
                    if let Some(i) = self.selected_index() {
                        self.downloads.toggle_pause_for(i).await;
                    }
                }
            }
            Key::Delete => {
                if let Some(i) = self.selected_index() {
                    self.downloads_view.downloads.delete(i).await;
                    if i == 0 {
                        self.select_widget_index(None);
                    }
                    self.select_previous();
                }
            }
            _ => {}
        }
    }

    async fn handle_archives_keys(&mut self, event: Event) {
        let key = if let Event::Key(key) = event { key } else { return };

        match key {
            Key::Char('i') => {
                if let Some(i) = self.selected_index() {
                    let path = self.archives.files.read().await.get(i).unwrap().path();
                    match self.archives.list_contents(path.clone()).await {
                        Ok(_) => {}
                        Err(e) => {
                            self.logger.log(format!("{:?}", e));
                        }
                    }
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    let dialog_title = "Target directory".to_string();
                    if let Some(fd) = self.cache.file_index.get_by_filename(&file_name).await {
                        self.popup_dialog.show(&fd.file_details.name, dialog_title);
                    } else {
                        self.logger.log(format!("Warn: mod for {file_name} doesn't exist in db"));
                        self.popup_dialog.show(&file_name, dialog_title);
                    }
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

        match key {
            Key::Delete => {
                if let Some(i) = self.selected_index() {
                    self.log_view.logger.remove(i).await;
                    if i == 0 {
                        self.select_widget_index(None);
                    }
                    self.select_previous();
                }
            }
            _ => {}
        }
    }

    async fn change_focused_tab(&mut self) {
        match self.tab_bar.selected() {
            Some(0) => {
                // TODO remember previously focused pane
                self.change_focus_to(FocusedWidget::FileTable);
            }
            Some(1) => self.change_focus_to(FocusedWidget::ArchiveTable),
            None => {
                panic!("Invalid tabstate")
            }
            _ => {}
        }
    }

    async fn read_input_line(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key {
                Key::Ctrl('c') | Key::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                Key::Char('\n') => {
                    let dest_dir = self.popup_dialog.get_contents();
                    self.archives.extract(self.archives_view.selected().unwrap(), dest_dir).await;
                    self.input_mode = InputMode::Normal;
                }
                // disable tab character
                Key::Char('\t') => {}
                _ => {
                    self.popup_dialog.textarea.input(key);
                }
            }
            self.redraw_terminal = true;
        }
    }
}
