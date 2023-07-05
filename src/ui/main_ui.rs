use super::component::*;
use super::event::{Event, Events};
use crate::ui::rectangles::Rectangles;

use crate::api::{Client, Downloads, UpdateChecker};
use crate::cache::Cache;
use crate::config::Config;
use crate::ui::*;
use crate::Messages;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use termion::event::Key;
use tokio::sync::RwLock;
use tokio::task;

pub struct MainUI<'a> {
    cache: Cache,
    downloads: Downloads,
    events: Events,
    rectangles: Rectangles,
    focused: FocusedWidget<'a>,
    top_bar: Arc<RwLock<TopBar<'a>>>,
    files_view: Arc<RwLock<FileTable<'a>>>,
    download_view: Arc<RwLock<DownloadTable<'a>>>,
    msg_view: Arc<RwLock<MessageList<'a>>>,
    bottom_bar: Arc<RwLock<BottomBar<'a>>>,
    redraw_terminal: Arc<AtomicBool>,
    updater: UpdateChecker,
    msgs: Messages,
}

impl<'a> MainUI<'static> {
    pub fn new(cache: Cache, client: Client, config: Config, downloads: Downloads, msgs: Messages) -> Self {
        // TODO use Tokio events?
        let events = Events::new();
        let updater = UpdateChecker::new(cache.clone(), client.clone(), config, msgs.clone());

        let top_bar = RwLock::new(TopBar::new()).into();

        let redraw_terminal = Arc::new(AtomicBool::new(true));

        let files_view = Arc::new(RwLock::new(FileTable::new(
            redraw_terminal.clone(),
            cache.file_index.clone(),
        )));
        let download_view = RwLock::new(DownloadTable::new(redraw_terminal.clone(), downloads.clone())).into();
        let msg_view = RwLock::new(MessageList::new(redraw_terminal.clone(), msgs.clone())).into();
        let bottom_bar = RwLock::new(BottomBar::new(redraw_terminal.clone(), client.request_counter)).into();

        let focused = FocusedWidget::FileTable(files_view.clone());

        Self {
            cache,
            downloads,
            events,
            rectangles: Rectangles::new(),
            focused,
            top_bar,
            files_view,
            download_view,
            msg_view,
            bottom_bar,
            redraw_terminal,
            updater,
            msgs,
        }
    }

    /* This is the main UI loop.
     * Redrawing the terminal is quite CPU intensive, so we use a bunch of locks and atomics to make sure it only
     * happens when necessary. */
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.files_view.write().await.focus().await;
        /* X11 (and maybe Wayland?) sends SIGWINCH when the window is resized, so we can listen to that. Otherwise we
         * redraw when something has changed.
         * We set this to true so that all widgets are rendered in the first loop. */
        let got_sigwinch = Arc::new(AtomicBool::new(true));
        let signals = Signals::new([SIGWINCH])?;
        let handle = signals.handle();
        let _sigwinch_task = task::spawn(handle_sigwinch(signals, got_sigwinch.clone()));
        let mut terminal = term_setup().unwrap();

        loop {
            if got_sigwinch.swap(false, Ordering::Relaxed) {
                self.rectangles.recalculate();
                self.redraw_terminal.store(true, Ordering::Relaxed);
            }
            self.files_view.write().await.refresh().await;
            // TODO make sure we don't redraw too often during downloads
            self.download_view.write().await.refresh().await;
            self.msg_view.write().await.refresh().await;
            self.top_bar.write().await.refresh().await;
            self.bottom_bar.write().await.refresh().await;
            if self.redraw_terminal.swap(false, Ordering::Relaxed) {
                let mut files_view = self.files_view.write().await;
                let mut downloads_view = self.download_view.write().await;
                let mut msgs_view = self.msg_view.write().await;
                let topbar = self.top_bar.read().await;
                let botbar = self.bottom_bar.read().await;
                // TODO should this be done in a blocking thread?
                terminal.draw(|f| {
                    f.render_stateful_widget(
                        files_view.widget.clone(),
                        self.rectangles.rect_main[0],
                        &mut files_view.state,
                    );
                    f.render_stateful_widget(
                        downloads_view.widget.clone(),
                        self.rectangles.rect_main[1],
                        &mut downloads_view.state,
                    );
                    f.render_stateful_widget(
                        msgs_view.widget.clone(),
                        self.rectangles.rect_root[1],
                        &mut msgs_view.state,
                    );
                    f.render_widget(topbar.widget.clone(), self.rectangles.rect_topbar[0]);
                    f.render_widget(botbar.widget.clone(), self.rectangles.rect_botbar[1]);
                })?;
            }

            if let Ok(Event::Input(key)) = self.events.next() {
                if let Key::Char('q') | Key::Ctrl('c') = key {
                    handle.close();
                    return Ok(());
                } else {
                    self.handle_keypress(key).await;
                }
            }
        }
    }

    async fn handle_keypress(&mut self, key: Key) {
        match key {
            Key::Char('q') | Key::Ctrl('c') => {
                //handle.close();
                //return Ok(());
            }
            Key::Down | Key::Char('j') => {
                self.focused.next().await;
            }
            Key::Up | Key::Char('k') => {
                self.focused.previous().await;
            }
            Key::Left | Key::Char('h') => match self.focused {
                FocusedWidget::MessageList(_) | FocusedWidget::DownloadTable(_) => {
                    self.focused.change_to(FocusedWidget::FileTable(self.files_view.clone())).await;
                }
                FocusedWidget::FileTable(_) => {
                    self.focused.change_to(FocusedWidget::MessageList(self.msg_view.clone())).await;
                }
            },
            Key::Right | Key::Char('l') => match self.focused {
                FocusedWidget::MessageList(_) | FocusedWidget::FileTable(_) => {
                    self.focused.change_to(FocusedWidget::DownloadTable(self.download_view.clone())).await;
                }
                FocusedWidget::DownloadTable(_) => {
                    self.focused.change_to(FocusedWidget::MessageList(self.msg_view.clone())).await;
                }
            },
            Key::Char('p') => {
                if let FocusedWidget::DownloadTable(_) = &self.focused {
                    let dls_table = self.download_view.read().await;
                    if let Some(i) = dls_table.state.selected() {
                        self.downloads.toggle_pause_for(i).await;
                    }
                }
            }
            Key::Char('U') => {
                let ftable = self.files_view.read().await;
                if let Some(i) = ftable.state.selected() {
                    let files = ftable.file_index.files.read().await;
                    let (_file_id, fdata) = files.get_index(i).unwrap();
                    let lf_lock = fdata.local_file.read().await;
                    let file_list = self.cache.file_lists.get((&lf_lock.game, lf_lock.mod_id)).await.unwrap();
                    let files_by_mod = self.cache.file_index.mod_file_mapping.read().await;
                    let modfiles = files_by_mod.get(&(lf_lock.game.clone(), lf_lock.mod_id)).unwrap();
                    self.updater.check_mod(modfiles, &file_list).await;
                }
            }
            Key::Char('u') => {
                if let FocusedWidget::FileTable(_fv) = &self.focused {
                    let updater = self.updater.clone();
                    task::spawn(async move {
                        let _res = updater.update_all().await;
                    });
                }
            }
            Key::Delete => {
                if let FocusedWidget::FileTable(_ft) = &self.focused {
                    let ftable = self.files_view.read().await;
                    if let Some(_i) = ftable.state.selected() {
                        // TODO implement deletion
                    }
                }
            }
            _ => {
                // Uncomment to log keypresses
                self.msgs.push(format!("{:?}", key)).await;
            }
        }
    }
}
