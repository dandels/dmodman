mod component;
mod event;

use self::component::Select;
use self::component::*;
use self::event::{Event, Events};

use crate::api::Client;
use crate::cache::FileDetailsCache;
use crate::cache::UpdateChecker;
use crate::Messages;

use std::error::Error;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

enum ActiveWidget {
    Downloads,
    Files,
    Messages,
}

pub async fn init(files: &FileDetailsCache, client: &Client, msgs: &Messages) -> Result<(), Box<dyn Error>> {
    let mut terminal = term_setup().unwrap();
    let events = Events::new();
    let mut msglist = MessageList::new(msgs);
    let mut files = FileTable::new(files);
    let mut downloads = DownloadTable::new(&client.downloads);

    let mut active = ActiveWidget::Files;
    let updates = UpdateChecker::new(client.clone());

    files.focus();

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

            if files.is_changed() {
                files.refresh();
            }

            if downloads.is_changed() {
                downloads.refresh();
            }
            if msglist.is_changed() {
                msglist.refresh();
            }

            f.render_stateful_widget(files.widget.clone(), rect_main[0], &mut files.state);

            f.render_stateful_widget(downloads.widget.clone(), rect_main[1], &mut downloads.state);

            f.render_stateful_widget(msglist.widget.clone(), rect_root[1], &mut msglist.state);
        })?;

        let selected: &mut dyn Select = match active {
            ActiveWidget::Downloads => &mut downloads,
            ActiveWidget::Messages => &mut msglist,
            ActiveWidget::Files => &mut files,
        };

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => break,
                Key::Char('e') => msgs.push("terribad error".to_string()),
                Key::Down | Key::Char('j') => selected.next(),
                Key::Up | Key::Char('k') => selected.previous(),
                Key::Left | Key::Char('h') => match active {
                    ActiveWidget::Messages | ActiveWidget::Downloads => {
                        selected.unfocus();
                        active = ActiveWidget::Files;
                        files.focus();
                    }
                    ActiveWidget::Files => {
                        selected.unfocus();
                        active = ActiveWidget::Messages;
                        msglist.focus();
                    }
                },
                Key::Right | Key::Char('l') => match active {
                    ActiveWidget::Messages | ActiveWidget::Files => {
                        selected.unfocus();
                        active = ActiveWidget::Downloads;
                        downloads.focus();
                    }
                    ActiveWidget::Downloads => {
                        selected.unfocus();
                        active = ActiveWidget::Messages;
                        msglist.focus();
                    }
                },
                Key::Char('\n') => match active {
                    ActiveWidget::Files => match files.state.selected() {
                        Some(i) => {
                            // TODO only update the selected file
                            let (_file_id, _fd) = files.files.get_index(i).unwrap();
                            updates.check_all().await?;
                            for (_mod_id, localfiles) in updates.updatable.read().unwrap().iter() {
                                for lf in localfiles {
                                    msgs.push(format!("{} has an update", lf.file_name));
                                }
                            }
                        }
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

fn term_setup() -> Result<Terminal<impl Backend>, Box<dyn Error>> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}
