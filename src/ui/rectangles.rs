use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};

struct Layouts {
    top_bar: Layout,
    main_vertical: Layout,
    tables: Layout,
    bottom_bar: Layout,
}

impl Layouts {
    pub fn new() -> Self {
        let main_vertical = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(1), // tab bar
            Constraint::Length(1), // key bar
            Constraint::Fill(1),   // main vertical container
            Constraint::Length(1), // bottom bar
        ]);

        let tables = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let top_bar = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .flex(Flex::End);

        let bottom_bar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(1)])
            .flex(Flex::Start);

        Self {
            top_bar,
            main_vertical,
            tables,
            bottom_bar,
        }
    }
}

pub struct Rectangles {
    layouts: Layouts,
    pub main_horizontal: Rc<[Rect]>,
    pub main_vertical: Rc<[Rect]>,
    pub top_bar: Rc<[Rect]>,
    pub bottom_bar: Rc<[Rect]>,
    pub confirm_dialog: Rc<[Rect]>,
    pub dialog_popup: Rc<[Rect]>,
    pub dialog_popup_input_line: Rc<[Rect]>,
}

impl Rectangles {
    pub fn new() -> Self {
        let layouts = Layouts::new();
        Self {
            layouts,
            main_vertical: [Rect { ..Default::default() }].into(),
            top_bar: [Rect { ..Default::default() }].into(),
            bottom_bar: [Rect { ..Default::default() }].into(),
            main_horizontal: [Rect { ..Default::default() }].into(),
            confirm_dialog: [Rect { ..Default::default() }].into(),
            dialog_popup: [Rect { ..Default::default() }].into(),
            dialog_popup_input_line: [Rect { ..Default::default() }].into(),
        }
    }

    pub fn recalculate(&mut self, window_size: Rect) {
        self.top_bar = self.layouts.top_bar.split(window_size);
        self.main_vertical = self.layouts.main_vertical.split(window_size);
        self.main_horizontal = self.layouts.tables.split(self.main_vertical[2]);
        self.bottom_bar = self.layouts.bottom_bar.split(self.main_vertical[2]);
    }

    pub fn recalculate_popup(&mut self, list_height: usize, window_size: Rect) {
        let dialog_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),                                 // Paragraph with install dir path
                Constraint::Length(3),                                 // Input box
                Constraint::Length((list_height).try_into().unwrap()), // List with suggested values
            ])
            .flex(Flex::Center);

        let dialog_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(80)])
            .flex(Flex::Center);

        let label_and_input = Layout::default().direction(Direction::Horizontal).constraints([
            Constraint::Length(16), // magic number: length of label "Directory name:"
            Constraint::Fill(1),
        ]);

        self.dialog_popup = dialog_vertical.split(dialog_horizontal.split(window_size)[0]);
        self.dialog_popup_input_line = label_and_input.split(self.dialog_popup[1]);
    }

    pub fn recalculate_confirmdialog(&mut self, list_height: usize, window_size: Rect) {
        let dialog_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length((list_height + 2).try_into().unwrap())])
            .flex(Flex::Center);

        let dialog_horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(50)])
            .flex(Flex::Center);

        self.confirm_dialog = dialog_vertical.split(dialog_horizontal.split(window_size)[0]);
    }
}
