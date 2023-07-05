use crate::util::format;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct DownloadProgress {
    bytes_read: Arc<AtomicU64>,
    size: String,
    size_unit: usize,
}

impl DownloadProgress {
    pub fn new(bytes_read: Arc<AtomicU64>, content_length: Option<u64>) -> Self {
        let size = match content_length {
            Some(total) => format::human_readable(total),
            None => ("?".to_string(), 3), // fall back to formatting size as mebibytes
        };
        Self {
            bytes_read,
            size: size.0,
            size_unit: size.1,
        }
    }
}

impl fmt::Display for DownloadProgress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let print = format!(
            "{}/{}",
            format::bytes_as_unit(self.bytes_read.load(Ordering::Relaxed), self.size_unit),
            self.size
        );
        write!(f, "{}", print)
    }
}
