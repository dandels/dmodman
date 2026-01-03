use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Tabs;

use crate::ui::tabs::TabType;

pub struct TabDisplay<'a> {
    tab_titles: [&'a str; 3],
    title_lines: Vec<Line<'a>>,
    pub widget: Tabs<'a>,
}

impl<'a> TabDisplay<'a> {
    pub fn new() -> Self {
        let highlight_style = Style::new().bg(Color::White).fg(Color::Black);
        let tab_titles = ["Archives", "Installed", "Log"];
        let title_lines: Vec<Line<'a>> = tab_titles.into_iter().map(Line::from).collect();
        let widget = Tabs::new(title_lines.clone()).highlight_style(highlight_style);

        Self {
            tab_titles,
            title_lines,
            widget,
        }
    }

    /* This is a bit fragile since urgency highlight conflicts with regular highlight and the state is tracked outside
     * this component */
    // Urgency is useful and could have its own trait
    pub fn add_urgency(&mut self, tab: TabType) {
        self.title_lines
            .get_mut(tab as usize)
            .unwrap()
            .to_owned()
            .style(Style::new().fg(Color::LightYellow));
        self.widget.to_owned().titles(&mut self.title_lines.to_owned().into_iter());
    }

    pub fn remove_urgency(&mut self, index: usize) {
        self.title_lines.get_mut(index).unwrap().to_owned().style(Style::new());
        self.widget.to_owned().titles(&mut self.title_lines.to_owned().into_iter());
    }

    pub fn focus_tab(&mut self, index: usize) {
        self.widget.to_owned().select(index);
    }
}
