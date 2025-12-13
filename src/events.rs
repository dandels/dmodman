use crate::ui::TickEvent;
use std::sync::Arc;
use tokio::sync::{mpsc, mpsc::UnboundedReceiver, mpsc::UnboundedSender};

pub enum EventSource {
    Archives,
    Downloads,
    Files,
    Installed,
    Log,
    RequestCounter,
    Input(TickEvent),
}

pub fn create_channel() -> (EventTx, EventRx) {
    let (tx, rx) = mpsc::unbounded_channel();
    return (EventTx { tx: tx.into() }, EventRx { rx: rx.into() });
}

#[derive(Clone)]
// wrap this so changing it doesn't propagate across the entire codebase
pub struct EventTx {
    pub tx: Arc<UnboundedSender<EventSource>>,
}

impl EventTx {
    pub fn send(&self, msg: EventSource) {
        self.tx.send(msg).unwrap()
    }
}

pub struct EventRx {
    pub rx: Arc<UnboundedReceiver<EventSource>>,
}
