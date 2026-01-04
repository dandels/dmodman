use std::io::Stdout;

use super::component::*;
use crate::api::{Downloads, Query, UpdateChecker};
use crate::db::Db;
use crate::events::EventSource;
use crate::extract::Installer;
use crate::prelude::*;
use crate::ui::rectangles::Rectangles;
use crate::ui::tabs::{FocusedWidget, TabWidgets};
use crate::ui::*;
use crate::Lib;
use ratatui::layout::Rect;
use ratatui::prelude::TermionBackend;
use ratatui::widgets::WidgetRef;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use termion::screen::IntoAlternateScreen;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub enum NeedsRefresh {
    BottomBar,
}

pub enum UIEvent {
    Backend(EventSource),
    Terminal(TickEvent),
    SigWinch,
    Frontend(NeedsRefresh),
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Confirm,
    Extract,
}

type TermBackend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>;

pub struct MainUI<'a> {
    // Backend structs
    pub installer: Installer,
    pub db: Db,
    pub downloads: Downloads,
    pub query: Query,
    pub updater: UpdateChecker,

    // Top level UI widgets.
    pub requests_widget: RequestCounterWidget<'a>,
    pub bottom_bar: BottomBar<'a>,
    pub hotkey_bar: HotkeyBar<'a>,

    // Contains Tab display widget and focusable widgets
    pub tabs: TabWidgets<'a>,

    // Alternate UI states
    pub confirm_dialog: ConfirmDialog<'a>,
    pub popup_dialog: PopupDialog<'a>,

    // UI state
    terminal: Terminal<TermBackend>,
    pub rectangles: Rectangles,
    pub ui_events_tx: mpsc::UnboundedSender<NeedsRefresh>,
    pub input_mode: InputMode,
    pub should_run: bool,
}

