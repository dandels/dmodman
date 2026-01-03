use crate::config;
use crate::events::EventTx;
use std::fmt::{Debug, Display};
use std::sync::Arc;

use std::fs::File;
use std::io::Write;
use std::sync::RwLock;

#[derive(Clone)]
pub struct Logger {
    pub messages: Arc<RwLock<Vec<String>>>,
    event_tx: EventTx, // used by UI to ask if error list needs to be redrawn
    is_interactive: bool,
}

#[cfg(test)]
impl Default for Logger {
    fn default() -> Self {
        // This is silly
        let event_tx = crate::events::Events::new().tx;
        Self {
            messages: Default::default(),
            event_tx,
            is_interactive: Default::default(),
        }
    }
}

impl Logger {
    pub fn new(event_tx: EventTx, is_interactive: bool) -> Self {
        Self {
            messages: Default::default(),
            event_tx,
            is_interactive,
        }
    }

    // TODO allow optionally logging to file (maybe with log levels?)
    pub fn log<S: Into<String> + Debug + Display>(&self, msg: S) {
        if !self.is_interactive {
            println!("{}", msg);
            return;
        }

        let mut path = config::config_dir();
        path.push("dmodman.log");
        let mut logfile = File::options().create(true).append(true).open(path).unwrap();
        // TODO maybe only do this if configured to
        logfile.write_all(format!("{}\n", msg).as_bytes()).unwrap();

        // TODO timestamp messages, but might require external crate
        self.messages.write().unwrap().push(msg.to_string());
        self.event_tx.send(crate::events::EventSource::Log);
    }

    // No longer needed since the UI drains the log and maintains internal list
    #[allow(dead_code)]
    pub async fn remove(&self, i: usize) {
        let mut lock = self.messages.write().unwrap();
        if !lock.is_empty() {
            lock.remove(i);
            self.event_tx.send(crate::events::EventSource::Log);
        }
    }
}

// Useful for testing UI code without causing re-rendering
#[allow(dead_code)]
pub fn log_to_file<S: Into<String> + Debug + Display>(msg: S) {
    let mut path = config::config_dir();
    path.push("dmodman.log");
    let mut logfile = File::options().create(true).append(true).open(path).unwrap();
    logfile.write_all(format!("{}\n", msg).as_bytes()).unwrap();
}
