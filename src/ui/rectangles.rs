use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use std::rc::Rc;

struct NormalLayouts {
    top_bar: Layout,
    main_vertical: Layout,
    bottom_bar: Layout,
    main_content: Layout,
}

impl NormalLayouts {
    fn new(main_pane_count: usize) -> Self {
        let main_vertical = Layout::vertical([
            Constraint::Length(1), // tab bar
            Constraint::Length(1), // key bar
            Constraint::Fill(1),   // main vertical container
            Constraint::Length(1), // bottom bar
        ]);

        let top_bar = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 4), Constraint::Ratio(3, 4)])
            .flex(Flex::End);

        let bottom_bar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(1)])
            .flex(Flex::Start);

        let mut ret = Self {
            top_bar,
            main_vertical,
            bottom_bar,
            main_content: Default::default(),
        };
        ret.set_pane_count(main_pane_count);
        ret
    }

    pub fn set_pane_count(&mut self, count: usize) {
        self.main_content = Layout::horizontal([Constraint::Fill(1)].repeat(count));
    }
}

#[derive(Default)]
pub struct ExtractDialogLayouts {
    horizontal: Layout,
    vertical: Layout,
    label_and_textbox: Layout,
}

impl ExtractDialogLayouts {
    fn new() -> Self {
        let horizontal: Layout = Layout::horizontal([Constraint::Max(80)]).flex(Flex::Center);

        let label_and_textbox: Layout = Layout::horizontal([
            Constraint::Length(16), // magic number: length of label "Directory name:"
            Constraint::Fill(1),
        ])
        .flex(Flex::Center);

        let mut ret = Self {
            horizontal,
            vertical: Default::default(),
            label_and_textbox,
        };
        ret.set_suggestions_len(0);
        ret
    }

    pub fn set_suggestions_len(&mut self, len: usize) {
        self.vertical = Layout::vertical([
            Constraint::Length(1),          // Paragraph with install dir path
            Constraint::Length(3),          // Input box
            Constraint::Length(len as u16), // List with suggested values
        ]);
    }
}

pub struct Rectangles {
    pub normal: NormalRects,
    pub confirm_dialog: ConfirmDialogRects,
    pub extract_dialog: ExtractDialogRects,
}

pub struct NormalRects {
    layouts: NormalLayouts,
    pub tabs: Rect,
    pub request_counter: Rect,
    pub hotkey_bar: Rect,
    pub main_content_panes: Rc<[Rect]>,
    pub bottom_bar: Rect,
}

#[derive(Default)]
pub struct ExtractDialogRects {
    pub layouts: ExtractDialogLayouts,
    pub description: Rect,
    pub prompt: Rect,
    pub textbox: Rect,
    pub suggestions: Rect,
}

pub struct ConfirmDialogRects {
    layout_horizontal: Layout,
    pub rect: Rect,
}

impl Rectangles {
    pub fn new(main_pane_count: usize) -> Self {
        Self {
            normal: NormalRects {
                layouts: NormalLayouts::new(main_pane_count),
                tabs: Default::default(),
                request_counter: Default::default(),
                hotkey_bar: Default::default(),
                main_content_panes: Default::default(),
                bottom_bar: Default::default(),
            },
            confirm_dialog: ConfirmDialogRects {
                layout_horizontal: Layout::horizontal([Constraint::Max(50)]).flex(Flex::Center),
                rect: Default::default(),
            },
            extract_dialog: ExtractDialogRects::new(),
        }
    }
}
impl NormalRects {
    pub fn recalculate(&mut self, window_size: Rect) {
        let main_vertical = self.layouts.main_vertical.split(window_size);
        let top_bar = self.layouts.top_bar.split(main_vertical[0]);
        self.tabs = top_bar[0];
        self.request_counter = top_bar[1];
        self.hotkey_bar = main_vertical[1];
        self.main_content_panes = self.layouts.main_content.split(main_vertical[2]);
        self.bottom_bar = main_vertical[3];
    }
}

impl ConfirmDialogRects {
    pub fn recalculate(&mut self, list_height: usize, window_size: Rect) {
        let dialog_vertical = Layout::vertical([Constraint::Length((list_height + 2) as u16)]).flex(Flex::Center);
        self.rect = dialog_vertical.split(self.layout_horizontal.split(window_size)[0])[0];
    }
}

impl ExtractDialogRects {
    pub fn new() -> Self {
        let layouts = ExtractDialogLayouts::new();
        Self {
            layouts,
            ..Default::default()
        }
    }

    pub fn recalculate(&mut self, window_size: Rect) {
        let split = self.layouts.vertical.split(self.layouts.horizontal.split(window_size)[0]);
        self.description = split[0];
        let dialog_and_textbox = self.layouts.label_and_textbox.split(split[1]);
        self.prompt = dialog_and_textbox[0];
        self.textbox = dialog_and_textbox[1];
        self.suggestions = split[2];
    }
}
