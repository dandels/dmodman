use std::io;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;
use tokio::sync::mpsc;

use std::time::Duration;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::UnboundedReceiver<Event<Key>>,
}

impl Events {
    pub fn new() -> Events {
        let tick_rate = Duration::from_millis(250);

        let (tx, rx) = mpsc::unbounded_channel();
        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for key in stdin.keys().flatten() {
                    if tx.send(Event::Input(key)).is_err() {
                        return;
                    }
                }
            })
        };
        let _tick_handle = {
            thread::spawn(move || loop {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                thread::sleep(tick_rate);
            })
        };
        Events { rx }
    }

    pub async fn next(&mut self) -> Option<Event<Key>> {
        self.rx.recv().await
    }
}