impl MainUI<'_> {
    pub async fn start(lib: Lib) {
        let Lib {
            db,
            client,
            config,
            downloads,
            query,
        } = lib;
        let installer = Installer::new(config.clone(), db.clone()).await;
        let updater = UpdateChecker::new(db.clone(), client.clone(), config.clone(), query.clone());

        // Generic widgets
        let bottom_bar = BottomBar::new(db.clone());
        let confirm_dialog = ConfirmDialog::default();
        let hotkey_bar = HotkeyBar::new();
        let popup_dialog = PopupDialog::init(config.clone());
        let requests_widget = RequestCounterWidget::new(client.request_counter.clone()).await;

        // Contains tab and tab-switchable widgets
        let tabs = TabWidgets::new(db.clone(), downloads.clone()).await;

        let terminal = init_term().expect("Failed to initialize terminal: {}");
        let (ui_events_tx, ui_events_rx) = mpsc::unbounded_channel::<NeedsRefresh>();

        let ui = Self {
            db,
            downloads,
            installer,
            query,
            requests_widget,
            hotkey_bar,
            bottom_bar,
            confirm_dialog,
            popup_dialog,
            input_mode: InputMode::default(),
            updater,
            rectangles: Rectangles::new(tabs.focused_tab().widget_types.len()),
            tabs,
            // nav,
            should_run: true,
            terminal,
            ui_events_tx,
        };
        ui.run(ui_events_rx).await
    }

    fn recalc_rects(&mut self) {
        let window_size = self.terminal.get_frame().area();
        self.rectangles.normal.recalculate(window_size);
        self.rectangles.extract_dialog.recalculate(window_size);
        self.rectangles.confirm_dialog.recalculate(self.confirm_dialog.len, window_size);
    }

    // This contains the main UI loop.
    async fn run(mut self, ui_events_rx: UnboundedReceiver<NeedsRefresh>) {
        // X11/Wayland sends SIGWINCH when the window is resized
        let signals = Signals::new([SIGWINCH]).unwrap();
        let input = InputEvents::new();

        let input_stream = UnboundedReceiverStream::new(input.rx).map(UIEvent::Terminal);
        let backend_stream = UnboundedReceiverStream::new(EVENTS.take_rx().unwrap()).map(UIEvent::Backend);
        let ui_event_stream = UnboundedReceiverStream::new(ui_events_rx).map(UIEvent::Frontend);
        let signals_iter = signals.map(|_e| UIEvent::SigWinch);
        let mut event_stream = input_stream.merge(signals_iter).merge(backend_stream).merge(ui_event_stream);

        self.recalc_rects();

        while self.should_run {
            // set redraw_terminal to true if any of the widgets have changed
            // self = self.refresh_widgets().await;
            // Using the suggested ratatui draw method is probably redundant since the UI tracks state internally and only renders widgets when needed
            if let Some(e) = event_stream.next().await {
                match e {
                    UIEvent::Backend(event_source) => self.handle_backend_event(event_source).await,
                    UIEvent::Terminal(tick_event) => {
                        if let TickEvent::Input(i) = tick_event {
                            self.handle_input(i).await;
                        }
                    }
                    UIEvent::Frontend(needs_refresh) => {
                        let frame = &mut self.terminal.get_frame();
                        match needs_refresh {
                            NeedsRefresh::BottomBar => {
                                self.bottom_bar.update_widget(&self.tabs).await;
                                frame.render_widget(&self.bottom_bar.widget, self.rectangles.normal.bottom_bar)
                            }
                        }
                    }
                    UIEvent::SigWinch => {
                        self.recalc_rects();
                        // TODO render visible widgets
                    }
                }
            }
        }
    }

    async fn handle_backend_event(&mut self, event_source: EventSource) {
        let tabs = &mut self.tabs;
        match event_source {
            EventSource::Archives => {
                tabs.archive_table.refresh().await;
                self.render_if_visible(FocusedWidget::ArchiveTable);
            }
            EventSource::Downloads => {
                tabs.downloads_table.refresh().await;
                self.render_if_visible(FocusedWidget::DownloadTable);
            }
            EventSource::Installed => {
                tabs.installed_mods_table.refresh().await;
                self.render_if_visible(FocusedWidget::InstalledMods);
            }
            EventSource::Log => {
                tabs.log_list.refresh().await;
                self.render_if_visible(FocusedWidget::LogList);
            }
            EventSource::RequestCounter => {
                self.requests_widget.refresh().await;
            }
        }
    }

    fn render_if_visible(&mut self, focused: FocusedWidget) {
        for (i, widget_type) in self.tabs.focused_tab().widget_types.iter().enumerate() {
            if *widget_type == focused {
                self.terminal
                    .draw(|frame| {
                        self.tabs
                            .widget_for_type_mut(*widget_type)
                            .draw(self.rectangles.normal.main_content_panes[i], frame);
                    })
                    .unwrap();
                return;
            }
        }
    }

    pub fn render_active_widget(&mut self) {
        self.terminal
            .draw(|frame| {
                match self.input_mode {
                    InputMode::Normal => {
                        let rects = &self.rectangles.normal;
                        let widget_rect_pairs: &[(&dyn WidgetRef, Rect)] = &[
                            (&self.tabs.tab_display.widget, rects.tabs),
                            (&self.requests_widget.widget, rects.request_counter),
                            (&self.hotkey_bar.widget, rects.hotkey_bar),
                            (&self.bottom_bar.widget, rects.bottom_bar),
                        ];
                        for (widget, rect) in widget_rect_pairs.iter() {
                            widget.render_ref(*rect, frame.buffer_mut());
                        }

                        let widget_types = self.tabs.focused_tab().widget_types;
                        for (i, wt) in widget_types.iter().enumerate() {
                            let rect = self.rectangles.normal.main_content_panes[i];
                            self.tabs.widget_for_type_mut(*wt).draw(rect, frame);
                        }
                    }
                    InputMode::Extract => {
                        // TODO use same rendering logic as other widgets
                        // Clear the area so we can render on top of it
                        //frame.render_widget(Clear, rectangles.dialogpopup[0]);
                        //frame.render_widget(Clear, rectangles.dialogpopup[1]);
                        //
                        self.popup_dialog.render_widgets(&self.rectangles.extract_dialog, frame);
                        // frame.render_widget(&self.popup_dialog.text_label, self.rectangles.normal.dialog_popup_input_line[0]);
                        // frame.render_widget(&self.popup_dialog.textarea, self.rectangles.normal.dialog_popup_input_line[1]);
                        frame.render_widget(&self.hotkey_bar.widget, self.rectangles.normal.hotkey_bar);
                    }
                    InputMode::Confirm => {
                        // TODO why use clear here?
                        frame.render_widget(&self.confirm_dialog.widget, self.rectangles.confirm_dialog.rect);
                    }
                }
            })
            .unwrap();
    }

    // Returns true if self.redraw_terminal is true or any widget has changed
    // async fn refresh_widgets(mut self) -> Self {
    //     if self.nav.focused_tab() != TabType::Log && self.logger.has_changed.load(Ordering::Relaxed) {
    //         self.tabs.tab_display.add_urgency(TabType::Log);
    //     }
    //     self.redraw_terminal
    //         | match self.nav.selected().unwrap().into() {
    //             TabType::Archives => self.archives_table.refresh().await | self.downloads_table.update_widget().await,
    //             TabType::Installed => self.installed_mods_table.refresh().await,
    //             TabType::Log => self.log_list.update_widget().await,
    //         }
    //         | self.top_bar.refresh().await
    //         | self.hotkey_bar.refresh(&self.input_mode, self.nav.focused_widget_type()).await
    //         | self.bottom_bar.refresh().await;
    //     self
    // }
}

fn init_term() -> Result<Terminal<TermBackend>, Box<dyn Error>> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    /* The alternate screen restores terminal state when dropped.
     * Disable it if you need to see rust backtraces */
    let stdout = stdout.into_alternate_screen()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}
