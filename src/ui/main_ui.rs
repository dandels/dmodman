use super::component::traits::*;
use super::component::*;
use super::navigation::*;
use crate::api::{Client, Downloads, UpdateChecker, Query};
use crate::cache::Cache;
use crate::config::Config;
use crate::install::Installer;
use crate::ui::rectangles::{Layouts, Rectangles};
use crate::ui::*;
use crate::Logger;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use tokio::task;

#[derive(Clone, Eq, PartialEq)]
pub enum InputMode {
    Normal,
    Confirm,
    ReadLine,
}

pub struct MainUI<'a> {
    // Structs handling app logic
    pub installer: Installer,
    pub cache: Cache,
    pub config: Arc<Config>,
    pub downloads: Downloads,
    pub logger: Logger,
    pub query: Query,
    pub updater: UpdateChecker,

    // UI widgets
    pub bottom_bar: BottomBar<'a>,
    pub archives_view: ArchiveTable<'a>,
    pub confirm_dialog: ConfirmDialog<'a>,
    pub downloads_view: DownloadTable<'a>,
    pub installed_mods_table: InstalledModsTable<'a>,
    pub hotkey_bar: HotkeyBar<'a>,
    pub log_view: LogList<'a>,
    pub popup_dialog: PopupDialog<'a>,
    pub top_bar: TopBar<'a>,

    // UI state
    pub tabs: Tabs,
    pub input_mode: InputMode,
    pub should_run: bool,
    pub redraw_terminal: bool,
}

