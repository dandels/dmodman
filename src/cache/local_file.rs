use crate::api::downloads::FileInfo;
use crate::cache::Cacheable;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

const UP_TO_DATE: u8 = 0;
const HAS_NEW_FILE: u8 = 1;
const IGNORED_UNTIL: u8 = 3;
const OUT_OF_DATE: u8 = 2;

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalFile {
    pub game: String,
    pub file_name: String,
    pub mod_id: u32,
    pub file_id: u64,
    update_status: StatusWrapper, // uses atomics internally but serializes to UpdateStatus enum
}

impl LocalFile {
    pub fn new(fi: FileInfo, update_status: UpdateStatus) -> Self {
        LocalFile {
            game: fi.game,
            file_name: fi.file_name,
            mod_id: fi.mod_id,
            file_id: fi.file_id,
            update_status: StatusWrapper::new(update_status),
        }
    }

    pub fn update_status(&self) -> UpdateStatus {
        self.update_status.to_enum()
    }

    pub fn set_update_status(&self, update_status: UpdateStatus) {
        let (variant, time) = StatusWrapper::values_from_enum(update_status);
        self.update_status.status.store(variant, Ordering::Relaxed);
        self.update_status.time.store(time, Ordering::Relaxed);
    }
}

#[derive(Debug)]
struct StatusWrapper {
    pub status: AtomicU8,
    pub time: AtomicU64,
}

impl StatusWrapper {
    pub fn new(status: UpdateStatus) -> Self {
        let (status, time) = Self::values_from_enum(status);
        Self {
            status: status.into(),
            time: time.into(),
        }
    }

    pub fn to_enum(&self) -> UpdateStatus {
        let time = self.time.load(Ordering::Relaxed);
        match self.status.load(Ordering::Relaxed) {
            UP_TO_DATE => UpdateStatus::UpToDate(time),
            HAS_NEW_FILE => UpdateStatus::HasNewFile(time),
            IGNORED_UNTIL => UpdateStatus::IgnoredUntil(time),
            OUT_OF_DATE => UpdateStatus::OutOfDate(time),
            _ => panic!("Invalid enum variant for UpdateStatus"),
        }
    }

    fn values_from_enum(status: UpdateStatus) -> (u8, u64) {
        let (variant, time) = match status {
            UpdateStatus::UpToDate(t) => (UP_TO_DATE, t),
            UpdateStatus::HasNewFile(t) => (HAS_NEW_FILE, t),
            UpdateStatus::IgnoredUntil(t) => (IGNORED_UNTIL, t),
            UpdateStatus::OutOfDate(t) => (OUT_OF_DATE, t),
        };
        (variant, time)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum UpdateStatus {
    UpToDate(u64),     // time of user's newest file,
    HasNewFile(u64),   // time of user's newest file
    OutOfDate(u64),    // time of user's newest file
    IgnoredUntil(u64), // time of latest file in update list
}

impl Cacheable for LocalFile {}

impl Serialize for StatusWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer, {
        self.to_enum().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StatusWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>, {
        Ok(Self::new(UpdateStatus::deserialize(deserializer)?))
    }
}

impl UpdateStatus {
    pub fn time(&self) -> u64 {
        match self {
            Self::UpToDate(t) | Self::HasNewFile(t) | Self::OutOfDate(t) | Self::IgnoredUntil(t) => *t,
        }
    }
}
