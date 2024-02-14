use std::process::Command;

use std::sync::atomic::Ordering;
use termion::event::{Event, Key, MouseEvent};

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
        let key: Key;
        match event {
            Event::Key(k) => key = k,
            Event::Mouse(m) => match m {
                MouseEvent::Press(mouse_event, x, y) => {
                    self.logger.log(format!("click! {mouse_event:?}, x: {x}, y: {y}"));
                    return;
                }
                _ => {
                    return;
                }
            },
            Event::Unsupported(u) => {
                self.logger.log(format!("Unsupported: {u:?}"));
                return;
            }
        }
        if let InputMode::ReadLine = self.input_mode {
            self.read_input_line(key).await;
            return;
        }

        if let Key::Char('q') | Key::Ctrl('c') = key {
            self.should_run = false;
            return;
        }

        match key {
            Key::Down | Key::Char('j') => {
                self.select_next();
            }
            Key::Up | Key::Char('k') => {
                self.select_previous();
            }
            Key::Left | Key::Char('h') => match self.focused {
                FocusedWidget::LogList | FocusedWidget::DownloadTable => {
                    self.change_focus_to(FocusedWidget::FileTable);
                }
                FocusedWidget::FileTable => {
                    self.change_focus_to(FocusedWidget::LogList);
                }
                _ => {}
            },
            Key::Right | Key::Char('l') => match self.focused {
                FocusedWidget::LogList | FocusedWidget::FileTable => {
                    self.change_focus_to(FocusedWidget::DownloadTable);
                }
                FocusedWidget::DownloadTable => {
                    self.change_focus_to(FocusedWidget::LogList);
                }
                _ => {}
            },
            Key::Char('\t') => {
                self.tab_bar.next_tab();
                self.change_focused_tab().await;
            }
            Key::BackTab => {
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
                self.handle_files_keys(key).await;
            }
            FocusedWidget::DownloadTable => {
                self.handle_downloads_keys(key).await;
            }
            FocusedWidget::ArchiveTable => {
                self.handle_archives_keys(key).await;
            }
            FocusedWidget::LogList => {
                self.handle_log_keys(key).await;
            }
        }
    }

    async fn handle_files_keys(&mut self, key: Key) {
        match key {
            Key::Char('i') => {
                if let FocusedWidget::FileTable = self.focused {
                    if let Some(i) = self.selected_index() {
                        self.updater.ignore_file(i).await;
                    }
                }
            }
            Key::Char('U') => {
                let game: String;
                let mod_id: u32;
                {
                    if let Some(i) = self.selected_index() {
                        let files_lock = self.files_view.file_index.files_sorted.read().await;
                        let fdata = files_lock.get(i).unwrap();
                        let lf_lock = fdata.local_file.read().await;
                        game = lf_lock.game.clone();
                        mod_id = lf_lock.mod_id;
                    } else {
                        return;
                    }
                }
                self.updater.update_mod(game, mod_id).await;
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

    async fn handle_downloads_keys(&mut self, key: Key) {
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

    async fn handle_archives_keys(&mut self, key: Key) {
        match key {
            Key::Char('i') => {
                if let Some(i) = self.selected_index() {
                    let path = self.archives.files.get(i).unwrap().path();
                    match self.archives.list_contents(path.clone()).await {
                        Ok(_) => {}
                        Err(e) => {
                            self.logger.log(format!("{:?}", e));
                        }
                    }
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    if let Some(fd) = self.cache.file_index.get_by_filename(&file_name).await {
                        self.input_line.ask_extract_destination(&fd.file_details.name);
                    } else {
                        self.logger.log("Warn: mod for {file_name} doesn't exist in db");
                        self.input_line.ask_extract_destination(&file_name);
                    }
                    self.input_mode = InputMode::ReadLine;
                    self.redraw_terminal.store(true, Ordering::Relaxed);
                }
            }
            Key::Delete => {
                self.logger.log("Not implemented.");
            }
            _ => {}
        }
    }

    async fn handle_log_keys(&mut self, key: Key) {
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

    async fn read_input_line(&mut self, key: Key) {
        match key {
            Key::Ctrl('c') | Key::Esc => {
                self.input_mode = InputMode::Normal;
            }
            Key::Char('\n') => {
                let dest_dir = self.input_line.get_contents();
                self.archives.extract(self.archives_view.selected().unwrap(), dest_dir).await;
                self.input_mode = InputMode::Normal;
                self.redraw_terminal.store(true, Ordering::Relaxed);
            }
            // disable tab character
            Key::Char('\t') => {}
            _ => {
                self.input_line.textarea.input(key);
            }
        }
        self.redraw_terminal.store(true, Ordering::Relaxed);
    }
}
