use crate::api::Downloads;
use crate::db::FileDetailsCache;
use crate::Errors;

use tui::layout::Constraint;
use tui::style::{Color, Modifier, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table, TableState};

#[derive(Clone)]
pub struct ErrorList<'a> {
    pub widget: List<'a>,
    pub block: Block<'a>,
    pub errors: &'a Errors,
    pub state: ListState,
    pub highlight_style: Style,
}

impl<'a> ErrorList<'a> {
    pub fn new(errors: &'a Errors) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Errors");
        let highlight_style = Style::default();
        Self {
            widget: Self::create(block.clone(), errors, highlight_style),
            block,
            errors,
            state: ListState::default(),
            highlight_style,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(self.block.clone(), &self.errors, self.highlight_style);
    }

    fn create(block: Block<'a>, errors: &Errors, highlight_style: Style) -> List<'a> {
        let list_items: Vec<ListItem> = errors
            .errors
            .read()
            .unwrap()
            .iter()
            .map(|i| {
                let lines = vec![Spans::from(i.to_string())];
                ListItem::new(lines)
            })
            .collect();

        let error_list = List::new(list_items).block(block).highlight_style(highlight_style);
        error_list
    }

    pub fn is_changed(&self) -> bool {
        self.errors.is_changed()
    }
}

pub struct FileTable<'a> {
    pub widget: Table<'a>,
    pub block: Block<'a>,
    pub files: &'a FileDetailsCache,
    headers: Row<'a>,
    pub state: TableState,
    pub highlight_style: Style,
}

impl<'a> FileTable<'a> {
    pub fn new(files: &'a FileDetailsCache) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Files");

        let highlight_style = Style::default();

        let headers = Row::new(
            vec!["Name", "Version"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        Self {
            widget: Self::create(block.clone(), headers.clone(), files, highlight_style),
            block,
            files,
            headers,
            state: TableState::default(),
            highlight_style,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(
            self.block.clone(),
            self.headers.clone(),
            &self.files,
            self.highlight_style,
        );
    }

    // TODO handle missing FileDetails and foreign (non-Nexusmods) mods
    fn create(block: Block<'a>, headers: Row<'a>, files: &FileDetailsCache, highlight_style: Style) -> Table<'a> {
        let rows: Vec<Row> = files
            .items()
            .iter()
            .map(|x| {
                Row::new(vec![
                    x.name.clone(),
                    x.version.as_ref().unwrap_or(&"".to_string()).to_string(),
                ])
            })
            .collect();

        let table = Table::new(rows)
            .header(headers)
            .block(block)
            .widths(&[Constraint::Percentage(85), Constraint::Percentage(15)])
            .highlight_style(highlight_style);

        table
    }

    pub fn is_changed(&self) -> bool {
        self.files.is_changed()
    }
}

#[derive(Clone)]
pub struct DownloadTable<'a> {
    pub widget: Table<'a>,
    pub block: Block<'a>,
    pub downloads: &'a Downloads,
    headers: Row<'a>,
    pub state: TableState,
    pub highlight_style: Style,
}

impl<'a> DownloadTable<'a> {
    pub fn new(downloads: &'a Downloads) -> Self {
        let block = Block::default().borders(Borders::ALL).title("Downloads");

        let headers = Row::new(
            vec!["Filename", "Progress"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red))),
        );

        let highlight_style = Style::default();

        Self {
            widget: Self::create(block.clone(), headers.clone(), downloads, highlight_style),
            block,
            downloads,
            headers,
            state: TableState::default(),
            highlight_style,
        }
    }

    pub fn refresh(&mut self) {
        self.widget = Self::create(
            self.block.clone(),
            self.headers.clone(),
            &self.downloads,
            self.highlight_style,
        );
    }

    fn create(block: Block<'a>, headers: Row<'a>, downloads: &Downloads, highlight_style: Style) -> Table<'a> {
        let rows: Vec<Row> = downloads
            .statuses
            .read()
            .unwrap()
            .iter()
            .map(|x| {
                let x = x.read().unwrap();
                Row::new(vec![x.file_name.clone(), x.progress()])
            })
            .collect();

        let table = Table::new(rows)
            .header(headers)
            .block(block)
            .widths(&[Constraint::Percentage(70), Constraint::Percentage(30)])
            .highlight_style(highlight_style);

        table
    }

    pub fn is_changed(&self) -> bool {
        self.downloads.is_changed()
    }
}
