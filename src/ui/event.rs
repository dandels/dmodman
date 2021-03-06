use std::io;
use std::sync::mpsc;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;

use std::time::Duration;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
}

impl Events {
    pub fn new() -> Events {
        let tick_rate = Duration::from_millis(250);

        let (tx, rx) = mpsc::channel();
        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for key in stdin.keys().into_iter().flatten() {
                    if tx.send(Event::Input(key)).is_err() {
                        return;
                    }
                }
            })
        };
        let _tick_handle = {
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    if tx.send(Event::Tick).is_err() {
                        break;
                    }
                    thread::sleep(tick_rate);
                }
            })
        };
        Events { rx }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
