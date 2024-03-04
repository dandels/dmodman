use crate::ui::navigation::Focused;
use crate::cache::{Cache, UpdateStatus};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct BottomBar<'a> {
    cache: Cache,
    pub widget: Paragraph<'a>,
    prev_focused: Focused,
    prev_selected_index: Option<usize>,
}

impl<'a> BottomBar<'a> {
    pub fn new(cache: Cache, focused: Focused) -> Self {
        Self {
            cache,
            widget: Paragraph::default(),
            prev_focused: focused,
            prev_selected_index: None,
        }
    }

    pub async fn refresh(&mut self, focused: &Focused, focused_index: Option<usize>) -> bool {
        if *focused != self.prev_focused || !focused_index.eq(&self.prev_selected_index) {
            if let Some(focused_index) = focused_index {
                #[allow(clippy::single_match)]
                match focused {
                    Focused::FileTable => {
                        let fd = self.cache.file_index.get_by_index(focused_index).await;
                        let style_outofdate = Style::default().fg(Color::Red);
                        let style_hasnewfile = Style::default().fg(Color::Yellow);
                        let flags = match fd.local_file.update_status() {
                            UpdateStatus::OutOfDate(_) => {
                                Some(StatusField::new("Flags", "Out of date".to_string()).style(style_outofdate))
                            }
                            UpdateStatus::HasNewFile(_) => {
                                Some(StatusField::new("Flags", "Mod has new file".to_string()).style(style_hasnewfile))
                            }
                            _ => None,
                        };
                        let modname = fd.md5results.as_ref().and_then(|res| {
                            res.r#mod
                                .name
                                .clone()
                                .map(|name| StatusField::new("Mod", name).style(Style::default().fg(Color::White)))
                        });
                        self.widget = Paragraph::new(Line::from(format_fields(vec![flags, modname])));
                    }
                    _ => {
                        self.widget = Paragraph::default();
                    }
                }
            } else {
                self.widget = Paragraph::default();
            }
            self.prev_focused = focused.clone();
            self.prev_selected_index = focused_index;
            return true;
        }
        false
    }
}

struct StatusField<'a> {
    title: Span<'a>,
    value: Span<'a>,
}

impl<'a> StatusField<'a> {
    pub fn new(title: &'a str, value: String) -> Self {
        Self {
            title: Span::from(format!("{}: ", title)),
            value: Span::from(value),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.value = self.value.style(style);
        self
    }
}

fn format_fields<'a>(fields: Vec<Option<StatusField<'a>>>) -> Vec<Span<'a>> {
    let mut ret: Vec<Span<'a>> = vec![];

    let fields: Vec<StatusField> = fields.into_iter().flatten().collect();

    let len = fields.len();
    for (i, StatusField { title, value }) in fields.into_iter().enumerate() {
        ret.push(title);
        ret.push(value);
        if i + 1 < len {
            ret.push(Span::from(" | "));
        }
    }
    ret
}
