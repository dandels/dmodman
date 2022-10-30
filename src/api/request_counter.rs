use reqwest::header::HeaderMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Default)]
struct Counter {
    hourly_remaining: Option<u16>,
    daily_remaining: Option<u16>,
}

#[derive(Clone)]
pub struct RequestCounter {
    counter: Arc<RwLock<Counter>>,
    has_changed: Arc<AtomicBool>,
}

impl RequestCounter {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(RwLock::new(Counter::default())),
            has_changed: Arc::new(AtomicBool::from(true)),
        }
    }

    pub fn push(&mut self, headers: &HeaderMap) {
        let mut counter = self.counter.write().unwrap();
        if let Some(value) = headers.get("x-rl-daily-remaining") {
            (*counter).daily_remaining = value.to_str().map_or(None, |v| str::parse::<u16>(v).ok());
        }
        if let Some(value) = headers.get("x-rl-hourly-remaining") {
            (*counter).hourly_remaining = value.to_str().map_or(None, |v| str::parse::<u16>(v).ok());
        }
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub fn format(&self) -> String {
        let counter = self.counter.read().unwrap();
        format!(
            "Remaining | hourly: {} | daily: {}",
            counter.hourly_remaining.map_or_else(|| "NA".to_string(), |i| i.to_string()),
            counter.daily_remaining.map_or_else(|| "NA".to_string(), |i| i.to_string())
        )
    }

    pub fn has_changed(&self) -> bool {
        let ret = self.has_changed.load(Ordering::Relaxed);
        self.has_changed.store(false, Ordering::Relaxed);
        ret
    }
}
