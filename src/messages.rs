use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct Messages {
    pub messages: Arc<RwLock<Vec<String>>>,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    len: Arc<AtomicUsize>,
}

impl Messages {
    // TODO allow optionally logging to file (maybe with log levels?)
    pub async fn push<S: Into<String>>(&self, msg: S) {
        self.messages.write().await.push(format!("{:?}: {}", self.len, msg.into()));
        self.has_changed.store(true, Ordering::Relaxed);
        self.len.fetch_add(1, Ordering::Relaxed);
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}
