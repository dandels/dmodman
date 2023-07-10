use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct Messages {
    pub messages: Arc<RwLock<Vec<String>>>,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
}

impl Messages {
    // TODO allow optionally logging to file (maybe with log levels?)
    pub async fn push<S: Into<String>>(&self, msg: S) {
        let mut lock = self.messages.write().await;
        let len = lock.len();
        lock.push(format!("{:?}: {}", len, msg.into()));
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn remove(&self, i: usize) {
        self.messages.write().await.remove(i);
        self.has_changed.store(true, Ordering::Relaxed);
    }
}
