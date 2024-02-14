use crate::config;
use std::fmt::{Debug, Display};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct Logger {
    pub messages: Arc<RwLock<Vec<String>>>,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    is_interactive: bool,
}

impl Logger {
    pub fn new(is_interactive: bool) -> Self {
        Self {
            is_interactive,
            ..Default::default()
        }
    }

    // TODO allow optionally logging to file (maybe with log levels?)
    pub async fn log<S: Into<String> + Debug + Display>(&self, msg: S) {
        if !self.is_interactive {
            println!("{:?}", msg);
            return;
        }

        let mut lock = self.messages.write().await;
        let len = lock.len();

        let mut path = config::config_dir();
        path.push("dmodman.log");
        let mut logfile = File::create(path).await.unwrap();
        logfile.write_all(msg.to_string().as_bytes()).await.unwrap();

        // TODO timestamp instead of number messages, but might require external crate to be sane
        lock.push(format!("{:?}: {}", len, msg.into()));
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn remove(&self, i: usize) {
        self.messages.write().await.remove(i);
        self.has_changed.store(true, Ordering::Relaxed);
    }
}