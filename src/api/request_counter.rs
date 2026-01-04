use crate::prelude::*;
use reqwest::header::HeaderMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Counter {
    pub hourly_remaining: Option<u16>,
    pub daily_remaining: Option<u16>,
}

#[derive(Clone)]
pub struct RequestCounter {
    pub counter: Arc<RwLock<Counter>>,
}

impl RequestCounter {
    pub fn new() -> Self {
        Self {
            counter: Default::default(),
        }
    }

    // TODO race condition when many requests are made at once
    pub async fn push(&self, headers: &HeaderMap) {
        let mut counter = self.counter.write().await;
        if let Some(value) = headers.get("x-rl-daily-remaining") {
            counter.daily_remaining = value.to_str().map_or(None, |v| str::parse::<u16>(v).ok());
        }
        if let Some(value) = headers.get("x-rl-hourly-remaining") {
            counter.hourly_remaining = value.to_str().map_or(None, |v| str::parse::<u16>(v).ok());
        }
        EVENTS.send(EventSource::RequestCounter)
    }
}
