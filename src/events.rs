use std::sync::Arc;
use tokio::sync::{mpsc, mpsc::UnboundedReceiver, mpsc::UnboundedSender};

pub enum EventSource {
    Archives,
    Downloads,
    Installed,
    Log,
    RequestCounter,
}

pub struct Events {
    pub tx: EventTx,
    pub rx: EventRx,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx: EventTx { inner: tx.into() },
            rx: EventRx { inner: rx },
        }
    }
}

#[derive(Clone)]
// wrap this so changing it doesn't propagate across the entire codebase
pub struct EventTx {
    pub inner: Arc<UnboundedSender<EventSource>>,
}

impl EventTx {
    pub fn send(&self, msg: EventSource) {
        self.inner.send(msg).unwrap()
    }
}

pub struct EventRx {
    pub inner: UnboundedReceiver<EventSource>,
}
