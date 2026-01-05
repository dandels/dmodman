use std::io::Stdout;

use super::component::*;
use crate::api::{Downloads, Query, UpdateChecker};
use crate::db::Db;
use crate::events::EventSource;
use crate::extract::Installer;
use crate::prelude::*;
use crate::ui::rectangles::Rectangles;
use crate::ui::tabs::*;
use crate::ui::*;
use crate::Lib;
use ratatui::prelude::{Backend, TermionBackend};
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use termion::screen::IntoAlternateScreen;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Debug)]
pub enum NeedsRefresh {
    BottomBar,
}

#[derive(Debug)]
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
    pub terminal: Terminal<TermBackend>,
    pub rectangles: Rectangles,
    pub ui_events_tx: mpsc::UnboundedSender<NeedsRefresh>,
    pub input_mode: InputMode,
    pub should_run: bool,
}

impl<'a> MainUI<'a> {
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
        ui.run(ui_events_rx).await;
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
        self.render_all_visible();
        // self.render_active_widget();

        while self.should_run {
            // set redraw_terminal to true if any of the widgets have changed
            // self = self.refresh_widgets().await;
            // Using the suggested ratatui draw method is probably redundant since the UI tracks state internally and only renders widgets when needed
            if let Some(e) = event_stream.next().await {
                match e {
                    UIEvent::Terminal(tick_event) => match tick_event {
                        TickEvent::Tick => continue,
                        TickEvent::Input(i) => self.handle_input(i).await,
                    },
                    UIEvent::Backend(event_source) => self.handle_backend_event(event_source).await,
                    UIEvent::Frontend(needs_refresh) => match needs_refresh {
                        NeedsRefresh::BottomBar => {
                            self.bottom_bar.update_widget(&self.tabs).await;
                        }
                    },
                    UIEvent::SigWinch => {
                        self.recalc_rects();
                    }
                }
                // TODO some input events don't require re-rendering
                self.render_all_visible();
            }
        }
    }

    async fn handle_backend_event(&mut self, event_source: EventSource) {
        if let EventSource::RequestCounter = event_source {
            self.requests_widget.refresh().await;
        } else {
            let focused = focusedwidget_from_event(event_source);
            self.refresh_component(focused).await;
            let i = tabs::INDEX_MAPPING[focused].tab;
            if self.tabs.focused_index != i {
                self.tabs.tab_display.add_urgency(i);
            }
        }
        self.render_all_visible();
    }

    pub fn render_all_visible(&mut self) {
        let frame = &mut self.terminal.get_frame();
        match self.input_mode {
            InputMode::Normal => {
                frame.render_widget(&self.requests_widget.widget, self.rectangles.normal.request_counter);
                frame.render_widget(&self.hotkey_bar.widget, self.rectangles.normal.hotkey_bar);
                frame.render_widget(&self.tabs.tab_display.widget, self.rectangles.normal.tabs);
                frame.render_widget(&self.bottom_bar.widget, self.rectangles.normal.bottom_bar);

                let widget_types = self.tabs.focused_tab().widget_types;
                for (i, wt) in widget_types.iter().enumerate() {
                    let rect = self.rectangles.normal.main_content_panes[i];
                    self.tabs.widget_for_type_mut(*wt).draw(rect, frame.buffer_mut());
                }
            }
            InputMode::Confirm => todo!(),
            InputMode::Extract => {
                self.render_extract_dialog();
            }
        }
        self.flush_terminal();
    }

    pub fn render_active_widget(&mut self) {
        let frame = &mut self.terminal.get_frame();
        match self.input_mode {
            InputMode::Normal => {
                let i = self.tabs.focused_tab().focused_widget_index;
                self.tabs
                    .focused_widget_mut()
                    .draw(self.rectangles.normal.main_content_panes[i], frame.buffer_mut());
            }
            InputMode::Extract => {
                frame.render_widget(&self.popup_dialog.textbox, self.rectangles.extract_dialog.textbox);
            }
            InputMode::Confirm => {
                frame.render_widget(&self.confirm_dialog.widget, self.rectangles.confirm_dialog.rect);
            }
        }
    }

    fn render_extract_dialog(&mut self) {
        let frame = &mut self.terminal.get_frame();
        self.popup_dialog.render_widgets(&self.rectangles.extract_dialog, frame);
        frame.render_widget(&self.hotkey_bar.widget, self.rectangles.normal.hotkey_bar);
    }

    async fn refresh_component(&mut self, focused: FocusedWidget) {
        match focused {
            FocusedWidget::ArchiveTable => self.tabs.archive_table.refresh().await,
            FocusedWidget::DownloadTable => self.tabs.downloads_table.refresh().await,
            FocusedWidget::InstalledMods => self.tabs.installed_mods_table.refresh().await,
            FocusedWidget::LogList => self.tabs.log_list.refresh().await,
        }
    }

    pub fn flush_terminal(&mut self) {
        let terminal = &mut self.terminal;
        terminal.flush().unwrap();
        terminal.swap_buffers();
        terminal.backend_mut().flush().unwrap();
    }
}

fn focusedwidget_from_event(event: EventSource) -> FocusedWidget {
    match event {
        EventSource::Archives => FocusedWidget::ArchiveTable,
        EventSource::Downloads => FocusedWidget::DownloadTable,
        EventSource::Installed => FocusedWidget::InstalledMods,
        EventSource::Log => FocusedWidget::LogList,
        EventSource::RequestCounter => unreachable!("Function invariant violated."),
    }
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
