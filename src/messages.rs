use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, RwLock,
};

#[derive(Clone, Default)]
pub struct Messages {
    pub messages: Arc<RwLock<Vec<String>>>,
    has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    len: Arc<AtomicUsize>,
}

impl Messages {
    pub fn push<S: Into<String>>(&self, msg: S) {
        self.messages.write().unwrap().push(msg.into());
        self.has_changed.store(true, Ordering::Relaxed);
        self.len.fetch_add(1, Ordering::Relaxed);
    }

    pub fn has_changed(&self) -> bool {
        let ret = self.has_changed.load(Ordering::Relaxed);
        self.has_changed.store(!self.has_changed.load(Ordering::Relaxed), Ordering::Relaxed);
        ret
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}
