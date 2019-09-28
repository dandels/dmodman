use std::io;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Row, Table, Widget};
use tui::Terminal;

use super::event::{Event, Events};
use termion::input::TermRead;

//struct View<'a> {
//    items: Vec<Vec<String>>,
//    selected: usize,
//    headers: Vec<&'a str>,
//}
//
//impl<'a> View<'a> {
//    fn new(headers: Vec<&'a str>, data: Vec<Vec<String>>) -> View<'a> {
//        View {
//            items: data,
//            headers: headers,
//            selected: 0,
//        }
//    }
//}

pub fn init<'a>(headers: Vec<&'a str>, items: &'a Vec<Vec<String>>) -> Result<(), failure::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();
    //let view = View::new(headers, rows);
    let mut selected = 0;

    let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    loop {
        terminal.draw(|mut f| {
            let rows = items.iter().enumerate().map(|(i, item)| {
                if i == selected {
                    Row::StyledData(item.into_iter(), selected_style)
                } else {
                    Row::StyledData(item.into_iter(), normal_style)
                }
            });

            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(1)
                .split(f.size());
            Table::new(headers.clone().into_iter(), rows.clone())
                .block(Block::default().borders(Borders::ALL).title("Files"))
                .widths(&[30, 7, 15, 10])
                .render(&mut f, layout[0]);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => break,
                Key::Down | Key::Char('j') => {
                    selected += 1;
                    if selected > items.len() - 1 {
                        selected = 0;
                    }
                }
                Key::Up | Key::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    } else {
                        selected = items.len() - 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
