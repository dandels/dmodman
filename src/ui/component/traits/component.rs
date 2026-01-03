use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidgetRef;
// use ratatui::widgets::WidgetRef;
use ratatui::Frame;

use crate::ui::component::traits::{Highlight, Select};
use crate::ui::component::*;

macro_rules! impl_focusable_component {
    ($T:ty) => {
        impl FocusableComponent for $T {
            fn draw(&mut self, rect: Rect, frame: &mut Frame) {
                // TODO it's unclear whether implementing Widget for a StatefulWidget would suffice
                StatefulWidgetRef::render_ref(&self.widget, rect, frame.buffer_mut(), &mut self.state)
            }
        }
    };
}

impl_focusable_component!(ArchivesWidget<'_>);
impl_focusable_component!(DownloadsWidget<'_>);
impl_focusable_component!(InstalledModsWidget<'_>);
impl_focusable_component!(LogWidget<'_>);
// impl_stateful_component!(ConfirmDialog<'_>);

// macro_rules! impl_component {
//     ($T:ty) => {
//         impl Component for $T {
//             fn draw(&mut self, rect: Rect, frame: &mut Frame) {
//                 WidgetRef::render_ref(&self.widget, rect, frame.buffer_mut())
//             }
//         }
//     };
// }

// impl_component!(BottomBar<'_>);
// impl_component!(HotkeyBar<'_>);
// impl_component!(TabDisplay<'_>);
// impl_component!(RequestCounterWidget<'_>);

/// Helper trait for rendering a list of StatefulWidgets (to avoid passing state around)
pub trait FocusableComponent: Highlight + Select {
    fn draw(&mut self, rect: Rect, frame: &mut Frame);
}
