use crate::events::EventSource;
use crate::ui::component::traits::FocusableComponent;
use crate::ui::tabs::FocusedWidget;
use ratatui::Frame;

use crate::ui::MainUI;

/// Maps EventSource to FocusedWidget and panics if it fails
