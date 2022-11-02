use super::component::*;
use super::event::{Event, Events};

use crate::api::Client;
use crate::api::UpdateChecker;
use crate::cache::Cache;
use crate::config::Config;
use crate::ui::*;
use crate::Messages;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use termion;
use termion::event::Key;
use tokio::sync::RwLock;
use tokio::task;
use tui::layout::{Constraint, Direction, Layout, Rect};

pub struct UI<'a> {
    focused: FocusedWidget<'a>,
    download_view: Arc<RwLock<DownloadTable<'a>>>,
    files_view: Arc<RwLock<FileTable<'a>>>,
    msg_view: Arc<RwLock<MessageList<'a>>>,
    top_bar: Arc<RwLock<TopBar<'a>>>,
    bottom_bar: Arc<RwLock<BottomBar<'a>>>,
    events: Events,
    updater: UpdateChecker,
    cache: Cache,
    client: Client,
    msgs: Messages,
}

impl<'a> UI<'static> {
    pub fn new(cache: Cache, client: Client, config: Config, msgs: Messages) -> Self {
        // TODO use Tokio events?
        let events = Events::new();
        let updater = UpdateChecker::new(cache.clone(), client.clone(), config.clone(), msgs.clone());

        let top_bar = RwLock::new(TopBar::new()).into();
        let files_view = Arc::new(RwLock::new(FileTable::new(&cache.file_index)));
        let download_view = RwLock::new(DownloadTable::new(client.downloads.clone())).into();
        let msg_view = RwLock::new(MessageList::new(msgs.clone())).into();
        let bottom_bar = RwLock::new(BottomBar::new(client.request_counter.clone())).into();

        let focused = FocusedWidget::FileTable(files_view.clone());

        Self {
            focused,
            files_view,
            download_view,
            msg_view,
            bottom_bar,
            top_bar,
            updater,
            cache,
            client,
            msgs,
            events,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.files_view.write().await.focus().await;
        /* X11 (and maybe Wayland?) sends SIGWINCH when the window is resized, so we can listen to that. Otherwise we
         * redraw when something has changed.
         * We set this to true so that all widgets are rendered in the first loop. */
        let got_sigwinch = Arc::new(AtomicBool::new(true));
        let signals = Signals::new(&[SIGWINCH])?;
        let handle = signals.handle();
        let _sigwinch_task = task::spawn(handle_sigwinch(signals, got_sigwinch.clone()));

        // TODO learn to use the constraints
        let topbar_layout: Layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Percentage(99)]);

