use std::io;
use std::thread;
use termion::event::Event;
use termion::input::TermRead;
use tokio::sync::mpsc;

use std::time::Duration;

pub enum TickEvent {
    Input(Event),
    Tick,
}

pub struct Events {
    rx: mpsc::UnboundedReceiver<TickEvent>,
}

impl Events {
    pub fn new() -> Events {
        let tick_rate = Duration::from_millis(250);

        let (tx, rx) = mpsc::unbounded_channel();
        let _input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for event in stdin.events().flatten() {
                    if tx.send(TickEvent::Input(event)).is_err() {
                        return;
                    }
                }
            })
        };
        let _tick_handle = {
            thread::spawn(move || loop {
                if tx.send(TickEvent::Tick).is_err() {
                    break;
                }
                thread::sleep(tick_rate);
            })
        };
        Events { rx }
    }

    pub async fn next(&mut self) -> Option<TickEvent> {
        self.rx.recv().await
    }
}