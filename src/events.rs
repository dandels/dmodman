use std::sync::RwLock;

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub enum EventSource {
    Archives,
    Downloads,
    Installed,
    Log,
    RequestCounter,
}

pub struct Events {
    pub tx: UnboundedSender<EventSource>,
    rx: RwLock<Option<UnboundedReceiver<EventSource>>>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: RwLock::new(Some(rx)),
        }
    }

    pub fn send(&self, e: EventSource) {
        let _ = self.tx.send(e);
    }

    pub fn take_rx(&self) -> Option<UnboundedReceiver<EventSource>> {
        self.rx.write().unwrap().take()
    }
}
