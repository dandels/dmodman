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
use tui::widgets::{List, Paragraph, Table};

enum FocusedWidget {
    DownloadTable,
    FileTable,
    Messages,
}

pub async fn run(cache: Cache, client: Client, config: Config, msgs: Messages) -> Result<(), Box<dyn Error>> {
    let mut focused = FocusedWidget::FileTable;
    // TODO use Tokio events?
    let events = Events::new();
    let updates = UpdateChecker::new(cache.clone(), client.clone(), config.clone(), msgs.clone());

    // TODO learn to use the constraints
    let topbar_layout: Layout =
        Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(1), Constraint::Percentage(99)]);

    let botbar_layout: Layout =
        Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(99), Constraint::Min(1)]);

    let tables_layout: Layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

    let main_vertical_layout: Layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)]);

    let topbar_text: Vec<Spans> = vec![Spans::from(vec![
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
    let got_sigwinch = Arc::new(AtomicBool::new(true));
    let signals = Signals::new(&[SIGWINCH])?;
    let handle = signals.handle();
    let _sigwinch_task = tokio::task::spawn(handle_sigwinch(signals, got_sigwinch.clone()));

    let mut terminal = term_setup().unwrap();
    let mut files_view = FileTable::new(&cache.file_index);
    let mut download_view = DownloadTable::new(client.downloads.clone());
    let mut msg_view = MessageList::new(msgs.clone());
    files_view.focus();

    let mut files_widget = files_view.create().await;
    let mut download_widget = download_view.create().await;
    let mut msg_widget = msg_view.create().await;
    let mut bottom_bar_widget: Paragraph =
        Paragraph::new(client.request_counter.format().await).alignment(Alignment::Right);

    loop {
        let mut redraw_files = false;
        let mut redraw_downloads = false;
        let mut redraw_msgs = false;
        let mut redraw_botbar = false;
        let mut redraw_topbar = false;

        if got_sigwinch.load(Ordering::Relaxed) {
            redraw_files = true;
            redraw_downloads = true;
            redraw_msgs = true;
            redraw_botbar = true;
            redraw_topbar = true;
            files_widget = files_view.create().await;
            download_widget = download_view.create().await;
            msg_widget = msg_view.create().await;
        } else {
            if cache.file_index.has_changed() {
                files_widget = files_view.create().await;
                redraw_files = true;
            }
            if client.downloads.has_changed() {
                download_widget = download_view.create().await;
                redraw_downloads = true;
            }
            if msgs.has_changed() {
                msg_widget = msg_view.create().await;
                redraw_msgs = true;
            }
            if client.request_counter.has_changed() {
                bottom_bar_widget = Paragraph::new(client.request_counter.format().await).alignment(Alignment::Right);
                redraw_botbar = true;
            }
        }

        terminal.draw(|f| {
            // TODO only recreate these after SIGWINCH
            let rect_root = main_vertical_layout.split(f.size());
            let rect_topbar = topbar_layout.split(rect_root[0]);
            let rect_main = tables_layout.split(rect_topbar[1]);
            let rect_botbar = botbar_layout.split(rect_root[1]);

            if redraw_files {
                f.render_stateful_widget(files_widget.clone(), rect_main[0], &mut files_view.state);
            }
            if redraw_downloads {
                f.render_stateful_widget(download_widget.clone(), rect_main[1], &mut download_view.state);
            }
            if redraw_msgs {
                f.render_stateful_widget(msg_widget.clone(), rect_root[1], &mut msg_view.state);
            }
            if redraw_topbar {
                f.render_widget(topbar.clone(), rect_topbar[0]);
            }
            if redraw_botbar {
                f.render_widget(bottom_bar_widget.clone(), rect_botbar[1]);
            }
        })?;

        // FIXME
        let selected: &mut dyn Select = match focused {
            FocusedWidget::DownloadTable => &mut download_view,
            FocusedWidget::Messages => &mut msg_view,
            FocusedWidget::FileTable => &mut files_view,
        };

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') | Key::Ctrl('c') => break,
                Key::Down | Key::Char('j') => selected.next(),
                Key::Up | Key::Char('k') => selected.previous(),
                Key::Left | Key::Char('h') => match focused {
                    FocusedWidget::Messages | FocusedWidget::DownloadTable => {
                        selected.unfocus();
                        focused = FocusedWidget::FileTable;
                        files_view.focus();
                    }
                    FocusedWidget::FileTable => {
                        selected.unfocus();
                        focused = FocusedWidget::Messages;
                        msg_view.focus();
                    }
                },
                Key::Right | Key::Char('l') => match focused {
                    FocusedWidget::Messages | FocusedWidget::FileTable => {
                        selected.unfocus();
                        focused = FocusedWidget::DownloadTable;
                        download_view.focus();
                    }
                    FocusedWidget::DownloadTable => {
                        selected.unfocus();
                        focused = FocusedWidget::Messages;
                        msg_view.focus();
                    }
                },
                Key::Char('u') => match focused {
                    FocusedWidget::FileTable => match files_view.state.selected() {
                        Some(_i) => {
                            updates.check_all().await?;
                            for (_mod_id, localfiles) in updates.updatable.read().await.iter() {
                                for lf in localfiles {
                                    msgs.push(format!("{} has an update", lf.file_name)).await;
                                }
                            }
                        }
                        None => {}
                    },
                    _ => {}
                },
                Key::Char('U') => match focused {
                    FocusedWidget::FileTable => match files_view.state.selected() {
                        Some(i) => {
                            if let Some((file_id, fd)) = files_view.files.get_index(i).await {
                                let lf = cache.local_files.get(file_id).await.unwrap();
                                if updates.check_file(lf.clone()).await {
                                    msgs.push(format!("{} has an update", lf.file_name)).await;
                                }
                            }
                        }
                        None => {}
                    },
                    _ => {}
                },
                Key::Delete => match focused {
                    FocusedWidget::FileTable => match files_view.state.selected() {
                        Some(i) => {
                            let (_file_id, _fd) = files_view.files.get_index(i).await.unwrap();
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {
                    // Uncomment to log keypresses
                    msgs.push(format!("{:?}", key)).await;
                }
            }
            got_sigwinch.store(true, Ordering::Relaxed);
        }
    }
    handle.close();
    Ok(())
}
