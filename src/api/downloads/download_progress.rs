use crate::util::format;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Deserialize, Serialize)]
pub struct DownloadProgress {
    pub bytes_read: Arc<AtomicU64>,
    size_and_unit: Option<(String, usize)>,
}

impl DownloadProgress {
    pub fn new(bytes_read: Arc<AtomicU64>, content_length: Option<u64>) -> Self {
        let size_and_unit = content_length.map(format::human_readable);
        Self {
            bytes_read,
            size_and_unit,
        }
    }
}

impl fmt::Display for DownloadProgress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.size_and_unit {
            Some((size, size_unit)) => write!(f, "{}/{size}", format::bytes_as_unit(self.bytes_read.load(Ordering::Relaxed), *size_unit)),
            None => f.write_str("")
        }
    }
}
