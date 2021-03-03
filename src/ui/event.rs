use std::io;
use std::sync::mpsc;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;

use std::time::Duration;

// This file is black magic from the tui termion example.

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(1000),
        }
    }
}

pub enum Event<I> {
    Input(I),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key>>,
    #[allow(dead_code)]
    input_handle: thread::JoinHandle<()>,
    #[allow(dead_code)]
    tick_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for key in stdin.keys().into_iter().flatten() {
                    if tx.send(Event::Input(key)).is_err() {
                        return;
                    }
                    if key == config.exit_key {
                        return;
                    }
                }
            })
        };
        let tick_handle = {
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    if tx.send(Event::Tick).is_err() {
                        break;
                    }
                    thread::sleep(config.tick_rate);
                }
            })
        };
        Events {
            rx,
            input_handle,
            tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }
}
