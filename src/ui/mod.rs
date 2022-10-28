mod component;
mod event;

use self::component::Select;
use self::component::*;
use self::event::{Event, Events};

use crate::api::Client;
use crate::api::UpdateChecker;
use crate::cache::Cache;
use crate::cache::LocalFile;
use crate::config::Config;
use crate::Messages;

use std::error::Error;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

enum FocusedWidget {
    DownloadTable,
    FileTable,
    Messages,
}

pub struct UI<'a> {
    events: Events,
    focused: FocusedWidget,
    files_view: FileTable<'a>,
    download_view: DownloadTable<'a>,
    msg_view: MessageList<'a>,
    updates: UpdateChecker,
    cache: Cache,
    client: Client,
    config: Config,
    msgs: Messages,
}

impl<'a> UI<'static> {
    pub async fn init(cache: Cache, client: Client, config: Config, msgs: Messages) -> Result<Self, Box<dyn Error>> {
        let events = Events::new();
        let msg_view = MessageList::new(msgs.clone());
        let mut files_view = FileTable::new(cache.clone().file_details);
        let download_view = DownloadTable::new(client.clone().downloads);

        let active = FocusedWidget::FileTable;
        let updates = UpdateChecker::new(client.clone(), config.clone());

        files_view.focus();
        Ok(Self {
            events,
            msg_view,
            updates,
            download_view,
            files_view,
            focused: FocusedWidget::FileTable,
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone()
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut terminal = term_setup().unwrap();

        let root_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .margin(0);

        let tables_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .margin(0);

        loop {
            terminal.draw(|f| {
                let rect_root = root_layout.split(f.size());
                let rect_main = tables_layout.split(rect_root[0]);

                self.refresh_widgets();

                f.render_stateful_widget(self.files_view.widget.clone(), rect_main[0], &mut self.files_view.state);

                f.render_stateful_widget(self.download_view.widget.clone(), rect_main[1], &mut self.download_view.state);

                f.render_stateful_widget(self.msg_view.widget.clone(), rect_root[1], &mut self.msg_view.state);
            })?;

            let selected: &mut dyn Select = match self.focused {
                FocusedWidget::DownloadTable => &mut self.download_view,
                FocusedWidget::Messages => &mut self.msg_view,
                FocusedWidget::FileTable => &mut self.files_view,
            };

            if let Event::Input(key) = self.events.next()? {
                match key {
                    Key::Char('q') => break,
                    Key::Char('e') => self.msgs.push("terribad error".to_string()),
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
                            Some(i) => {
                                let (file_id, _fd) = self.files_view.files.get_index(i).unwrap();
                                let lf: LocalFile = self.cache.local_files.read().unwrap().iter().find(|x| x.file_id == file_id).unwrap().clone();
                                self.updates.check_mod(&lf.game, lf.mod_id, vec!(lf.clone())).await.unwrap();
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
                    Key::Char('\n') => match self.focused {
                        FocusedWidget::FileTable => match self.files_view.state.selected() {
                            Some(_i) => {
                                self.updates.check_all().await?;
                                for (_mod_id, localfiles) in self.updates.updatable.read().unwrap().iter() {
                                    for lf in localfiles {
                                        self.msgs.push(format!("{} has an update", lf.file_name));
                                    }
                                }
                            },
                            None => {}
                        },
                        _ => {}
                    },
                    _ => {
                        // Uncomment to log keypresses
                        //msgs.messages.push(format!("{:?}", key));
                    }
                }
            }
        }
        Ok(())
    }

    fn refresh_widgets(&mut self) {
        if self.files_view.is_changed() {
            self.files_view.refresh();
        }

        if self.download_view.is_changed() {
            self.download_view.refresh();
        }
        if self.msg_view.is_changed() {
            self.msg_view.refresh();
        }
    }
}

fn term_setup() -> Result<Terminal<impl Backend>, Box<dyn Error>> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}
