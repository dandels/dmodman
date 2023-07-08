use crate::util::format;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct DownloadProgress {
    pub bytes_read: Arc<AtomicU64>,
    pub size: String,
    size_unit: usize,
}

impl DownloadProgress {
    pub fn new(bytes_read: Arc<AtomicU64>, content_length: Option<u64>) -> Self {
        let size = match content_length {
            Some(total) => format::human_readable(total),
            None => ("?".to_string(), 3),
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
        let print =
            format!("{}/{}", format::bytes_as_unit(self.bytes_read.load(Ordering::Relaxed), self.size_unit), self.size);
        write!(f, "{}", print)
    }
}
