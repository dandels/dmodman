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
use crate::ui::rectangles::Rectangles;
use crate::ui::*;
use crate::Logger;

pub enum InputMode {
    Normal,
    ReadLine,
}

pub struct MainUI<'a> {
    pub archives: Archives,
    pub cache: Cache,
    pub downloads: Downloads,
    pub logger: Logger,
    pub updater: UpdateChecker,
    rectangles: Rectangles,
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
    pub redraw_terminal: Arc<AtomicBool>,
    pub should_run: bool,
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

        let redraw_terminal = Arc::new(AtomicBool::new(true));

        let focused = FocusedWidget::FileTable;

        let tab_bar = TabBar::new(redraw_terminal.clone());
        let hotkey_bar = HotkeyBar::new(focused.clone());
        let bottom_bar = BottomBar::new(redraw_terminal.clone(), client.request_counter);
        let archives_view = ArchiveTable::new(redraw_terminal.clone());
        let files_view = FileTable::new(redraw_terminal.clone(), cache.file_index.clone());
        let downloads_view = DownloadTable::new(redraw_terminal.clone(), downloads.clone());
        let log_view = LogList::new(redraw_terminal.clone(), logger.clone());
        let popup_dialog = PopupDialog::new(redraw_terminal.clone());

        Self {
            archives,
            cache,
            downloads,
            rectangles: Rectangles::new(),
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
            redraw_terminal,
            updater,
            logger,
            should_run: true,
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

        while self.should_run {
            self.files_view.refresh().await;
            self.downloads_view.refresh().await;
            self.log_view.refresh().await;
            self.archives_view.refresh(&mut self.archives).await;
            self.hotkey_bar.refresh(&self.focused).await;
            self.tab_bar.refresh().await;
            self.bottom_bar.refresh().await;

            let recalculate_rects = got_sigwinch.swap(false, Ordering::Relaxed);

            if self.redraw_terminal.swap(false, Ordering::Relaxed) || recalculate_rects {
                terminal
                    .draw(|frame| {
                        if recalculate_rects {
                            self.rectangles.recalculate(frame.size());
                        }
                        if self.tab_bar.selected().unwrap() == 0 {
                            frame.render_stateful_widget(
                                &self.files_view.widget,
                                self.rectangles.rect_main_horizontal[0],
                                &mut self.files_view.state,
                            );
                            frame.render_stateful_widget(
                                &self.downloads_view.widget,
                                self.rectangles.rect_main_horizontal[1],
                                &mut self.downloads_view.state,
                            );
                        } else if self.tab_bar.selected().unwrap() == 1 {
                            frame.render_stateful_widget(
                                &self.archives_view.widget,
                                self.rectangles.rect_main_vertical[2],
                                &mut self.archives_view.state,
                            );
                        }
                        frame.render_stateful_widget(
                            &self.log_view.widget,
                            self.rectangles.rect_main_vertical[3],
                            &mut self.log_view.state,
                        );

                        frame.render_widget(&self.tab_bar.widget, self.rectangles.rect_main_vertical[0]);
                        frame.render_widget(&self.hotkey_bar.widget, self.rectangles.rect_main_vertical[1]);
                        frame.render_widget(&self.bottom_bar.widget, self.rectangles.rect_statcounter[0]);

                        if let InputMode::ReadLine = self.input_mode {
                            // Clear the space first so we can draw on top of the rest of the widgets
                            frame.render_widget(Clear, self.rectangles.rect_inputline[0]);
                            frame.render_widget(self.input_line.widget(), self.rectangles.rect_inputline[0]);
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
