use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};

pub struct Layouts {
    main_vertical: Layout,
    tables: Layout,
    statcounter: Layout,
    dialog_horizontal: Layout,
    dialog_vertical: Layout,
}

pub struct Rectangles {
    pub main_horizontal: Rc<[Rect]>,
    pub main_vertical: Rc<[Rect]>,
    pub statcounter: Rc<[Rect]>,
    pub dialogpopup: Rc<[Rect]>,
}

impl Default for Rectangles {
    fn default() -> Self {
        Self {
            main_vertical: [Rect { ..Default::default() }].into(),
            statcounter: [Rect { ..Default::default() }].into(),
            main_horizontal: [Rect { ..Default::default() }].into(),
            dialogpopup: [Rect { ..Default::default() }].into(),
        }
    }
}

impl Layouts {
    pub fn new() -> Self {
        let main_vertical = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(1),      // tab bar
            Constraint::Length(1),      // key bar
            Constraint::Percentage(75), // main vertical container
            Constraint::Fill(1),        // log view,
        ]);

        let tables = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let statcounter =
            Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1)]).flex(Flex::End);

        let dialog_horizontal =
            Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3)]).flex(Flex::Center);

        let dialog_vertical =
            Layout::default().direction(Direction::Horizontal).constraints([Constraint::Max(50)]).flex(Flex::Center);

        Self {
            main_vertical,
            tables,
            statcounter,
            dialog_horizontal,
            dialog_vertical,
        }
    }
}

impl Rectangles {
    pub fn recalculate(&mut self, layout: &Layouts, window_size: Rect) {
        self.main_vertical = layout.main_vertical.split(window_size);
        self.main_horizontal = layout.tables.split(self.main_vertical[2]);
        self.statcounter = layout.statcounter.split(window_size);
        self.dialogpopup = layout.dialog_vertical.split(layout.dialog_horizontal.split(window_size)[0]);
    }
}
