use tokio::io::Interest;
use tokio::net::{UnixListener, UnixStream};
use tokio::task;
use std::str;
use std::io::Error;

// Listens for downloads to add
struct NxmSocket {
    listener: UnixListener
}

impl NxmSocket {
    pub fn new() -> Result<Self, Error> {
        let listener = UnixListener::bind(&format!("/run/user/{}/dmodman.socket", users::get_current_uid()))?;
        Ok(Self { listener })
    }
}

// Remove the socket when program quits
impl Drop for NxmSocket {
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
                    Ok(s) => {
                        println!("{}", s);
                        return Ok(Some(s.to_string()))
                    }
                    Err(e) => {
                        println!("Invalid UTF-8 sequence: {}", e);
                        return Ok(None)
                    }
                };
            }
            // This is a false positive
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => return Err(e)
        }
    // Is this an error case?
    } else {
        println!("Nxm socket not readable");
        Ok(None)
    }
}

pub fn listen() -> task::JoinHandle<Result<Option<String>, Error>> {
    let join_handle: task::JoinHandle<Result<Option<String>, Error>>  = task::spawn(async {
        let socket = NxmSocket::new()?;
        loop {
            match socket.listener.accept().await {
                Ok((stream, _addr)) => {
                    println!("new client!");
                    handle_input(stream).await?;
                }
                Err(e) => return Err(e)
            }
        }
    });
    join_handle
}
