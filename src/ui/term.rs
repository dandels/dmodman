use std::io;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Row, Table};
use tui::Terminal;

use super::event::{Event, Events};
use super::table::StatefulTable;

/* Planned windows:
 * - Downloaded mods (with filters for installed, uninstalled and sorting)
 * - Enabled plugins (for Skyrim, Morrowind, etc)
 * - Downloads
 * - Errors
 * - Files of mod
 */

pub fn init(
    headers: Vec<String>,
    items: Vec<Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::BOLD);
    let normal_style = Style::default().fg(Color::White);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();
    let mut table = StatefulTable::new(headers, items);

    loop {
        terminal.draw(|mut f| {
            let rows = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.iter(), normal_style));

            let rect_main = Layout::default()
                .direction(Direction::Horizontal)
                // The second value doesn't seem to matter, but it has to exist to split the screen
                .constraints([Constraint::Percentage(50), Constraint::Percentage(0)])
                .margin(0)
                .split(f.size());

            let rect_left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .margin(0)
                .split(rect_main[0]);

            let rect_right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .margin(0)
                .split(rect_main[1]);

            let table_files = Table::new(table.headers.iter(), rows)
                .block(Block::default().borders(Borders::ALL).title("Files"))
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    /* Width of fields in the table.
                     * These magic numbers match the table headers & values nicely.
                     */
                    Constraint::Length(30),
                    Constraint::Length(7),
                    Constraint::Length(15),
                    Constraint::Length(10),
                ]);

            let rows_foo = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.iter(), normal_style));

            let table_foo = Table::new(table.headers.iter(), rows_foo)
                .block(Block::default().borders(Borders::ALL).title("Table Foo"))
                .widths(&[
                    Constraint::Length(30),
                    Constraint::Length(7),
                    Constraint::Length(15),
                    Constraint::Length(10),
                ]);

            let rows_2 = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.iter(), normal_style));

            let table_2 = Table::new(table.headers.iter(), rows_2)
                .block(Block::default().borders(Borders::ALL).title("Table 2"))
                .widths(&[
                    Constraint::Length(30),
                    Constraint::Length(7),
                    Constraint::Length(15),
                    Constraint::Length(10),
                ]);

            let rows_3 = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.iter(), normal_style));

            let table_3 = Table::new(table.headers.iter(), rows_3)
                .block(Block::default().borders(Borders::ALL).title("Table 3"))
                .widths(&[
                    Constraint::Length(30),
                    Constraint::Length(7),
                    Constraint::Length(15),
                    Constraint::Length(10),
                ]);

            f.render_stateful_widget(table_files, rect_left[0], &mut table.state);
            f.render_stateful_widget(table_foo, rect_left[1], &mut table.state);
            f.render_stateful_widget(table_2, rect_right[0], &mut table.state);
            f.render_stateful_widget(table_3, rect_right[1], &mut table.state);
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => break,
                Key::Down | Key::Char('j') => {
                    table.select_next();
                }
                Key::Up | Key::Char('k') => {
                    table.select_previous();
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
