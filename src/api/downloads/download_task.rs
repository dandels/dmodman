use super::{ApiError, DownloadProgress};
use tokio::task::JoinHandle;

pub enum DownloadState {
    Finished,
    Paused,
    Running,
}

pub struct DownloadTask {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub file_name: String,
    pub progress: DownloadProgress,
    pub state: DownloadState,
    join_handle: JoinHandle<Result<(), ApiError>>,
}

impl DownloadTask {
    pub fn new(
        game: String,
        mod_id: u32,
        file_id: u64,
        file_name: String,
        progress: DownloadProgress,
        join_handle: JoinHandle<Result<(), ApiError>>,
    ) -> Self {
        Self {
            game,
            mod_id,
            file_id,
            file_name,
            progress,
            state: DownloadState::Running,
            join_handle,
        }
    }

    pub async fn toggle_pause(&mut self) {
        match self.state {
            DownloadState::Running => {
                self.join_handle.abort();
                self.state = DownloadState::Paused;
            }
            DownloadState::Paused => {
                // TODO resume download
            }
            DownloadState::Finished => {}
        }
    }
}
