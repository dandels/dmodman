use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};

pub struct Rectangles {
    main_vertical_layout: Layout,
    tables_layout: Layout,
    statcounter_layout: Layout,
    inputline_layout: Layout,
    inputline_vert_limit: Layout,
    pub rect_root: Rc<[Rect]>,
    pub rect_main_horizontal: Rc<[Rect]>,
    pub rect_main_vertical: Rc<[Rect]>,
    pub rect_statcounter: Rc<[Rect]>,
    pub rect_inputline: Rc<[Rect]>,
}
impl Rectangles {
    pub fn new() -> Self {
        let main_vertical_layout: Layout = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(1),      // tab bar
            Constraint::Length(1),      // key bar
            Constraint::Percentage(75), // main vertical container
            Constraint::Fill(1),        // log view,
        ]);

        let tables_layout: Layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let statcounter_layout: Layout =
            Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1)]).flex(Flex::End);

        let inputline_layout: Layout =
            Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3)]).flex(Flex::Center);

        let inputline_vert_limit =
            Layout::default().direction(Direction::Horizontal).constraints([Constraint::Max(50)]).flex(Flex::Center);

        let (width, height) = termion::terminal_size().unwrap();

        let rect_root = Rc::new([Rect {
            x: 0,
            y: 0,
            height,
            width,
        }]);

        let rect_main_vertical = main_vertical_layout.split(rect_root[0]);
        let rect_main_horizontal = tables_layout.split(rect_main_vertical[2]);
        let rect_statcounter = statcounter_layout.split(rect_root[0]);
        let mut rect_inputline = inputline_layout.split(rect_root[0]);
        rect_inputline = inputline_vert_limit.split(rect_inputline[0]);

        Self {
            tables_layout,
            inputline_layout,
            inputline_vert_limit,
            main_vertical_layout,
            statcounter_layout,
            rect_root,
            rect_main_horizontal,
            rect_main_vertical,
            rect_statcounter,
            rect_inputline,
        }
    }
}

impl Rectangles {
    // TODO violates DRY, this is copypasted from constructor
    pub fn recalculate(&mut self, rect: Rect) {
        self.rect_root = [rect].into();
        self.rect_main_vertical = self.main_vertical_layout.split(self.rect_root[0]);
        self.rect_main_horizontal = self.tables_layout.split(self.rect_main_vertical[2]);
        self.rect_statcounter = self.statcounter_layout.split(self.rect_root[0]);
        self.rect_inputline = self.inputline_layout.split(self.rect_root[0]);
        self.rect_inputline = self.inputline_vert_limit.split(self.rect_inputline[0]);
    }
}
