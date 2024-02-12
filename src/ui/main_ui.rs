use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use termion::event::Key;
use tokio::task;

use super::component::traits::*;
use super::component::*;
use super::event::{Event, Events};
use crate::api::{Client, Downloads, UpdateChecker};
use crate::archives::Archives;
use crate::cache::Cache;
use crate::config::Config;
use crate::ui::rectangles::Rectangles;
use crate::ui::*;
use crate::Messages;

pub struct MainUI<'a> {
    cache: Cache,
    downloads: Downloads,
    rectangles: Rectangles,
    pub focused: FocusedWidget,
    pub tab_bar: TabBar<'a>,
    pub key_bar: KeyBar<'a>,
    pub bottom_bar: BottomBar<'a>,
    pub archives_view: ArchiveTable<'a>,
    pub files_view: FileTable<'a>,
    pub downloads_view: DownloadTable<'a>,
    pub msgs_view: MessageList<'a>,
    redraw_terminal: Arc<AtomicBool>,
    updater: UpdateChecker,
    msgs: Messages,
}

impl<'a> MainUI<'static> {
    pub async fn new(
        cache: Cache,
        client: Client,
        config: Config,
        downloads: Downloads,
        msgs: Messages,
        archives: Archives,
    ) -> Self {
        let updater = UpdateChecker::new(cache.clone(), client.clone(), config, msgs.clone());

        let redraw_terminal = Arc::new(AtomicBool::new(true));

        let tab_bar = TabBar::new(redraw_terminal.clone());
        let key_bar = KeyBar::new();
        let bottom_bar = BottomBar::new(redraw_terminal.clone(), client.request_counter);
        let archives_view = ArchiveTable::new(redraw_terminal.clone(), archives);
        let files_view = FileTable::new(redraw_terminal.clone(), cache.file_index.clone());
        let downloads_view = DownloadTable::new(redraw_terminal.clone(), downloads.clone());
        let msgs_view = MessageList::new(redraw_terminal.clone(), msgs.clone()).await;

        let focused = FocusedWidget::FileTable;

        Self {
            cache,
            downloads,
            rectangles: Rectangles::new(),
            focused,
            tab_bar,
            key_bar,
            archives_view,
            files_view,
            downloads_view,
            msgs_view,
            bottom_bar,
            redraw_terminal,
            updater,
            msgs,
        }
    }

    /* This is the main UI loop.
     * Redrawing the terminal is CPU intensive - locks and atomics are used to ensure it's done only when necessary. */
    pub async fn run(mut self) {
        let mut events = Events::new();
        self.files_view.focus();
        // X11 (and maybe Wayland?) sends SIGWINCH when the window is resized
        let got_sigwinch = Arc::new(AtomicBool::new(false));
        let signals = Signals::new([SIGWINCH]).unwrap();
        let handle = signals.handle();
        let _sigwinch_task = task::spawn(handle_sigwinch(signals, got_sigwinch.clone()));
        let mut terminal = term_setup().unwrap();

        loop {
            {
                self.files_view.refresh().await;
                self.downloads_view.refresh().await;
                self.msgs_view.refresh().await;
                self.archives_view.refresh().await;
                self.key_bar.refresh().await;
                self.tab_bar.refresh().await;
                self.bottom_bar.refresh().await;

                let recalculate_rects = got_sigwinch.swap(false, Ordering::Relaxed);

                if self.redraw_terminal.swap(false, Ordering::Relaxed) || recalculate_rects {
                    terminal
                        .draw(|f| {
                            if recalculate_rects {
                                self.rectangles.recalculate(f.size());
                            }
                            if self.tab_bar.selected().unwrap() == 0 {
                                f.render_stateful_widget(
                                    self.files_view.widget.clone(),
                                    self.rectangles.rect_main[0],
                                    &mut self.files_view.state,
                                );
                                f.render_stateful_widget(
                                    self.downloads_view.widget.clone(),
                                    self.rectangles.rect_main[1],
                                    &mut self.downloads_view.state,
                                );
                                f.render_widget(self.bottom_bar.widget.clone(), self.rectangles.rect_botbar[1]);
                            } else if self.tab_bar.selected().unwrap() == 1 {
                                f.render_stateful_widget(
                                    self.archives_view.widget.clone(),
                                    self.rectangles.rect_tabbar[1],
                                    &mut self.archives_view.state,
                                );
                            }
                            f.render_stateful_widget(
                                self.msgs_view.widget.clone(),
                                self.rectangles.rect_root[1],
                                &mut self.msgs_view.state,
                            );

                            f.render_widget(self.key_bar.widget.clone(), self.rectangles.rect_keybar[0]);
                            f.render_widget(self.tab_bar.widget.clone(), self.rectangles.rect_tabbar[0]);
                        })
                        .unwrap();
                }
            }

            if let Some(Event::Input(key)) = events.next().await {
                if let Key::Char('q') | Key::Ctrl('c') = key {
                    handle.close();
                    return;
                } else {
                    self.handle_keypress(key).await;
                }
            }
        }
    }

    async fn handle_keypress(&mut self, key: Key) {
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
            // TODO abstract things like this away from the UI code
            Key::Char('i') => {
                if let FocusedWidget::FileTable = self.focused {
                    if let Some(i) = self.selected_index() {
                        self.updater.ignore_file(i).await;
                    }
                }
            }
            Key::Char('p') => {
                if let FocusedWidget::DownloadTable = self.focused {
                    if let Some(i) = self.selected_index() {
                        self.downloads.toggle_pause_for(i).await;
                    }
                }
            }
            Key::Char('U') => {
                if let FocusedWidget::FileTable = self.focused {
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
            }
            Key::Char('u') => {
                if let FocusedWidget::FileTable = self.focused {
                    self.updater.update_all().await;
                }
            }
            Key::Char('v') => {
                if let FocusedWidget::FileTable = self.focused {
                    if let Some(i) = self.selected_index() {
                        let files_lock = self.files_view.file_index.files_sorted.read().await;
                        let fdata = files_lock.get(i).unwrap();
                        let lf_lock = fdata.local_file.read().await;
                        let url = format!("https://www.nexusmods.com/{}/mods/{}", &lf_lock.game, &lf_lock.mod_id);
                        if Command::new("xdg-open").arg(url).status().is_err() {
                            self.msgs.push("xdg-open is needed to open URLs in browser.".to_string()).await;
                        }
                    }
                } else if let FocusedWidget::ArchiveTable = &self.focused {
                    if let Some(i) = self.selected_index() {
                        let path = self.archives_view.archives.files.get(i).unwrap().path();
                        self.archives_view.archives.list_contents(&path).await;
                    }
                }
            }
            Key::Delete => match self.focused {
                FocusedWidget::ArchiveTable => {
                    self.msgs.push("Not implemented.").await;
                }
                FocusedWidget::FileTable => {
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
                FocusedWidget::DownloadTable => {
                    if let Some(i) = self.selected_index() {
                        self.downloads_view.downloads.delete(i).await;
                        if i == 0 {
                            self.select_widget_index(None);
                        }
                        self.focus_previous();
                    }
                }
                FocusedWidget::MessageList => {
                    if let Some(i) = self.selected_index() {
                        self.msgs_view.msgs.remove(i).await;
                        if i == 0 {
                            self.select_widget_index(None);
                        }
                        self.focus_previous();
                    }
                }
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
}
