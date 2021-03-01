mod component;
mod event;

use crate::api::FileDetails;

use self::component::StatefulCollection;
use self::event::{Event, Events};
use tui::widgets::{Cell, ListState, TableState};

use crate::db::*;

use std::io;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Row, Table};
use tui::Terminal;

enum SelectedView {
    Files,
}

fn term_setup() -> Result<Terminal<impl Backend>, Box<dyn std::error::Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub fn init(game: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = term_setup().unwrap();

    let mut selected_view: SelectedView = SelectedView::Files;

    let events = Events::new();

    let cache = Cache::new(&game)?;

    let mut errors = StatefulCollection::list_with_items(vec!["Item0", "Item1", "Item2"]);
    let mut files = StatefulCollection::table_with_items(cache.file_details_map.values().collect());
    //let foo = format!("{:?}", files.items);
    //errors.items.append(&mut vec![&foo]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(50)])
        .margin(0);

    loop {
        terminal.draw(|f| {
            let blocks = layout.split(f.size());

            match selected_view {
                SelectedView::Files => {
                    let left_table = create_file_table(files.items.as_slice());
                    f.render_stateful_widget(
                        left_table,
                        blocks[0],
                        &mut files.state.as_table_state(),
                    );
                }
            }
            let error_list = create_error_list(&errors.items);
            f.render_stateful_widget(error_list, blocks[1], &mut errors.state.as_list_state());
        })?;

        if let Event::Input(key) = events.next()? {
            match key {
                Key::Char('q') => break,
                Key::Char('f') => {
                    selected_view = SelectedView::Files;
                    //errors.items.append(&mut vec!["foo"]);
                }
                Key::Down | Key::Char('j') => {
                    errors.next();
                }
                Key::Up | Key::Char('k') => {
                    errors.previous();
                }
                _ => {}
            }
        }
    }
    Ok(())
}

// TODO fetch the FileDetails when downloading a file. Hope the API has it in every case!
// TODO handle missing FileDetails and foreign (non-Nexusmods) mods
fn create_file_table<'a>(fdl: &[&FileDetails]) -> Table<'a> {
    let header = Row::new(
        vec!["Name", "Version"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
    );

    let rows: &Vec<Row> = &fdl
        .iter()
        .map(|x| {
            Row::new(vec![
                x.name.clone(),
                x.version.as_ref().unwrap_or(&"".to_string()).to_string(),
            ])
        })
        .collect();

    /* need: headers and fields. field data is optional except for file name. headers are (at least):
     * [file_name, mod_name, version]
     * This data isn't acquired when downloading, but is contained in either FileDetails and Md5FileDetails.
     * The required metadata doesn't really conveniently exist in any automagically acquired form, so we probably just
     * need to ask for it when downloading.
     */

    let table = Table::new(rows.clone())
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .widths(&[Constraint::Length(50), Constraint::Length(7)]);
    table
}

fn create_error_list<'a>(items: &'a Vec<&str>) -> List<'a> {
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|i| {
            let lines = vec![Spans::from(*i)];
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
