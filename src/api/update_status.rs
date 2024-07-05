use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

pub const UP_TO_DATE: u8 = 0;
pub const HAS_NEW_FILE: u8 = 1;
pub const OUT_OF_DATE: u8 = 2;
pub const IGNORED_UNTIL: u8 = 3;
pub const INVALID: u8 = u8::MAX;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum UpdateStatus {
    UpToDate(u64),     // time of user's newest file,
    HasNewFile(u64),   // time of user's newest file
    OutOfDate(u64),    // time of user's newest file
    IgnoredUntil(u64), // time of latest file in update list
    Invalid(u64),
}

impl Default for UpdateStatus {
    fn default() -> Self {
        UpdateStatus::Invalid(0)
    }
}

impl UpdateStatus {
    pub fn time(&self) -> u64 {
        match self {
            Self::UpToDate(t) | Self::HasNewFile(t) | Self::OutOfDate(t) | Self::IgnoredUntil(t) | Self::Invalid(t) => {
                *t
            }
        }
    }
}

// Hack to retain backward compatibility with previously serialized data and provide a better API than two atomics
#[derive(Clone, Debug)]
pub struct UpdateStatusWrapper {
    status: Arc<AtomicU8>,
    time: Arc<AtomicU64>,
}

impl UpdateStatusWrapper {
    pub fn set(&self, update_status: UpdateStatus) {
        let (variant, time) = UpdateStatusWrapper::values_from_enum(update_status);
        self.status.store(variant, Ordering::Relaxed);
        self.time.store(time, Ordering::Relaxed);
    }

    pub fn new(status: UpdateStatus) -> Self {
        let (status, time) = Self::values_from_enum(status);
        Self {
            status: Arc::new(status.into()),
            time: Arc::new(time.into()),
        }
    }

    pub fn time(&self) -> u64 {
        self.time.load(Ordering::Relaxed)
    }

    pub fn to_enum(&self) -> UpdateStatus {
        let time = self.time.load(Ordering::Relaxed);
        match self.status.load(Ordering::Relaxed) {
            UP_TO_DATE => UpdateStatus::UpToDate(time),
            HAS_NEW_FILE => UpdateStatus::HasNewFile(time),
            IGNORED_UNTIL => UpdateStatus::IgnoredUntil(time),
            OUT_OF_DATE => UpdateStatus::OutOfDate(time),
            _ => UpdateStatus::Invalid(0),
        }
    }

    pub fn sync_with(&self, other: &Self) {
        if self.time.load(Ordering::Relaxed) < other.time.load(Ordering::Relaxed) {
            self.set(other.to_enum());
        } else {
            other.set(self.to_enum());
        }
    }

    fn values_from_enum(status: UpdateStatus) -> (u8, u64) {
        let (variant, time) = match status {
            UpdateStatus::UpToDate(t) => (UP_TO_DATE, t),
            UpdateStatus::HasNewFile(t) => (HAS_NEW_FILE, t),
            UpdateStatus::IgnoredUntil(t) => (IGNORED_UNTIL, t),
            UpdateStatus::OutOfDate(t) => (OUT_OF_DATE, t),
            UpdateStatus::Invalid(_) => (INVALID, 0),
        };
        (variant, time)
    }
}

impl Default for UpdateStatusWrapper {
    fn default() -> Self {
        Self::new(UpdateStatus::Invalid(0))
    }
}

impl From<UpdateStatus> for UpdateStatusWrapper {
    fn from(value: UpdateStatus) -> Self {
        Self::new(value)
    }
}

impl Serialize for UpdateStatusWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_enum().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UpdateStatusWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(UpdateStatus::deserialize(deserializer)?))
    }
}
