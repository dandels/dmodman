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
        self.title_lines.get_mut(index).unwrap().to_owned().set_style(Style::new().fg(Color::LightYellow));
        self.widget = self.widget.to_owned().titles(&mut self.title_lines.iter().cloned());
    }

    pub fn remove_urgency(&mut self, index: usize) {
        self.title_lines.get_mut(index).unwrap().to_owned().set_style(Style::new());
        self.widget = self.widget.to_owned().titles(&mut self.title_lines.iter().cloned());
    }

    pub fn focus_tab(&mut self, index: usize) {
        self.widget = self.widget.to_owned().select(index);
    }
}
