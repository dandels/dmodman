use std::io::{Error, ErrorKind};
use std::str;

use tokio::io::Interest;
use tokio::net::{UnixListener, UnixStream};
use tokio::task;

use crate::api::Downloads;
use crate::Logger;

// Listens for nxm:// urls to queue as downloads
pub struct NxmSocketListener {
    listener: UnixListener, // Wrapped into a struct so we can impl Drop on it
}

impl NxmSocketListener {
    fn bind() -> Result<Self, Error> {
        Ok(Self {
            listener: UnixListener::bind(get_socket_path())?,
        })
    }
}

impl Drop for NxmSocketListener {
    fn drop(&mut self) {
        remove_socket().unwrap()
    }
}

pub async fn try_bind() -> Result<NxmSocketListener, Error> {
    match NxmSocketListener::bind() {
        Ok(listener) => Ok(listener),
        Err(ref e) if e.kind() == ErrorKind::AddrInUse => {
            // Even if the socket address is in use, we can't know if it's responding without trying to connect
            match connect().await {
                // Another running instance is accepting connections
                Ok(_stream) => Err(ErrorKind::AddrInUse.into()),
                // Socket probably hasn't been cleanly removed. Remove it and bind to it.
                Err(ref e) if e.kind() == ErrorKind::ConnectionRefused => {
                    println!(
                        "Previous socket {} exists but is refusing connections. \
                        dmodman might not have shut down cleanly. Removing it...",
                        get_socket_path()
                    );
                    remove_socket()?;
                    // Retry bind() and return whatever the result is
                    NxmSocketListener::bind()
                }
                /* Catch-all for unanticipated ways in which the socket can break.
                 * Hitting this case should be unlikely. */
                Err(e) => panic!("Binding to dmodman socket failed in unexpected way: {}", e),
            }
        }
        Err(e) => panic!("Binding to dmodman socket failed in unexpected way: {}", e),
    }
}

pub async fn listen_for_downloads(nxm_sock: NxmSocketListener, downloads: Downloads, logger: Logger) {
    task::spawn(async move {
        loop {
            match nxm_sock.listener.accept().await {
                Ok((stream, _addr)) => {
                    if let Ok(ready) = stream.ready(Interest::READABLE).await {
                        if ready.is_readable() {
                            handle_incoming_stream(stream, &downloads, &logger).await;
                        }
                    } // It doesn't seem like the two else {} paths here require dealing with
                }
                Err(e) => {
                    logger.log(format!("nxm socket was unable to accept connection: {}", e));
                }
            }
        }
    });
}

async fn handle_incoming_stream(stream: UnixStream, downloads: &Downloads, logger: &Logger) {
    let mut data = vec![0; 1024];
    match stream.try_read(&mut data) {
        Ok(_bytes) => match str::from_utf8(&data) {
            Ok(msg) => {
                if msg.starts_with("nxm://") {
                    downloads.try_queue(msg).await;
                }
            }
            Err(e) => {
                logger.log(format!("nxm socket received invalid UTF-8 sequence: {}", e));
            }
        },
        // is_readable returned a false positive
        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
        Err(e) => {
            logger.log(format!("nxm socket encountered error: {}", e));
        }
    }
}

fn get_socket_path() -> String {
    extern "C" {
        fn getuid() -> u32;
    }
    let uid;
    unsafe { uid = getuid() }
    format!("/run/user/{}/dmodman.socket", uid)
}

fn remove_socket() -> Result<(), Error> {
    std::fs::remove_file(get_socket_path())
}

async fn connect() -> Result<UnixStream, Error> {
    UnixStream::connect(get_socket_path()).await
}

pub async fn send_msg(msg: &str) -> Result<(), Error> {
    let stream = connect().await?;
    loop {
        let ready = stream.ready(Interest::WRITABLE).await?;
        if ready.is_writable() {
            match stream.try_write(msg.as_bytes()) {
                Ok(_byte_amount) => {
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