        let botbar_layout: Layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(99), Constraint::Min(1)]);

        let tables_layout: Layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let main_vertical_layout: Layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)]);

        let mut terminal = term_setup().unwrap();

        let (width, height) = termion::terminal_size()?;
        let mut rect_root = main_vertical_layout.split(Rect {
            x: 0,
            y: 0,
            height,
            width,
        });
        let mut rect_topbar = topbar_layout.split(rect_root[0]);
        let mut rect_main = tables_layout.split(rect_topbar[1]);
        let mut rect_botbar = botbar_layout.split(rect_root[1]);

        let mut needs_redraw = false;
        loop {
            if got_sigwinch.swap(false, Ordering::Relaxed) {
                self.msgs.push("redraw everything").await;
                self.files_view.write().await.refresh().await;
                self.download_view.write().await.refresh().await;
                self.msg_view.write().await.refresh().await;
                self.top_bar.write().await.refresh().await;
                self.bottom_bar.write().await.refresh().await;
                let (width, height) = termion::terminal_size()?;
                rect_root = main_vertical_layout.split(Rect {
                    x: 0,
                    y: 0,
                    height,
                    width,
                });
                rect_topbar = topbar_layout.split(rect_root[0]);
                rect_main = tables_layout.split(rect_topbar[1]);
                rect_botbar = botbar_layout.split(rect_root[1]);
                needs_redraw = true;
            } else {
                if self.cache.file_index.has_changed.swap(false, Ordering::Relaxed) {
                    self.files_view.write().await.refresh().await;
                    needs_redraw = true;
                    self.msgs.push("redraw files").await;
                }
                // TODO make sure we don't redraw too often during downloads
                if self.client.downloads.has_changed.swap(false, Ordering::Relaxed) {
                    self.download_view.write().await.refresh().await;
                    needs_redraw = true;
                    self.msgs.push("redraw downloads").await;
                }
                if self.msgs.has_changed.swap(false, Ordering::Relaxed) {
                    self.msg_view.write().await.refresh().await;
                    needs_redraw = true;
                }
                if self.client.request_counter.has_changed.swap(false, Ordering::Relaxed) {
                    self.bottom_bar.write().await.refresh().await;
                    needs_redraw = true;
                    self.msgs.push("redraw counter").await;
                }
            }
            if needs_redraw {
                needs_redraw = false;
                let mut files = self.files_view.write().await;
                let mut downloads = self.download_view.write().await;
                let mut msgs = self.msg_view.write().await;
                let topbar = self.top_bar.read().await;
                let botbar = self.bottom_bar.read().await;
                terminal.draw(|f| {
                    f.render_stateful_widget(files.widget.clone(), rect_main[0], &mut files.state);
                    f.render_stateful_widget(downloads.widget.clone(), rect_main[1], &mut downloads.state);
                    f.render_stateful_widget(msgs.widget.clone(), rect_root[1], &mut msgs.state);
                    f.render_widget(topbar.widget.clone(), rect_topbar[0]);
                    f.render_widget(botbar.widget.clone(), rect_botbar[1]);
                })?;
            }

            if let Event::Input(key) = self.events.next()? {
                match key {
                    Key::Char('q') | Key::Ctrl('c') => {
                        handle.close();
                        return Ok(());
                    }
                    Key::Down | Key::Char('j') => {
                        self.focused.next().await;
                        needs_redraw = true;
                    }
                    Key::Up | Key::Char('k') => {
                        self.focused.previous().await;
                        needs_redraw = true;
                    }
                    Key::Left | Key::Char('h') => match self.focused {
                        FocusedWidget::MessageList(_) | FocusedWidget::DownloadTable(_) => {
                            self.focused.change_to(FocusedWidget::FileTable(self.files_view.clone())).await;
                            needs_redraw = true;
                        }
                        FocusedWidget::FileTable(_) => {
                            self.focused.change_to(FocusedWidget::MessageList(self.msg_view.clone())).await;
                            needs_redraw = true;
                        }
                    },
                    Key::Right | Key::Char('l') => match self.focused {
                        FocusedWidget::MessageList(_) | FocusedWidget::FileTable(_) => {
                            self.focused.change_to(FocusedWidget::DownloadTable(self.download_view.clone())).await;
                            needs_redraw = true;
                        }
                        FocusedWidget::DownloadTable(_) => {
                            self.focused.change_to(FocusedWidget::MessageList(self.msg_view.clone())).await;
                            needs_redraw = true;
                        }
                    },
                    Key::Char('U') => {
                        let ftable = self.files_view.read().await;
                        match ftable.state.selected() {
                            Some(i) => {
                                if let Some((file_id, _fd)) = ftable.files.get_index(i).await {
                                    let lf = self.cache.local_files.get(file_id).await.unwrap();
                                    if self.updater.check_file(lf.clone()).await {
                                        self.msgs.push(format!("{} has an update", lf.file_name)).await;
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    Key::Char('u') => match self.focused.clone() {
                        FocusedWidget::FileTable(_fv) => {
                            let ftable = self.files_view.read().await;
                            match ftable.state.selected() {
                                Some(_i) => {
                                    let updater = self.updater.clone();
                                    let msgs = self.msgs.clone();
                                    task::spawn(async move {
                                        updater.check_all().await;
                                        for (_mod_id, localfiles) in updater.updatable.read().await.iter() {
                                            for lf in localfiles {
                                                msgs.push(format!("{} has an update", lf.file_name)).await;
                                            }
                                        }
                                    });
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    },
                    Key::Delete => match self.focused.clone() {
                        FocusedWidget::FileTable(_ft) => {
                            let ftable = self.files_view.read().await;
                            match ftable.state.selected() {
                                Some(i) => {
                                    let (_file_id, _fd) = ftable.files.get_index(i).await.unwrap();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    _ => {
                        // Uncomment to log keypresses
                        self.msgs.push(format!("{:?}", key)).await;
                    }
                }
            }
        }
    }
}
