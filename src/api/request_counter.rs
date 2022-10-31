use reqwest::header::HeaderMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
struct Counter {
    hourly_remaining: Option<u16>,
    daily_remaining: Option<u16>,
}

#[derive(Clone)]
pub struct RequestCounter {
    counter: Arc<RwLock<Counter>>,
    pub has_changed: Arc<AtomicBool>,
}

impl RequestCounter {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(RwLock::new(Counter::default())),
            has_changed: Arc::new(AtomicBool::from(true)),
        }
    }

    // TODO race condition when many requests are made at once
    pub async fn push(&mut self, headers: &HeaderMap) {
        let mut counter = self.counter.write().await;
        if let Some(value) = headers.get("x-rl-daily-remaining") {
            (*counter).daily_remaining = value.to_str().map_or(None, |v| str::parse::<u16>(v).ok());
        }
        if let Some(value) = headers.get("x-rl-hourly-remaining") {
            (*counter).hourly_remaining = value.to_str().map_or(None, |v| str::parse::<u16>(v).ok());
        }
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn format(&self) -> String {
        let counter = self.counter.read().await;
        format!(
            "Remaining | hourly: {} | daily: {}",
            counter.hourly_remaining.map_or_else(|| "NA".to_string(), |i| i.to_string()),
            counter.daily_remaining.map_or_else(|| "NA".to_string(), |i| i.to_string())
        )
    }
}
