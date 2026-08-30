use super::{ArchivesWidget, DownloadsWidget, InstalledModsWidget};
use crate::api::UpdateStatus;
use crate::extract::ModDirectory;
use crate::ui::component::traits::Select;
use crate::ui::tabs::{Focused, TabWidgets};
use crate::Db;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

const STYLE_OUTOFDATE: Style = Style::new().fg(Color::Red);
const STYLE_HASNEWFILE: Style = Style::new().fg(Color::Yellow);

pub struct BottomBar<'a> {
    db: Db,
    pub widget: Paragraph<'a>,
}

impl<'a> BottomBar<'a> {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            widget: Paragraph::default(),
        }
    }

    pub async fn update_widget(&mut self, tabs: &TabWidgets<'a>) {
        match tabs.focused_widget_type() {
            Focused::ArchiveTable => self.refresh_for_archive_table(&tabs.archive_table).await,
            Focused::DownloadTable => self.refresh_for_downloads(&tabs.downloads_table).await,
            Focused::InstalledMods => self.refresh_for_installed_mods(&tabs.installed_mods_table).await,
            Focused::LogList => self.focus_none(),
        }
    }

    pub async fn refresh_for_installed_mods(&mut self, installed_mods_table: &InstalledModsWidget<'a>) {
        if let Some(focused_index) = installed_mods_table.selected() {
            let (_, mod_dir) = installed_mods_table.get_by_index(focused_index);
            if let ModDirectory::Nexus(im) = mod_dir {
                let mut modname = StatusField::from_mod_name(im.mod_name.clone());
                if modname.is_none() {
                    if let Some(mfd) = self.db.metadata_index.get_by_file_id(&im.file_id).await {
                        modname = StatusField::from_mod_name(mfd.mod_name().await);
                    }
                }
                let flags = StatusField::from_update_status(im.update_status.to_enum());
                self.widget = Paragraph::new(Line::from(format_fields(vec![modname, flags])));
            } else {
                self.focus_none();
            }
        }
    }

    pub async fn refresh_for_archive_table(&mut self, archive_table: &ArchivesWidget<'a>) {
        if let Some(focused_index) = archive_table.selected() {
            let (_, archive) = archive_table.get_by_index(focused_index);
            if let Some(metadata) = archive.metadata() {
                if let Some(mfd) = self.db.metadata_index.get_by_file_id(&metadata.file_id).await {
                    let modname = mfd.mod_name().await.map(|n| StatusField::new("Mod", n.clone()));
                    let flags = StatusField::from_update_status(mfd.update_status.to_enum());
                    self.widget = Paragraph::new(Line::from(format_fields(vec![modname, flags])));
                }
            } else {
                self.focus_none();
            }
        }
    }

    pub async fn refresh_for_downloads(&mut self, downloads_table: &DownloadsWidget<'a>) {
        if let Some(focused_index) = downloads_table.selected() {
            let file_info = downloads_table.get_by_index(focused_index);
            if let Some(mfd) = self.db.metadata_index.get_by_file_id(&file_info.file_id).await {
                let modname = mfd.mod_name().await.map(|n| StatusField::new("Mod", n.clone()));
                let flags = StatusField::from_update_status(mfd.update_status.to_enum());
                self.widget = Paragraph::new(Line::from(format_fields(vec![modname, flags])));
            } else {
                self.focus_none();
            }
        }
    }

    fn focus_none(&mut self) {
        self.widget = Paragraph::default();
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
