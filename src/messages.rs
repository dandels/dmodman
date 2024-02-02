use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct Messages {
    pub messages: Arc<RwLock<Vec<String>>>,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    is_interactive: bool,
}

impl Messages {
    pub fn new(is_interactive: bool) -> Self {
        Self {
            is_interactive,
            ..Default::default()
        }
    }

    // TODO allow optionally logging to file (maybe with log levels?)
    pub async fn push<S: Into<String> + std::fmt::Debug>(&self, msg: S) {
        if !self.is_interactive {
            println!("{:?}", msg);
            return;
        }

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
