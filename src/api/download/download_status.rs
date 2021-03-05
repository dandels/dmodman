use crate::utils;

pub struct DownloadStatus {
    pub file_name: String,
    pub file_id: u64,
    bytes_read: u64,
    size: String,
}

impl DownloadStatus {
    pub fn new(file_name: String, file_id: u64, content_length: Option<u64>) -> Self {
        let size = match content_length {
            Some(total) => utils::human_readable(total),
            None => "NIL".to_string(),
        };
        Self {
            file_name,
            file_id,
            bytes_read: 0,
            size,
        }
    }

    /* TODO use some kind of task/thread parking to pause/continue downloads?:
     * - https://tokio-rs.github.io/tokio/doc/tokio/sync/struct.Notify.html
     * - https://doc.rust-lang.org/std/thread/fn.park.html
     *
     * The HTTP range header might be required for cold-resuming downloads, which might also mean we don't need to park
     * threads. The easiest is probably simply stopping downloads and calculating their size in bytes when resuming.
     */

    pub fn update_progress(&mut self, bytes: u64) {
        self.bytes_read += bytes;
    }

    pub fn progress(&self) -> String {
        format!(
            "{}/{}",
            utils::human_readable_without_unit(self.bytes_read),
            self.size
        )
    }
}
