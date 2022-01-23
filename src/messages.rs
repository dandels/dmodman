use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, RwLock,
};

#[derive(Clone, Default)]
pub struct Messages {
    pub messages: Arc<RwLock<Vec<String>>>,
    is_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    len: Arc<AtomicUsize>,
}

impl Messages {
    pub fn push(&self, msg: String) {
        self.messages.write().unwrap().push(msg);
        self.is_changed.store(true, Ordering::Relaxed);
        self.len.fetch_add(1, Ordering::Relaxed);
    }

    pub fn is_changed(&self) -> bool {
        let ret = self.is_changed.load(Ordering::Relaxed);
        self.is_changed
            .store(!self.is_changed.load(Ordering::Relaxed), Ordering::Relaxed);
        ret
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len.load(Ordering::Relaxed) == 0
    }
}
