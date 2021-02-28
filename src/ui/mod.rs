mod component;
mod event;

use self::component::StatefulList;
use self::event::{Event, Events};

use std::io;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Terminal;

enum SelectedView {
    None,
    Files,
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut selected_view: SelectedView = SelectedView::None;

    let events = Events::new();

    let mut errors: StatefulList<&str> = StatefulList::with_items(vec!["Item0", "Item1", "Item2"]);

    let rect_main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(50)])
        .margin(0);

    loop {
        terminal.draw(|f| {
            let blocks = rect_main.split(f.size());

            //let left_table = create_left_table(selected_view);
            let error_list = create_error_list(errors.clone());
            f.render_stateful_widget(error_list, blocks[1], &mut errors.state);
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

fn create_left_table(view: SelectedView) {}

fn create_error_list(errors: StatefulList<&str>) -> List {
    let list_items: Vec<ListItem> = errors
        .items
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
