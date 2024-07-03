use super::{ArchiveTable, DownloadTable, InstalledModsTable};
use crate::api::UpdateStatus;
use crate::install::ModDirectory;
use crate::ui::navigation::Focused;
use crate::Cache;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const STYLE_OUTOFDATE: Style = Style::new().fg(Color::Red);
const STYLE_HASNEWFILE: Style = Style::new().fg(Color::Yellow);

pub struct BottomBar<'a> {
    cache: Cache,
    pub widget: Paragraph<'a>,
    prev_focused: Focused,
    prev_selected_index: Option<usize>,
    pub selected_has_changed: Arc<AtomicBool>,
}

impl<'a> BottomBar<'a> {
    pub fn new(cache: Cache, focused: Focused) -> Self {
        Self {
            cache,
            widget: Paragraph::default(),
            prev_focused: focused,
            prev_selected_index: None,
            selected_has_changed: Default::default(),
        }
    }

    pub async fn refresh(
        &mut self,
        archives: &ArchiveTable<'_>,
        installed: &InstalledModsTable<'_>,
        downloads: &DownloadTable<'_>,
        focused: &Focused,
        focused_index: Option<usize>,
    ) -> bool {
        if *focused != self.prev_focused
            || !focused_index.eq(&self.prev_selected_index)
            || self.selected_has_changed.swap(false, Ordering::Relaxed)
        {
            if let Some(focused_index) = focused_index {
                match focused {
                    // TODO get rid of copypaste
                    Focused::InstalledMods => {
                        let (_, mod_dir) = installed.get_by_index(focused_index);
                        if let ModDirectory::Nexus(im) = mod_dir {
                            let mut modname = StatusField::from_mod_name(im.mod_name.clone());
                            if modname.is_none() {
                                if let Some(mfd) = self.cache.metadata_index.get_by_file_id(&im.file_id).await {
                                    modname = StatusField::from_mod_name(mfd.mod_name().await);
                                }
                            }
                            let flags = StatusField::from_update_status(im.update_status.to_enum());
                            self.widget = Paragraph::new(Line::from(format_fields(vec![modname, flags])));
                        } else {
                            self.widget = Paragraph::default();
                        }
                    }
                    Focused::ArchiveTable => {
                        let (_, archive) = archives.get_by_index(focused_index);
                        if let Some(metadata) = archive.metadata() {
                            if let Some(mfd) = self.cache.metadata_index.get_by_file_id(&metadata.file_id).await {
                                let modname = mfd.mod_name().await.map(|n| StatusField::new("Mod", n.clone()));
                                let flags = StatusField::from_update_status(mfd.update_status.to_enum());
                                self.widget = Paragraph::new(Line::from(format_fields(vec![modname, flags])));
                            }
                        } else {
                            self.widget = Paragraph::default();
                        }
                    }
                    Focused::DownloadTable => {
                        let file_info = downloads.get_by_index(focused_index);
                        if let Some(mfd) = self.cache.metadata_index.get_by_file_id(&file_info.file_id).await {
                            let modname = mfd.mod_name().await.map(|n| StatusField::new("Mod", n.clone()));
                            let flags = StatusField::from_update_status(mfd.update_status.to_enum());
                            self.widget = Paragraph::new(Line::from(format_fields(vec![modname, flags])));
                        } else {
                            self.widget = Paragraph::default();
                        }
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

    pub fn from_mod_name(name: Option<String>) -> Option<Self> {
        name.map(|name| StatusField::new("Mod", name.clone()).style(Style::default().fg(Color::White)))
    }

    pub fn from_update_status(update_status: UpdateStatus) -> Option<Self> {
        match update_status {
            UpdateStatus::OutOfDate(_) => {
                Some(StatusField::new("Flags", "Out of date".to_string()).style(STYLE_OUTOFDATE))
            }
            UpdateStatus::HasNewFile(_) => {
                Some(StatusField::new("Flags", "Mod has new file".to_string()).style(STYLE_HASNEWFILE))
            }
            _ => None,
        }
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
