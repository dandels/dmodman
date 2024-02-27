use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ratatui::widgets::Clear;
use tokio::task;

use super::component::traits::*;
use super::component::*;
use super::event::{Events, TickEvent};
use crate::api::{Client, Downloads, UpdateChecker};
use crate::archives::Archives;
use crate::cache::Cache;
use crate::config::Config;
use crate::ui::rectangles::{Layouts, Rectangles};
use crate::ui::*;
use crate::Logger;

pub enum InputMode {
    Normal,
    ReadLine,
}

pub struct MainUI<'a> {
    pub cache: Cache,
    pub downloads: Downloads,
    pub logger: Logger,
    pub updater: UpdateChecker,
    pub focused: FocusedWidget,
    pub tab_bar: TabBar<'a>,
    pub hotkey_bar: HotkeyBar<'a>,
    pub bottom_bar: BottomBar<'a>,
    pub archives_view: ArchiveTable<'a>,
    pub files_view: FileTable<'a>,
    pub downloads_view: DownloadTable<'a>,
    pub log_view: LogList<'a>,
    pub popup_dialog: PopupDialog<'a>,
    pub input_mode: InputMode,
    pub should_run: bool,
    pub redraw_terminal: bool,
}

impl MainUI<'_> {
    pub async fn new(
        cache: Cache,
        client: Client,
        config: Config,
        downloads: Downloads,
        logger: Logger,
        archives: Archives,
    ) -> Self {
        let updater = UpdateChecker::new(cache.clone(), client.clone(), config, logger.clone());

        let focused = FocusedWidget::FileTable;

        let tab_bar = TabBar::new();
        let hotkey_bar = HotkeyBar::new(focused.clone());
        let bottom_bar = BottomBar::new(client.request_counter);
        let archives_view = ArchiveTable::new(archives).await;
        let files_view = FileTable::new(cache.file_index.clone());
        let downloads_view = DownloadTable::new(downloads.clone());
        let log_view = LogList::new(logger.clone());
        let popup_dialog = PopupDialog::default();

        Self {
            cache,
            downloads,
            focused,
            tab_bar,
            hotkey_bar,
            archives_view,
            files_view,
            downloads_view,
            log_view,
            bottom_bar,
            popup_dialog,
            input_mode: InputMode::Normal,
            updater,
            logger,
            should_run: true,
            redraw_terminal: true,
        }
    }

    /* This is the main UI loop.
     * Redrawing the terminal is CPU intensive - locks and atomics are used to ensure it's done only when necessary. */
    pub async fn run(mut self) {
        let mut events = Events::new();
        self.files_view.focus();
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

        while self.should_run {
            // set redraw_terminal to true if any of the widgets have changed
            if self.tab_bar.selected().unwrap() == 0 {
                self.redraw_terminal |= self.files_view.refresh().await;
                self.redraw_terminal |= self.downloads_view.refresh().await;
            } else if self.tab_bar.selected().unwrap() == 1 {
                self.redraw_terminal |= self.archives_view.refresh().await;
            }
            self.redraw_terminal |= self.hotkey_bar.refresh(&self.focused).await;
            self.redraw_terminal |= self.tab_bar.refresh().await;
            self.redraw_terminal |= self.bottom_bar.refresh().await;
            self.redraw_terminal |= self.log_view.refresh().await;

            let recalculate_rects = got_sigwinch.swap(false, Ordering::Relaxed);

            if self.redraw_terminal || recalculate_rects {
                terminal
                    .draw(|frame| {
                        self.redraw_terminal = false;
                        if recalculate_rects {
                            rectangles.recalculate(&layouts, frame.size());
                        }
                        if let InputMode::ReadLine = self.input_mode {
                            rectangles.recalculate_popup(self.popup_dialog.len, frame.size());
                        }
                        if self.tab_bar.selected().unwrap() == 0 {
                            frame.render_stateful_widget(
                                &self.files_view.widget,
                                rectangles.main_horizontal[0],
                                &mut self.files_view.state,
                            );
                            frame.render_stateful_widget(
                                &self.downloads_view.widget,
                                rectangles.main_horizontal[1],
                                &mut self.downloads_view.state,
                            );
                        } else if self.tab_bar.selected().unwrap() == 1 {
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

                        frame.render_widget(&self.tab_bar.widget, rectangles.main_vertical[0]);
                        frame.render_widget(&self.hotkey_bar.widget, rectangles.main_vertical[1]);
                        frame.render_widget(&self.bottom_bar.widget, rectangles.statcounter[0]);

                        // TODO use same rendering logic as other widgets
                        if let InputMode::ReadLine = self.input_mode {
                            // Clear the area so we can render on top of it
                            frame.render_widget(Clear, rectangles.dialogpopup[0]);
                            frame.render_widget(Clear, rectangles.dialogpopup[1]);
                            frame.render_stateful_widget(
                                &self.popup_dialog.list,
                                rectangles.dialogpopup[0],
                                &mut self.popup_dialog.state,
                            );
                            frame.render_widget(self.popup_dialog.widget(), rectangles.dialogpopup[1]);
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
