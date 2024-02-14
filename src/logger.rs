use crate::config;
use std::fmt::{Debug, Display};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::fs::File;
use std::io::Write;
use std::sync::RwLock;

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
    pub fn log<S: Into<String> + Debug + Display>(&self, msg: S) {
        if !self.is_interactive {
            println!("{:?}", msg);
            return;
        }

        let mut lock = self.messages.write().unwrap();
        let len = lock.len();

        let mut path = config::config_dir();
        path.push("dmodman.log");
        let mut logfile = File::options().create(true).append(true).open(path).unwrap();
        logfile.write(format!("{}\n", msg).as_bytes()).unwrap();

        // TODO timestamp instead of number messages, but might require external crate to be sane
        lock.push(format!("{:?}: {}", len, msg.into()));
        self.has_changed.store(true, Ordering::Relaxed);
    }

    // Useful for testing UI code without causing re-rendering
    #[allow(dead_code)]
    pub fn log_to_file<S: Into<String> + Debug + Display>(&self, msg: S) {
        let mut path = config::config_dir();
        path.push("dmodman.log");
        let mut logfile = File::options().create(true).append(true).open(path).unwrap();
        logfile.write(format!("{}\n", msg).as_bytes()).unwrap();
    }

    pub async fn remove(&self, i: usize) {
        self.messages.write().unwrap().remove(i);
        self.has_changed.store(true, Ordering::Relaxed);
    }
}