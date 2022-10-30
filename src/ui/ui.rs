use super::component::Select;
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
use termion::event::Key;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::Paragraph;

enum FocusedWidget {
    DownloadTable,
    FileTable,
    Messages,
}

#[allow(dead_code)]
pub struct UI<'a> {
    cache: Cache,
    client: Client,
    config: Config,
    msgs: Messages,
    files_view: FileTable<'a>,
    download_view: DownloadTable<'a>,
    msg_view: MessageList<'a>,
    events: Events,
    focused: FocusedWidget,
    updates: UpdateChecker,
    botbar: Paragraph<'a>,
}

impl<'a> UI<'static> {
    pub fn init(cache: Cache, client: Client, config: Config, msgs: Messages) -> Result<Self, Box<dyn Error>> {
        let mut ret = Self {
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
            download_view: DownloadTable::new(client.clone().downloads),
            files_view: FileTable::new(&cache.file_index),
            msg_view: MessageList::new(msgs.clone()),
            events: Events::new(),
            focused: FocusedWidget::FileTable,
            updates: UpdateChecker::new(cache.clone(), client.clone(), config.clone(), msgs.clone()),
            botbar: Paragraph::new(client.request_counter.format()).alignment(Alignment::Right),
        };
        ret.files_view.focus();
        Ok(ret)
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = term_setup().unwrap();

        let topbar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Percentage(100)]);

        let botbar_layout =
            // TODO learn to use the constraints
            Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(1), Constraint::Max(1)]);

        let tables_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let main_vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)]);

        let topbar_text = vec![Spans::from(vec![
            Span::styled("<q>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("quit,"),
            Span::styled(" <u>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("update all"),
            Span::styled(" <U>", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("update selected,"),
        ])];

        let topbar = Paragraph::new(topbar_text);

        /* X11 (and maybe Wayland?) sends SIGWINCH when the window is resized, so we can listen to that. Otherwise we
         * redraw when something has changed
         */
        let needs_redraw = Arc::new(AtomicBool::new(true));
        let signals = Signals::new(&[SIGWINCH])?;
        let handle = signals.handle();
        let _sigwinch_task = tokio::task::spawn(handle_sigwinch(signals, needs_redraw.clone()));

        loop {
            if needs_redraw.load(Ordering::Relaxed) {
                needs_redraw.store(false, Ordering::Relaxed);
                terminal.draw(|f| {
                    let rect_root = main_vertical_layout.split(f.size());
                    let rect_topbar = topbar_layout.split(rect_root[0]);
                    let rect_main = tables_layout.split(rect_topbar[1]);
                    let rect_botbar = botbar_layout.split(rect_root[1]);

                    f.render_stateful_widget(self.files_view.widget.clone(), rect_main[0], &mut self.files_view.state);
                    f.render_stateful_widget(
                        self.download_view.widget.clone(),
                        rect_main[1],
                        &mut self.download_view.state,
                    );
                    f.render_stateful_widget(self.msg_view.widget.clone(), rect_root[1], &mut self.msg_view.state);
                    f.render_widget(topbar.clone(), rect_topbar[0]);
                    f.render_widget(self.botbar.clone(), rect_botbar[1]);
                })?;
            }

            // TODO doing this in the loop is wasteful, but otherwise it causes lifetime issues
            let selected: &mut dyn Select = match self.focused {
                FocusedWidget::DownloadTable => &mut self.download_view,
                FocusedWidget::Messages => &mut self.msg_view,
                FocusedWidget::FileTable => &mut self.files_view,
            };

            if let Event::Input(key) = self.events.next()? {
                match key {
                    Key::Char('q') | Key::Ctrl('c') => break,
                    Key::Down | Key::Char('j') => selected.next(),
                    Key::Up | Key::Char('k') => selected.previous(),
                    Key::Left | Key::Char('h') => match self.focused {
                        FocusedWidget::Messages | FocusedWidget::DownloadTable => {
                            selected.unfocus();
                            self.focused = FocusedWidget::FileTable;
                            self.files_view.focus();
                        }
                        FocusedWidget::FileTable => {
                            selected.unfocus();
                            self.focused = FocusedWidget::Messages;
                            self.msg_view.focus();
                        }
                    },
                    Key::Right | Key::Char('l') => match self.focused {
                        FocusedWidget::Messages | FocusedWidget::FileTable => {
                            selected.unfocus();
                            self.focused = FocusedWidget::DownloadTable;
                            self.download_view.focus();
                        }
                        FocusedWidget::DownloadTable => {
                            selected.unfocus();
                            self.focused = FocusedWidget::Messages;
                            self.msg_view.focus();
                        }
                    },
                    Key::Char('u') => match self.focused {
                        FocusedWidget::FileTable => match self.files_view.state.selected() {
                            Some(_i) => {
                                self.updates.check_all().await?;
                                for (_mod_id, localfiles) in self.updates.updatable.read().unwrap().iter() {
                                    for lf in localfiles {
                                        self.msgs.push(format!("{} has an update", lf.file_name));
                                    }
                                }
                            }
                            None => {}
                        },
                        _ => {}
                    },
                    Key::Char('U') => match self.focused {
                        FocusedWidget::FileTable => match self.files_view.state.selected() {
                            Some(i) => {
                                let (file_id, _lf) = self.files_view.files.get_index(i).unwrap();
                                let lf = self.cache.local_files.get(file_id).unwrap();
                                self.updates.check_mod(&lf.game, lf.mod_id, vec![lf.clone()]).await.unwrap();
                                for (_mod_id, localfiles) in self.updates.updatable.read().unwrap().iter() {
                                    for lf in localfiles {
                                        self.msgs.push(format!("{} has an update", lf.file_name));
                                    }
                                }
                            }
                            None => {}
                        },
                        _ => {}
                    },
                    Key::Delete => match self.focused {
                        FocusedWidget::FileTable => match self.files_view.state.selected() {
                            Some(i) => {
                                let (file_id, _fd) = self.files_view.files.get_index(i).unwrap();
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {
                        // Uncomment to log keypresses
                        self.msgs.push(format!("{:?}", key));
                    }
                }
                needs_redraw.store(true, Ordering::Relaxed);
            }
            self.refresh_widgets(needs_redraw.clone());
        }
        handle.close();
        Ok(())
    }

    fn refresh_widgets(&mut self, needs_redraw: Arc<AtomicBool>) {
        if self.files_view.files.has_changed() {
            self.files_view.refresh();
            needs_redraw.store(true, Ordering::Relaxed);
            self.msgs.push("files changed");
        }

        if self.download_view.downloads.has_changed() {
            self.download_view.refresh();
            needs_redraw.store(true, Ordering::Relaxed);
            self.msgs.push("downloads changed");
        }
        if self.msg_view.msgs.has_changed() {
            self.msg_view.refresh();
            needs_redraw.store(true, Ordering::Relaxed);
        }

        if self.client.request_counter.has_changed() {
            self.msgs.push("I'M HERE");
            self.botbar = Paragraph::new(self.client.request_counter.format()).alignment(Alignment::Right)
        }
    }
}
