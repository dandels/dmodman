mod api;
mod cache;
mod cmd;
mod config;
mod messages;
mod nxm_listener;
mod ui;
mod util;

use api::Client;
use cache::Cache;
use config::Config;
use messages::Messages;
use std::error::Error;
use std::io::ErrorKind;
use std::str::FromStr;
use tokio::sync::mpsc::Receiver;
use std::rc::Rc;

/* dmodman acts as an url handler for nxm:// links in order for the "download with mod manager" button to work on
 * NexusMods.
 * - If the program is invoked without argument, it starts the TUI unless another instance is already running.
 * - If an nxm:// link is passed as an argument, we try to queue it in an already running instance. If none exists, we
 * start the TUI normally and queue the download.
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = cmd::args();

    let mut nxm_str_opt: Option<&str> = None;
    let mut nxm_game_opt: Option<String> = None;

    if let Some(unnamed_arg) = matches.value_of(cmd::ARG_UNNAMED) {
        if unnamed_arg.starts_with("nxm://") {
            let nxm = api::NxmUrl::from_str(unnamed_arg).expect("Unable to parse nxm url, aborting.");
            nxm_str_opt = Some(unnamed_arg);
            nxm_game_opt = Some(nxm.domain_name);
        } else {
            println!("Invalid unnamed argument. See --help for usage.");
            return Ok(());
        }
    }

    let config: Rc<Config> = Rc::new(Config::new(matches.value_of(cmd::ARG_GAME), nxm_game_opt).unwrap());

    /* Check if another instance for the same game is already running. If it is, optionally queue the download, then
     * exit early.
     */
    let nxm_rx;
    match queue_download_else_bind_to_socket(nxm_str_opt).await? {
        Some(v) => nxm_rx = v,
        None => return Ok(())
    }

    let msgs = Messages::default();
    let cache = Cache::new(&config).await.unwrap();
    let client = Client::new(&cache, &config, &msgs).unwrap();

    if let Some(nxm_str) = nxm_str_opt {
        client.queue_download(nxm_str.to_string()).await;
    }

    listen_for_downloads(&client, &msgs, nxm_rx);

    ui::init(&cache, &client, &config, &msgs).await?;
    Ok(())
}

/* Try bind to /run/user/$uid/dmodman.socket in order to queue downloads for nxm:// urls.
 * If the socket is already in use and the program was invoked with an nxm url, queue that download in the already
 * running instance and exit early.
 * If another instance is already running, we exit early.
 *
 * Returns Ok(None) if we we want to exit early, otherwise returns the mpsc receiver for the socket we bind to.
 */
async fn queue_download_else_bind_to_socket(
    nxm_str_opt: Option<&str>,
) -> Result<Option<Receiver<Result<String, std::io::Error>>>, std::io::Error> {
    let nxm_rx;
    match nxm_listener::listen() {
        Ok(v) => {
            nxm_rx = v;
        }
        /* If the address is in use, either another instance is using it or a previous instance was killed without
         * closing it.
         */
        Err(ref e) if e.kind() == ErrorKind::AddrInUse => {
            match nxm_listener::connect().await {
                // Another running instance is listening to the socket
                Ok(stream) => {
                    // If there's an nxm:// argument, queue it and exit
                    if let Some(nxm_str) = nxm_str_opt {
                        nxm_listener::send_msg(&stream, &nxm_str.as_bytes()).await?;
                        println!("Added download to already running instance: {}", nxm_str);
                        return Ok(None);
                    // otherwise just exit to avoid duplicate instances.
                    } else {
                        println!("Another instance of dmodman is already running.");
                        return Ok(None);
                    }
                }
                // Socket probably hasn't been cleanly removed. Remove it and bind to it.
                Err(ref e) if e.kind() == ErrorKind::ConnectionRefused => {
                    nxm_listener::remove_existing()?;
                    nxm_rx = nxm_listener::listen()?;
                }
                /* Catchall for unanticipated ways in which the socket can break. Hitting this case should be
                 * unlikely.
                 */
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
        }
        Err(e) => return Err(e),
    }
    Ok(Some(nxm_rx))
}

// Listen to socket for nxm links to download
fn listen_for_downloads(client: &Client, msgs: &Messages, mut nxm_rx: Receiver<Result<String, std::io::Error>>) {
    let client = client.clone();
    let msgs = msgs.clone();
    let _handle = tokio::task::spawn(async move {
        while let Some(socket_msg) = nxm_rx.recv().await {
            match socket_msg {
                Ok(msg) => {
                    if msg.starts_with("nxm://") {
                        match api::NxmUrl::from_str(&msg) {
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
