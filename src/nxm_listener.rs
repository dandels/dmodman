use std::io::{Error, ErrorKind};
use std::str;
use std::str::FromStr;
use tokio::io::Interest;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, mpsc::Receiver};
use tokio::task;

use crate::Messages;
use crate::api::{Client, NxmUrl};
use crate::config::Config;

// Listens for downloads to add
struct NxmListener {
    listener: UnixListener,
}

impl NxmListener {
    pub fn new() -> Result<Self, Error> {
        let uid = users::get_current_uid();
        let path = format!("/run/user/{}/dmodman.socket", uid);
        let listener = UnixListener::bind(path)?;
        Ok(Self { listener })
    }
}

// Remove the socket when program quits
impl Drop for NxmListener {
    fn drop(&mut self) {
        let addr = self.listener.local_addr().unwrap();
        let path = addr.as_pathname().unwrap();
        std::fs::remove_file(path).unwrap();
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

// Listen to socket for nxm links to download
pub fn listen_for_downloads(client: &Client, msgs: &Messages, mut nxm_rx: Receiver<Result<String, std::io::Error>>) {
    let client = client.clone();
    let msgs = msgs.clone();
    let _handle = tokio::task::spawn(async move {
        while let Some(socket_msg) = nxm_rx.recv().await {
            match socket_msg {
                Ok(msg) => {
                    if msg.starts_with("nxm://") {
                        match NxmUrl::from_str(&msg) {
                            Ok(_) => client.queue_download(msg).await,
                            Err(_e) => msgs.push(format!("Unable to parse string as a valid nxm url: {msg}")),
                        }
                    }
                },
                Err(e) => {
                    println!("{}", e.to_string());
                }
            }
        }
    });
}

/* Try bind to /run/user/$uid/dmodman.socket in order to queue downloads for nxm:// urls.
 * If the socket is already in use and the program was invoked with an nxm url, queue that download in the already
 * running instance and exit early.
 * If another instance is already running, we exit early.
 *
 * Returns Ok(None) if we we want to exit early, otherwise returns the mpsc receiver for the socket we bind to.
 */
pub async fn queue_download_else_bind_to_socket(
    nxm_str_opt: Option<&str>,
) -> Result<Option<Receiver<Result<String, std::io::Error>>>, std::io::Error> {
    match listen() {
        Ok(nxm_rx) => Ok(Some(nxm_rx)),
        /* If the address is in use, either another instance is using it or a previous instance was killed without
         * closing it.
         */
        Err(ref e) if e.kind() == ErrorKind::AddrInUse => {
            match connect().await {
                // Another running instance is listening to the socket
                Ok(stream) => {
                    // If there's an nxm:// argument, queue it and exit
                    if let Some(nxm_str) = nxm_str_opt {
                        send_msg(&stream, &nxm_str.as_bytes()).await?;
                        println!("Added download to already running instance: {}", nxm_str);
                        Ok(None)
                    // otherwise just exit to avoid duplicate instances.
                    } else {
                        println!("Another instance of dmodman is already running.");
                        Ok(None)
                    }
                }
                // Socket probably hasn't been cleanly removed. Remove it and bind to it.
                Err(ref e) if e.kind() == ErrorKind::ConnectionRefused => {
                    remove_existing()?;
                    Ok(Some(listen()?))
                }
                /* Catchall for unanticipated ways in which the socket can break. Hitting this case should be
                 * unlikely.
                 */
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
        }
        Err(e) => Err(e),
    }
}
