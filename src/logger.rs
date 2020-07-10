use super::config;
use chrono::Local;
use log::{LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init(loglevel: LevelFilter) -> Result<(), SetLoggerError> {
    static LOGGER: Logger = Logger {};
    log::set_logger(&LOGGER).map(|()| {
        log::set_max_level(loglevel);
    })
}

static WRITE_ERR: &str = "Unable to write to log file.";

//static lock: RwLock<Vec<String>> = RwLock::new(vec![]);

// TODO figure out how to deal with asynchronous logging

#[allow(dead_code)]
pub fn info(msg: &str) {
    // TODO implement checking of log level both via setting and command line argument
    append(&(time() + ": [INFO] - " + msg + "\n"));
}

#[allow(dead_code)]
pub fn err(msg: &str) {
    append(&(time() + ": [ERROR] - " + msg + "\n"));
}

pub fn append(msg: &str) {
    let log = log_file();
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log)
        .expect("");
    file.write_all(msg.as_bytes()).expect(WRITE_ERR);
}

fn time() -> String {
    let date = Local::now();
    let time = format!("{}", date.format("%Y-%m-%d %H:%M:%S"));
    time
}

fn log_file() -> PathBuf {
    let mut data_dir = config::log_dir();
    data_dir.push("log");
    data_dir
}
