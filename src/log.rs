use super::config;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

static WRITE_ERR: &str = "Unable to write to log file.";

pub fn info(msg: &str) {
    append(&(time() + ": [INFO] - " + msg + "\n"));
}

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
