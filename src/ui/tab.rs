use crate::ui::{component::traits::Select, tabs::Focused};

pub struct Tab {
    pub widget_types: &'static [Focused],
    pub focused_widget_index: usize,
}

impl Tab {
    pub fn new(widgets: &'static [Focused]) -> Self {
        Self {
            widget_types: widgets,
            focused_widget_index: 0,
        }
    }

    pub fn focused_widget_type(&self) -> Focused {
        self.widget_types[self.focused_widget_index]
    }
}

impl Select for Tab {
    fn len(&self) -> usize {
        self.widget_types.len()
    }

    fn select(&mut self, index: Option<usize>) {
        if let Some(index) = index {
            if index < self.len() {
                self.focused_widget_index = index;
            }
        }
    }

    fn selected(&self) -> Option<usize> {
        Some(self.focused_widget_index)
    }
}
