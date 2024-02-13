use std::process::Command;

use std::sync::atomic::Ordering;
use termion::event::{Event, Key, MouseEvent};

//use tui_textarea::{Input, Key};
//use tui_textarea::{Input, Key, TextArea};
use super::component::traits::*;
use super::component::*;
use super::main_ui::*;

impl MainUI<'_> {
    pub async fn handle_events(&mut self, event: Event) {
        let key: Key;
        match event {
            Event::Key(k) => key = k,
            Event::Mouse(m) => match m {
                MouseEvent::Press(mouse_event, x, y) => {
                    self.msgs.push(format!("click! {mouse_event:?}, x: {x}, y: {y}")).await;
                    return;
                }
                _ => {
                    return;
                }
            },
            Event::Unsupported(u) => {
                self.msgs.push(format!("Unsupported: {u:?}")).await;
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
                self.focus_next();
            }
            Key::Up | Key::Char('k') => {
                self.focus_previous();
            }
            Key::Left | Key::Char('h') => match self.focused {
                FocusedWidget::MessageList | FocusedWidget::DownloadTable => {
                    self.change_focus_to(FocusedWidget::FileTable);
                }
                FocusedWidget::FileTable => {
                    self.change_focus_to(FocusedWidget::MessageList);
                }
                _ => {}
            },
            Key::Right | Key::Char('l') => match self.focused {
                FocusedWidget::MessageList | FocusedWidget::FileTable => {
                    self.change_focus_to(FocusedWidget::DownloadTable);
                }
                FocusedWidget::DownloadTable => {
                    self.change_focus_to(FocusedWidget::MessageList);
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
                //self.msgs.push(format!("{:?}", key)).await;
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
            FocusedWidget::MessageList => {
                self.handle_messages_keys(key).await;
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
                        self.msgs.push("xdg-open is needed to open URLs in browser.".to_string()).await;
                    }
                }
            }
            Key::Delete => {
                if let Some(i) = self.selected_index() {
                    if let Err(e) = self.cache.delete_by_index(i).await {
                        self.msgs.push(format!("Unable to delete file: {}", e)).await;
                    } else {
                        if i == 0 {
                            self.select_widget_index(None);
                        }
                        self.focus_previous();
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
                    self.focus_previous();
                }
            }
            _ => {}
        }
    }

    async fn handle_archives_keys(&mut self, key: Key) {
        match key {
            Key::Char('\n') => {
                println!("pressed enter");
                if let Some(i) = self.selected_index() {
                    let path = self.archives_view.archives.files.get(i).unwrap().path();
                    let _contents = self.archives_view.archives.list_contents(&path).await;
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    if let Some(fd) = self.cache.file_index.get_by_filename(&file_name).await {
                        self.input_line.get_file_name(&fd.file_details.name);
                    } else {
                        self.msgs.push("Warn: mod for {file_name} doesn't exist in db").await;
                        self.input_line.get_file_name(&file_name);
                    }
                    self.input_mode = InputMode::ReadLine;
                    self.redraw_terminal.store(true, Ordering::Relaxed);
                }
            }
            Key::Delete => {
                self.msgs.push("Not implemented.").await;
            }
            _ => {}
        }
    }

    async fn handle_messages_keys(&mut self, key: Key) {
        match key {
            Key::Delete => {
                if let Some(i) = self.selected_index() {
                    self.msgs_view.msgs.remove(i).await;
                    if i == 0 {
                        self.select_widget_index(None);
                    }
                    self.focus_previous();
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
                self.msgs.push(self.input_line.get_contents()).await;
                if self.input_line.clear() {
                    self.msgs.push("Succesful delete").await;
                } else {
                    self.msgs.push("Failed delete").await;
                }
                self.input_mode = InputMode::Normal;
                self.redraw_terminal.store(true, Ordering::Relaxed);
            }
            _ => {
                self.input_line.textarea.input(key);
            }
        }
        self.redraw_terminal.store(true, Ordering::Relaxed);
    }
}