use ratatui::style::{Color, Style, Styled};
use ratatui::text::Line;
use ratatui::widgets::Tabs;

pub const TAB_COUNT: usize = 3;
pub const TAB_TITLES: [&str; TAB_COUNT] = ["Archives", "Installed", "Log"];

pub struct TabDisplay<'a> {
    title_lines: Vec<Line<'a>>,
    pub widget: Tabs<'a>,
}

impl<'a> TabDisplay<'a> {
    pub fn new() -> Self {
        let highlight_style = Style::new().bg(Color::White).fg(Color::Black);
        let title_lines: Vec<Line<'a>> = TAB_TITLES.iter().map(|f| Line::from(*f)).collect();
        let widget = Tabs::new(title_lines.clone()).highlight_style(highlight_style);

        Self { title_lines, widget }
    }

    /* This is a bit fragile since urgency highlight conflicts with regular highlight and the state is tracked outside
     * this component */
    // Urgency is useful and could have its own trait
    pub fn add_urgency(&mut self, index: usize) {
        self.set_tab_urgency(index, Style::new().fg(Color::LightYellow));
    }

    pub fn remove_urgency(&mut self, index: usize) {
        self.set_tab_urgency(index, Style::new());
    }

    fn set_tab_urgency(&mut self, index: usize, style: Style) {
        let title = self.title_lines.get_mut(index).unwrap().to_owned().set_style(style);
        *self.title_lines.get_mut(index).unwrap() = title;
        self.widget = self.widget.clone().titles(self.title_lines.clone());
    }

    pub fn focus_tab(&mut self, index: usize) {
        self.widget = self.widget.to_owned().select(index);
    }
}
