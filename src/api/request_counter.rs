use reqwest::header::HeaderMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::events::EventSource;
use crate::events::EventTx;

#[derive(Debug, Default)]
pub struct Counter {
    pub hourly_remaining: Option<u16>,
    pub daily_remaining: Option<u16>,
}

#[derive(Clone)]
pub struct RequestCounter {
    pub counter: Arc<RwLock<Counter>>,
    event_tx: EventTx,
}

impl RequestCounter {
    pub fn new(event_tx: EventTx) -> Self {
        Self {
            counter: Default::default(),
            event_tx,
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
        self.event_tx.send(EventSource::RequestCounter)
    }
}
