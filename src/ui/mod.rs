mod component;
mod event;

use self::component::State;
use self::event::{Event, Events};

use crate::api::FileDetails;
use crate::api::Client;
use crate::db::*;

use std::io;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Cell, List, ListItem, Row, Table};
use tui::Terminal;
use std::error::Error;
use std::sync::{Arc, RwLock};

enum ActiveBlock {
    Errors,
    Downloads,
    Files,
}

fn term_setup() -> Result<Terminal<impl Backend>, Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub async fn init(cache: &mut Cache, client: &Client) -> Result<(), Box<dyn Error>> {
    let mut terminal = term_setup().unwrap();

    let events = Events::new();

    let mut selected_view = ActiveBlock::Files;


    let files_headers = Row::new(
        vec!["Name", "Version"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
    );

    let downloads_headers = Row::new(
        vec!["Filename", " ", "%"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
    );

    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(50)])
        .margin(0);

    let tables_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(0);

    let errors = Arc::new(RwLock::new(Vec::new()));
    let mut errors_state = State::new_list();

    let mut files_state = State::new_table();
    let mut files_table = create_file_table(cache, &files_headers);

    let mut downloads_state = State::new_table();
    let mut downloads_table = create_downloads_table(client, &downloads_headers);
    // TODO implement check for this to save CPU
    let downloads_is_changed = true;

    loop {
        terminal.draw(|f| {
            let rect_root = root_layout.split(f.size());
            let rect_main = tables_layout.split(rect_root[0]);

            if cache.file_details.is_changed() {
                files_table = create_file_table(cache, &files_headers);
            }
            f.render_stateful_widget(files_table.clone(), rect_main[0], &mut files_state.state.as_table_state());

            if downloads_is_changed {
                downloads_table= create_downloads_table(client, &files_headers);
            }
            f.render_stateful_widget(downloads_table.clone(), rect_main[1], &mut downloads_state.state.as_table_state());


            //let downloads_table = create_downloads_table(downloads.items.read().unwrap().as_slice(), &downloads_headers);
            //f.render_stateful_widget(downloads_table, rect_main[1], &mut downloads.state.as_table_state());

            let error_list = create_error_list(errors.read().unwrap().as_slice());
            f.render_stateful_widget(error_list, rect_root[1], &mut errors_state.state.as_list_state());
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => break,
                Key::Char('f') => {
                    //errors.items.append(&mut vec!["foo"]);
                }
                Key::Char('e') => {
                    errors.write().unwrap().push("terribad error".to_string());
                }
                Key::Down | Key::Char('j') => match selected_view {
                    ActiveBlock::Downloads => downloads_state.next(client.downloads.read().unwrap().len()),
                    ActiveBlock::Errors => errors_state.next(errors.read().unwrap().len()),
                    ActiveBlock::Files => files_state.next(cache.file_details.len()),
                },
                Key::Up | Key::Char('k') => match selected_view {
                    ActiveBlock::Downloads => downloads_state.previous(errors.read().unwrap().len()),
                    ActiveBlock::Errors => errors_state.previous(errors.read().unwrap().len()),
                    ActiveBlock::Files => files_state.previous(cache.file_details.len()),
                },
                Key::Left | Key::Char('h') => match selected_view {
                    ActiveBlock::Errors | ActiveBlock::Downloads => selected_view = ActiveBlock::Files,
                    ActiveBlock::Files => selected_view = ActiveBlock::Errors,
                },
                Key::Right | Key::Char('l') => match selected_view {
                    ActiveBlock::Errors | ActiveBlock::Files => selected_view = ActiveBlock::Downloads,
                    ActiveBlock::Downloads => selected_view = ActiveBlock::Errors,
                },
                Key::Char('u') => {
                    if let ActiveBlock::Files = selected_view {
                        errors.write().unwrap().push("terribad error".to_string());
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

// TODO handle missing FileDetails and foreign (non-Nexusmods) mods
fn create_file_table<'a>(cache: &Cache, headers: &'a Row) -> Table<'a> {
    let map = cache.file_details.map.read().unwrap();
    let vals: Vec<&FileDetails> = map.values().collect();
    let rows: Vec<Row> = vals
        .iter()
        .map(|x| {
            Row::new(vec![
                x.name.clone(),
                x.version.as_ref().unwrap_or(&"".to_string()).to_string(),
            ])
        })
        .collect();

    let table = Table::new(rows.clone())
        .header(headers.clone())
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .widths(&[Constraint::Length(50), Constraint::Length(7)])
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    table
}

fn create_downloads_table<'a>(client: &Client, headers: &'a Row) -> Table<'a> {
    let rows: Vec<Row> = client.downloads.read().unwrap()
        .iter()
        .map(|x| {
            let x = x.read().unwrap();
            Row::new(vec![
                x.file_name.clone(),
                "".to_string(),
                x.progress()
            ])
        })
        .collect();

    let table = Table::new(rows.clone())
        .header(headers.clone())
        .block(Block::default().borders(Borders::ALL).title("Downloads"))
        .widths(&[Constraint::Length(50), Constraint::Length(1), Constraint::Length(4)])
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    table
}


fn create_error_list<'a>(items: &[String]) -> List<'a> {
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from(i.clone())];
            ListItem::new(lines).style(Style::default().fg(Color::Red))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let error_list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Errors"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );
    error_list
}