impl MainUI<'_> {
    pub async fn new(
        cache: Cache,
        client: Client,
        config: Arc<Config>,
        downloads: Downloads,
        logger: Logger,
        query: Query,
    ) -> Self {
        let installer = Installer::new(cache.clone(), config.clone(), logger.clone()).await;
        let updater = UpdateChecker::new(cache.clone(), client.clone(), config.clone(), logger.clone(), query.clone());

        let tabs = Tabs::new();

        let archives_view = ArchiveTable::new(cache.clone()).await;
        let bottom_bar = BottomBar::new(cache.clone(), tabs.focused().clone());
        let confirm_dialog = ConfirmDialog::default();
        let downloads_view = DownloadTable::new(downloads.clone());
        let files_view = InstalledModsTable::new(cache.installed.clone());
        let hotkey_bar = HotkeyBar::new(tabs.focused().clone());
        let log_view = LogList::new(logger.clone());
        let popup_dialog = PopupDialog::default();
        let top_bar = TopBar::new(client.request_counter).await;

        Self {
            cache,
            config,
            downloads,
            installer,
            query,
            top_bar,
            hotkey_bar,
            archives_view,
            installed_mods_table: files_view,
            downloads_view,
            log_view,
            bottom_bar,
            confirm_dialog,
            popup_dialog,
            input_mode: InputMode::Normal,
            updater,
            logger,
            tabs,
            should_run: true,
            redraw_terminal: true,
        }
    }

    /* This is the main UI loop.
     * Redrawing the terminal is CPU intensive - locks and atomics are used to ensure it's done only when necessary. */
    pub async fn run(mut self) {
        let mut events = Events::new();
        self.installed_mods_table.add_highlight();
        // X11 (and maybe Wayland?) sends SIGWINCH when the window is resized
        // Set to true so rectangles are calculated on first loop
        let got_sigwinch = Arc::new(AtomicBool::new(true));
        let _sigwinch_task = task::spawn(handle_sigwinch(got_sigwinch.clone()));
        let mut terminal = match term_setup() {
            Ok(term) => term,
            Err(e) => {
                println!("Failed to initialize terminal: {}", e);
                return;
            }
        };

        let layouts = Layouts::new();
        let mut rectangles = Rectangles::default();
        let mut i = 0;

        while self.should_run {
            // set redraw_terminal to true if any of the widgets have changed
            if self.tabs.selected().unwrap() == 0 {
                self.redraw_terminal |= self.installed_mods_table.refresh().await;
                self.redraw_terminal |= self.downloads_view.refresh().await;
            } else if self.tabs.selected().unwrap() == 1 {
                self.redraw_terminal |= self.archives_view.refresh().await;
            }
            self.redraw_terminal |= self.top_bar.refresh(&self.tabs).await;
            self.redraw_terminal |= self.hotkey_bar.refresh(&self.input_mode, self.tabs.focused()).await;
            self.redraw_terminal |=
                self.bottom_bar.refresh(&self.archives_view, &self.installed_mods_table, self.tabs.focused(), self.focused_widget().selected()).await;
            self.redraw_terminal |= self.log_view.refresh().await;

            let recalculate_rects = got_sigwinch.swap(false, Ordering::Relaxed);

            if self.redraw_terminal || recalculate_rects {
                terminal
                    .draw(|frame| {
                        self.redraw_terminal = false;
                        i += 1;
                        if recalculate_rects {
                            rectangles.recalculate(&layouts, frame.size());
                        }
                        if let InputMode::ReadLine = self.input_mode {
                            rectangles.recalculate_popup(self.popup_dialog.get_required_height(), frame.size());
                        }
                        if let InputMode::Confirm = self.input_mode {
                            rectangles.recalculate_confirmdialog(self.confirm_dialog.len, frame.size());
                        }
                        match self.input_mode {
                            InputMode::Normal => {
                                if let Tab::Main = self.tabs.selected().unwrap().into() {
                                    frame.render_stateful_widget(
                                        &self.installed_mods_table.widget,
                                        rectangles.main_horizontal[0],
                                        &mut self.installed_mods_table.state,
                                    );
                                    frame.render_stateful_widget(
                                        &self.downloads_view.widget,
                                        rectangles.main_horizontal[1],
                                        &mut self.downloads_view.state,
                                    );
                                } else if let Tab::Archives = self.tabs.selected().unwrap().into() {
                                    frame.render_stateful_widget(
                                        &self.archives_view.widget,
                                        rectangles.main_vertical[2],
                                        &mut self.archives_view.state,
                                    );
                                }
                                frame.render_stateful_widget(
                                    &self.log_view.widget,
                                    rectangles.main_vertical[3],
                                    &mut self.log_view.state,
                                );

                                frame.render_widget(&self.top_bar.tabs_widget, rectangles.topbar[0]);
                                frame.render_widget(&self.top_bar.counter_widget, rectangles.topbar[1]);
                                frame.render_widget(&self.hotkey_bar.widget, rectangles.main_vertical[1]);
                                frame.render_widget(&self.bottom_bar.widget, rectangles.main_vertical[4]);
                            }
                            InputMode::ReadLine => {
                                // TODO use same rendering logic as other widgets
                                // Clear the area so we can render on top of it
                                //frame.render_widget(Clear, rectangles.dialogpopup[0]);
                                //frame.render_widget(Clear, rectangles.dialogpopup[1]);
                                frame.render_widget(
                                    Paragraph::new(format!(
                                        "Extracting to {}",
                                        self.config.install_dir().to_str().unwrap()
                                    )),
                                    rectangles.dialogpopup[0],
                                );
                                frame.render_widget(&self.popup_dialog.text_label, rectangles.dialogpopup_inputline[0]);
                                frame.render_widget(
                                    self.popup_dialog.textarea.widget(),
                                    rectangles.dialogpopup_inputline[1],
                                );
                                frame.render_stateful_widget(
                                    &self.popup_dialog.list,
                                    rectangles.dialogpopup[2],
                                    &mut self.popup_dialog.state,
                                );
                                frame.render_widget(&self.hotkey_bar.widget, rectangles.main_vertical[0]);
                            }
                            InputMode::Confirm => {
                                frame.render_widget(Clear, rectangles.confirmdialog[0]);
                                frame.render_stateful_widget(
                                    &self.confirm_dialog.widget,
                                    rectangles.confirmdialog[0],
                                    &mut self.confirm_dialog.state,
                                );
                            }
                        }
                    })
                    .unwrap();
            }

            if let Some(TickEvent::Input(event)) = events.next().await {
                self.handle_events(event).await;
            }
        }
    }
}
