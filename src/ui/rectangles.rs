use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};

pub struct Layouts {
    topbar: Layout,
    main_vertical: Layout,
    tables: Layout,
}

impl Layouts {
    pub fn new() -> Self {
        let main_vertical = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(1),      // tab bar
            Constraint::Length(1),      // key bar
            Constraint::Percentage(70), // main vertical container
            Constraint::Fill(1),        // log view,
            Constraint::Length(1),      // bottom bar
        ]);

        let tables = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let topbar = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .flex(Flex::End);

        Self {
            topbar,
            main_vertical,
            tables,
        }
    }
}

pub struct Rectangles {
    pub main_horizontal: Rc<[Rect]>,
    pub main_vertical: Rc<[Rect]>,
    pub topbar: Rc<[Rect]>,
    pub confirmdialog: Rc<[Rect]>,
    pub dialogpopup: Rc<[Rect]>,
    pub dialogpopup_inputline: Rc<[Rect]>,
}

impl Rectangles {
    pub fn recalculate(&mut self, layout: &Layouts, window_size: Rect) {
        self.topbar = layout.topbar.split(window_size);
        self.main_vertical = layout.main_vertical.split(window_size);
        self.main_horizontal = layout.tables.split(self.main_vertical[2]);
    }

    pub fn recalculate_popup(&mut self, list_height: usize, window_size: Rect) {
        let dialog_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Paragraph with install dir path
                Constraint::Length(3), // Input box
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

        self.dialogpopup = dialog_vertical.split(dialog_horizontal.split(window_size)[0]);
        self.dialogpopup_inputline = label_and_input.split(self.dialogpopup[1]);
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

        self.confirmdialog = dialog_vertical.split(dialog_horizontal.split(window_size)[0]);
    }
}

impl Default for Rectangles {
    fn default() -> Self {
        Self {
            main_vertical: [Rect { ..Default::default() }].into(),
            topbar: [Rect { ..Default::default() }].into(),
            main_horizontal: [Rect { ..Default::default() }].into(),
            confirmdialog: [Rect { ..Default::default() }].into(),
            dialogpopup: [Rect { ..Default::default() }].into(),
            dialogpopup_inputline: [Rect { ..Default::default() }].into(),
        }
    }
}
