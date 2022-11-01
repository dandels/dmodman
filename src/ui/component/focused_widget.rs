use super::*;
use super::{Highlight, Select};

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
            Self::DownloadTable(x) => x.write().await.next(),
            Self::FileTable(x) => x.write().await.next(),
            Self::MessageList(x) => x.write().await.next(),
        }
    }
    pub async fn previous(&mut self) {
        match self {
            Self::DownloadTable(x) => x.write().await.previous(),
            Self::FileTable(x) => x.write().await.previous(),
            Self::MessageList(x) => x.write().await.previous(),
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
