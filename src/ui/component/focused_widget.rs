use super::*;
use super::{Highlight, Select};

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::sync::RwLock;

#[derive(Clone)]
pub enum FocusedWidget<'a> {
    DownloadTable(Arc<RwLock<DownloadTable<'a>>>),
    FileTable(Arc<RwLock<FileTable<'a>>>),
    MessageList(Arc<RwLock<MessageList<'a>>>),
}

/* I couldn't figure out how to avoid copypasting here. All the enum members implement the Highlight and Select
 * trait, which have the methods we need here. */
impl<'a> FocusedWidget<'a> {
    pub async fn change_to(&mut self, mut selected: FocusedWidget<'a>) {
        match self {
            FocusedWidget::DownloadTable(current) => {
                current.write().await.unfocus().await;
                selected.focus().await;
                *self = selected;
            }
            FocusedWidget::FileTable(current) => {
                current.write().await.unfocus().await;
                selected.focus().await;
                *self = selected;
            }
            FocusedWidget::MessageList(current) => {
                current.write().await.unfocus().await;
                selected.focus().await;
                *self = selected;
            }
        }
    }

    pub async fn next(&mut self) {
        match self {
            Self::DownloadTable(dt) => {
                let mut table_lock = dt.write().await;
                let dls = table_lock.downloads.clone();
                let tasks_lock = dls.tasks.read().await;
                table_lock.next(tasks_lock.len());
                table_lock.needs_redraw.store(true, Ordering::Relaxed);
            }
            Self::FileTable(ft) => {
                let mut table_lock = ft.write().await;
                let file_index = table_lock.file_index.clone();
                let files_lock = file_index.file_id_map.read().await;
                table_lock.next(files_lock.len());
                table_lock.needs_redraw.store(true, Ordering::Relaxed);
            }
            Self::MessageList(ml) => {
                let mut list_lock = ml.write().await;
                let msgs = list_lock.msgs.clone();
                let msgs_lock = msgs.messages.read().await;
                list_lock.next(msgs_lock.len());
                list_lock.needs_redraw.store(true, Ordering::Relaxed);
            }
        }
    }
    pub async fn previous(&mut self) {
        match self {
            Self::DownloadTable(dt) => {
                let mut table_lock = dt.write().await;
                let dls = table_lock.downloads.clone();
                let tasks_lock = dls.tasks.read().await;
                table_lock.previous(tasks_lock.len());
                table_lock.needs_redraw.store(true, Ordering::Relaxed);
            }
            Self::FileTable(ft) => {
                let mut table_lock = ft.write().await;
                let file_index = table_lock.file_index.clone();
                let files_lock = file_index.file_id_map.read().await;
                table_lock.previous(files_lock.len());
                table_lock.needs_redraw.store(true, Ordering::Relaxed);
            }
            Self::MessageList(ml) => {
                let mut list_lock = ml.write().await;
                let msgs = list_lock.msgs.clone();
                let msgs_lock = msgs.messages.read().await;
                list_lock.previous(msgs_lock.len());
                list_lock.needs_redraw.store(true, Ordering::Relaxed);
            }
        }
    }

    pub async fn focus(&mut self) {
        match self {
            FocusedWidget::DownloadTable(current) => {
                current.write().await.focus().await;
            }
            FocusedWidget::FileTable(current) => {
                current.write().await.focus().await;
            }
            FocusedWidget::MessageList(current) => {
                current.write().await.focus().await;
            }
        }
    }
}
