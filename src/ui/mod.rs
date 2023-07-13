mod component;
mod event;
mod main_ui;
mod rectangles;
pub mod sso;

pub use main_ui::MainUI;

//use self::event::{Event, Events};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use signal_hook_tokio::Signals;
//use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tokio_stream::StreamExt;
//use ratatui::layout::{Constraint, Direction, Layout};

use ratatui::backend::{Backend, TermionBackend};
use ratatui::Terminal;

pub fn term_setup() -> Result<Terminal<impl Backend>, Box<dyn Error>> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub async fn handle_sigwinch(mut signals: Signals, is_window_resized: Arc<AtomicBool>) {
    while signals.next().await.is_some() {
        is_window_resized.store(true, Ordering::Relaxed);
    }
}

//pub fn pick_from_list<'a>(title: String, list: Vec<String>) -> Result<Option<String>, Box<dyn Error>> {
//    let mut terminal = term_setup().unwrap();
//    let mut lp = ListPicker::new(title, list);
//    let needs_redraw = Arc::new(AtomicBool::new(true));
//    let root_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(100)]);
//    let events = Events::new();
//
//    loop {
//        if needs_redraw.load(Ordering::Relaxed) {
//            terminal.draw(|f| {
//                let rect_root = root_layout.split(f.size());
//
//                f.render_stateful_widget(lp.widget.clone(), rect_root[0], &mut lp.state);
//            })?;
//            needs_redraw.store(false, Ordering::Relaxed);
//        }
//
//        if let Event::Input(key) = events.next()? {
//            match key {
//                Key::Char('q') | Key::Ctrl('c') => return Ok(None),
//                Key::Char('\n') => match lp.state.selected() {
//                    Some(i) => {
//                        return Ok(match lp.list.get(i) {
//                            Some(game) => Some(game.to_string()),
//                            None => None,
//                        })
//                    }
//                    None => return Ok(None),
//                },
//                _ => continue,
//            }
//        }
//    }
//}
