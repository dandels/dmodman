use std::rc::Rc;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Rectangles {
    topbar_layout: Layout,
    botbar_layout: Layout,
    tables_layout: Layout,
    main_vertical_layout: Layout,
    pub rect_root: Rc<[Rect]>,
    pub rect_tabbar: Rc<[Rect]>,
    pub rect_keybar: Rc<[Rect]>,
    pub rect_main: Rc<[Rect]>,
    pub rect_botbar: Rc<[Rect]>,
}
impl Rectangles {
    pub fn new() -> Self {
        // TODO learn to use the constraints
        let topbar_layout: Layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Percentage(99)]);

        let botbar_layout: Layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(99), Constraint::Min(1)]);

        let tables_layout: Layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(2, 4), Constraint::Ratio(2, 4)]);

        let main_vertical_layout: Layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)]);

        let (width, height) = termion::terminal_size().unwrap();

        let rect_root = main_vertical_layout.split(Rect {
            x: 0,
            y: 0,
            height,
            width,
        });
        let rect_tabbar = topbar_layout.split(rect_root[0]);
        let rect_keybar = Rc::new([Rect {
            y: rect_tabbar[0].y + 1,
            ..rect_tabbar[0]
        }]);
        let rect_main = tables_layout.split(rect_tabbar[1]);
        let rect_botbar = botbar_layout.split(rect_root[1]);

        Self {
            topbar_layout,
            botbar_layout,
            tables_layout,
            main_vertical_layout,
            rect_root,
            rect_tabbar,
            rect_keybar,
            rect_main,
            rect_botbar,
        }
    }
}

impl Rectangles {
    pub fn recalculate(&mut self, rect: Rect) {
        self.rect_root = self.main_vertical_layout.split(rect);
        self.rect_tabbar = self.topbar_layout.split(self.rect_root[0]);
        self.rect_keybar = Rc::new([Rect {
            y: self.rect_tabbar[0].y + 1,
            ..self.rect_tabbar[0]
        }]);
        self.rect_main = self.tables_layout.split(self.rect_tabbar[1]);
        self.rect_botbar = self.botbar_layout.split(self.rect_root[1]);
    }
}
