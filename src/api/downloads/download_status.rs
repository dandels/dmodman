use crate::util::format;

pub struct DownloadStatus {
    pub file_name: String,
    pub file_id: u64,
    bytes_read: u64,
    size: String,
    size_unit: usize,
}

impl DownloadStatus {
    pub fn new(
        file_name: String,
        file_id: u64,
        bytes_read: u64,
        content_length: Option<u64>,
    ) -> Self {
        let size = match content_length {
            Some(total) => format::human_readable(total),
            None => ("?".to_string(), 3), // fall back to formatting size as mebibytes
        };
        Self {
            file_name,
            file_id,
            bytes_read,
            size: size.0,
            size_unit: size.1,
        }
    }

    /* TODO use some kind of task/thread parking to pause/continue downloads?:
     * - https://tokio-rs.github.io/tokio/doc/tokio/sync/struct.Notify.html
     * - https://doc.rust-lang.org/std/thread/fn.park.html
     */

    pub fn update_progress(&mut self, bytes: u64) {
        self.bytes_read += bytes;
    }

    pub fn progress(&self) -> String {
        format!(
            "{}/{}",
            format::bytes_as_unit(self.bytes_read, self.size_unit),
            self.size
        )
    }
}
