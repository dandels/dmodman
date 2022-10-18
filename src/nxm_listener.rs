use std::io::{Error, ErrorKind};
use std::str;
use tokio::io::Interest;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, mpsc::Receiver};
use tokio::task;

// Listens for downloads to add
struct NxmListener {
    listener: UnixListener,
}

impl NxmListener {
    pub fn new() -> Result<Self, Error> {
        let uid = users::get_current_uid();
        let listener = UnixListener::bind(&format!("/run/user/{}/dmodman.socket", uid))?;
        Ok(Self { listener })
    }
}

// Remove the socket when program quits
impl Drop for NxmListener {
    fn drop(&mut self) {
        let addr = self.listener.local_addr().unwrap();
        let path = addr.as_pathname().unwrap();
        let _ = std::fs::remove_file(path).unwrap();
    }
}

async fn handle_input(stream: UnixStream) -> Result<Option<String>, Error> {
    if stream.ready(Interest::READABLE).await?.is_readable() {
        let mut data = vec![0; 1024];
        match stream.try_read(&mut data) {
            Ok(_bytes) => {
                match str::from_utf8(&data) {
                    Ok(s) => return Ok(Some(s.to_string())),
                    Err(e) => {
                        println!("Invalid UTF-8 sequence: {}", e);
                        return Ok(None);
                    }
                };
            }
            // This is a false positive
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => return Err(e),
        }
    // Is this an error case?
    } else {
        println!("Nxm socket not readable");
        Ok(None)
    }
}

pub fn listen() -> Result<Receiver<Result<String, Error>>, Error> {
    // Channel capacity is arbitrarily chosen. It would be strange for a high number of downloads to be queued at once.
    let (tx, rx) = mpsc::channel(100);
    let socket = NxmListener::new()?;

    task::spawn(async move {
        loop {
            match socket.listener.accept().await {
                Ok((stream, _addr)) => match handle_input(stream).await {
                    Ok(opt_s) => {
                        if let Some(msg) = opt_s {
                            tx.send(Ok(msg)).await.unwrap();
                        }
                    }
                    Err(e) => {
                        tx.send(Err(e)).await.unwrap();
                    }
                },
                Err(e) => {
                    tx.send(Err(e)).await.unwrap();
                }
            }
        }
    });
    Ok(rx)
}

pub async fn connect() -> Result<UnixStream, Error> {
    let uid = users::get_current_uid();
    UnixStream::connect(&format!("/run/user/{}/dmodman.socket", uid)).await
}

pub async fn send_msg(stream: &UnixStream, msg: &[u8]) -> Result<(), Error> {
    loop {
        let ready = stream.ready(Interest::WRITABLE).await?;
        if ready.is_writable() {
            match stream.try_write(msg) {
                Ok(n) => {
                    println!("wrote {} bytes", n);
                    return Ok(());
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

pub fn remove_existing() -> Result<(), Error> {
    let uid = users::get_current_uid();
    let s = &format!("/run/user/{}/dmodman.socket", uid);
    let path = std::path::Path::new(s);
    let _ = std::fs::remove_file(path)?;
    Ok(())
}
